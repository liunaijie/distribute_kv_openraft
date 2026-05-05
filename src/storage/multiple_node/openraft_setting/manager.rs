use std::path::Path;
use std::sync::OnceLock;
use std::{collections::BTreeMap, sync::Arc};

use crate::storage::entity::DataEntity;
use crate::storage::multiple_node::openraft_setting::RaftInnerRequest;
use crate::storage::multiple_node::openraft_setting::network::Network;
use crate::storage::multiple_node::openraft_setting::network::proxy::RequestProxy;
use crate::storage::multiple_node::openraft_setting::store::new_storage;
use crate::storage::multiple_node::openraft_setting::store::state_machine::StateMachineStore;
use crate::storage::multiple_node::openraft_setting::type_config::{AppNode, TypeConfig};
use openraft::{Config, Raft, ReadPolicy};
use rocksdb::{ColumnFamilyDescriptor, DB, DBCompactionStyle, Options, SliceTransform};
use std::result::Result::Ok;
use tracing::{info, warn};

#[derive(Debug)]
pub(crate) struct OpenRaftManagerService {
    state_machine_store: StateMachineStore,
    raft_instance: Raft<TypeConfig>,
}

static RAFT_INSTANCE: OnceLock<Arc<Raft<TypeConfig>>> = OnceLock::new();

impl OpenRaftManagerService {
    pub async fn new(
        broker_id: u16,
        storage_path: String,
    ) -> Result<OpenRaftManagerService, anyhow::Error> {
        // Raft configuration
        let config = Config {
            heartbeat_interval: 250,
            election_timeout_min: 299,
            allow_log_reversion: Some(true),
            ..Default::default()
        };

        let config = Arc::new(match config.validate() {
            Ok(data) => data,
            Err(e) => {
                return Err(anyhow::format_err!(e));
            }
        });

        // Create storage layer (log store + state machine)
        info!("Initializing storage (log + state machine)...",);

        let log_store = Self::create_log_store_db(format!("{}/log", storage_path.clone())).unwrap();
        let state_machine =
            Self::create_state_machine_db(format!("{}/state", storage_path.clone())).unwrap();

        let snapshot_dir = Path::new(&storage_path).join("snapshots");

        let (log_store, state_machine_store) =
            new_storage(log_store, state_machine, snapshot_dir).await;

        // Create network layer
        let network = Network::new();

        // Create Raft instance
        info!("Creating Raft instance (node_id={})...", broker_id);
        let raft_instance = match Raft::new(
            broker_id,
            config.clone(),
            network,
            log_store,
            state_machine_store.clone(),
        )
        .await
        {
            Ok(raft_node) => {
                info!("Raft instance created successfully");
                Ok(raft_node)
            }
            Err(e) => Err(anyhow::format_err!("Failed to create Raft instance: {}", e)),
        }
        .unwrap();
        Ok(OpenRaftManagerService {
            state_machine_store,
            raft_instance,
        })
    }

    pub async fn start_raft_node(&self, member_list: Vec<String>) -> Result<(), anyhow::Error> {
        info!("Starting Raft node...");

        let mut nodes: BTreeMap<u16, AppNode> = BTreeMap::new();

        for node_def in member_list {
            let (id_part, addr_port) = node_def.split_once('@').unwrap();
            let node_id = id_part.parse().unwrap();
            let (ip, port) = addr_port.split_once(':').unwrap();
            let rpc_port = port.parse().unwrap();

            // 构建 AppNode
            let app_node = AppNode {
                node_id,
                rpc_addr: ip.to_string(),
                rpc_port,
            };
            nodes.insert(app_node.node_id, app_node);
        }

        // Print cluster members
        let node_list: Vec<String> = nodes
            .iter()
            .map(|(id, node)| format!("node_{}={}", id, node.rpc_addr))
            .collect();
        info!("Cluster members: [{}]", node_list.join(", "));

        // Check if already initialized
        match self.raft_instance.is_initialized().await {
            Ok(is_initialized) => {
                if !is_initialized {
                    info!("Initializing Raft cluster with {} nodes...", nodes.len());

                    match self.raft_instance.initialize(nodes.clone()).await {
                        Ok(_) => {
                            info!("Raft cluster initialized successfully");
                        }
                        Err(e) => {
                            return Err(anyhow::format_err!(
                                "Failed to initialize Raft cluster, {}",
                                e
                            ));
                        }
                    }
                } else {
                    info!("Raft cluster already initialized, skipping");
                }
            }
            Err(e) => {
                return Err(anyhow::format_err!(
                    "Failed to check initialization status: {}",
                    e
                ));
            }
        }

        info!("Raft node started successfully");
        let _ = RAFT_INSTANCE.set(Arc::new(self.raft_instance.clone()));
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn shutdown(&self) -> Result<(), anyhow::Error> {
        self.raft_instance
            .shutdown()
            .await
            .map_err(|e| anyhow::format_err!(e))?;
        Ok(())
    }
}

impl OpenRaftManagerService {
    pub async fn write_value(&self, key: String, value: DataEntity) -> Result<(), anyhow::Error> {
        let request = RaftInnerRequest::set(key, value.clone());
        let res = self.raft_instance.client_write(request).await;
        match res {
            Ok(_) => Ok(()),
            Err(e) => {
                if let Some(forward_err) = e.forward_to_leader() {
                    warn!("follow node write, need forward to leader");
                    if let Some(leader_node) = &forward_err.leader_node {
                        let leader_addr = &leader_node.rpc_addr;
                        let leader_port = &leader_node.rpc_port;
                        warn!("leader addr {}:{}", leader_addr, leader_port);
                        RequestProxy::get_proxy()
                            .forward_to_remote_leader(leader_addr, leader_port, value)
                            .await
                            .unwrap();
                        return Ok(());
                    } else {
                        return Err(anyhow::anyhow!(
                            "Leader found (ID:{:?}), but its node info is missing",
                            forward_err.leader_id
                        ));
                    }
                }
                Err(anyhow::anyhow!("Write failed: {}", e))
            }
        }
    }

    /**
     * 弱读，直接从本地状态读取，可能不全，可能读取到历史数据
     */
    pub async fn read_value(&self, key: &str) -> Result<Option<DataEntity>, anyhow::Error> {
        let res = self.state_machine_store.get_value(&key.to_string());
        res?.map(|bytes| DataEntity::from_bytes(&bytes)).transpose()
    }

    /**
     * 强一致读取
     */
    #[allow(dead_code)]
    pub async fn linearizable_read(&self, key: &str) -> Result<Option<DataEntity>, anyhow::Error> {
        let ret = self
            .raft_instance
            .get_read_linearizer(ReadPolicy::ReadIndex)
            .await;
        match ret {
            Ok(linearizer) => {
                linearizer.await_ready(&self.raft_instance).await.unwrap();
                let res = self.state_machine_store.get_value(&key.to_string());
                Ok(res?
                    .map(|bytes| DataEntity::from_bytes(&bytes))
                    .transpose()?)
            }
            Err(e) => Err(anyhow::format_err!(e)),
        }
    }
}

impl OpenRaftManagerService {
    fn create_log_store_db(storage_path: String) -> Result<Arc<DB>, anyhow::Error> {
        let db_opts = default_db_options();
        let meta = ColumnFamilyDescriptor::new("meta", Options::default());
        let logs = ColumnFamilyDescriptor::new("logs", Options::default());

        let db = DB::open_cf_descriptors(&db_opts, storage_path, vec![meta, logs])
            .map_err(|e| anyhow::Error::msg(format!("Failed to open DB: {}", e)))?;
        Ok(Arc::new(db))
    }

    fn create_state_machine_db(storage_path: String) -> Result<Arc<DB>, anyhow::Error> {
        let db_opts = default_db_options();
        let sm_meta = ColumnFamilyDescriptor::new("sm_meta", Options::default());
        let sm_data = ColumnFamilyDescriptor::new("sm_data", Options::default());

        let db = DB::open_cf_descriptors(&db_opts, storage_path, vec![sm_data, sm_meta])
            .map_err(|e| anyhow::Error::msg(format!("Failed to open DB: {}", e)))?;
        Ok(Arc::new(db))
    }
}

impl OpenRaftManagerService {
    pub fn get_raft_instance() -> &'static Arc<Raft<TypeConfig>> {
        RAFT_INSTANCE.get().expect("RAFT_INSTANCE 未初始化")
    }
}

fn default_db_options() -> Options {
    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.create_missing_column_families(true);
    opts.set_max_open_files(1000);
    opts.set_use_fsync(false);
    opts.set_bytes_per_sync(8388608);
    opts.optimize_for_point_lookup(1024);
    opts.set_table_cache_num_shard_bits(6);
    opts.set_max_write_buffer_number(32);
    opts.set_write_buffer_size(536870912);
    opts.set_target_file_size_base(1073741824);
    opts.set_min_write_buffer_number_to_merge(4);
    opts.set_level_zero_stop_writes_trigger(2000);
    opts.set_level_zero_slowdown_writes_trigger(0);
    opts.set_compaction_style(DBCompactionStyle::Universal);
    opts.set_disable_auto_compactions(true);
    let transform = SliceTransform::create_fixed_prefix(10);
    opts.set_prefix_extractor(transform);
    opts.set_memtable_prefix_bloom_ratio(0.2);
    opts
}
