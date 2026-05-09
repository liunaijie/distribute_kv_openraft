use std::{collections::BTreeMap, path::Path, sync::Arc};

use openraft::{Config, Raft, ReadPolicy};
use tracing::{error, info};

use crate::{
    raft::{
        network::Network,
        store::{new_storage, rocksdb::RocksDBEngine, state_machine::StateMachineStore},
        type_config::{RaftRpcRequest, RaftRpcRequestType, TypeConfig},
    },
    utils::{config::NodeRpcInfo, entity::DataEntity},
};

#[derive(Debug, Clone)]
pub(crate) struct OpenRaftManager {
    pub raft_node: Raft<TypeConfig>,
    state_machine: StateMachineStore,
}

/**
 * lifecycle related
 */
impl OpenRaftManager {
    pub async fn new(
        broker_id: u16,
        db_path: &str,
        max_open_files: i32,
    ) -> Result<Self, anyhow::Error> {
        // Create Raft instance
        info!("Creating Raft instance (node_id={})...", broker_id);
        // Raft configuration
        let config = Arc::new(Config {
            heartbeat_interval: 250,
            election_timeout_min: 299,
            allow_log_reversion: Some(true),
            ..Default::default()
        });

        let db_engine = Arc::new(RocksDBEngine::new(
            db_path,
            max_open_files,
            vec!["meta", "logs", "sm_meta", "sm_data"],
        ));

        let snapshot_dir = Path::new(db_path).join("snapshots");

        let (log_store, state_machine_store) = new_storage(db_engine, snapshot_dir).await;

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

        Ok(OpenRaftManager {
            raft_node: raft_instance,
            state_machine: state_machine_store,
        })
    }

    pub async fn start_raft_node(
        &self,
        member_list: &Vec<NodeRpcInfo>,
    ) -> Result<(), anyhow::Error> {
        info!("Starting Raft node...");

        let mut nodes: BTreeMap<u16, NodeRpcInfo> = BTreeMap::new();

        for node_def in member_list {
            nodes.insert(node_def.node_id, node_def.clone());
        }

        // Print cluster members
        let node_list: Vec<String> = nodes
            .iter()
            .map(|(id, node)| format!("node_{}={}", id, node.rpc_addr))
            .collect();
        info!("Cluster members: [{}]", node_list.join(", "));

        // Check if already initialized
        match self.raft_node.is_initialized().await {
            Ok(is_initialized) => {
                if !is_initialized {
                    info!("Initializing Raft cluster with {} nodes...", nodes.len());

                    match self.raft_node.initialize(nodes.clone()).await {
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
        Ok(())
    }
}

impl OpenRaftManager {
    pub async fn set_value(&self, entity: DataEntity) -> Result<DataEntity, anyhow::Error> {
        let request = RaftRpcRequest::new(RaftRpcRequestType::KvSet, entity.as_bytes().unwrap());
        let res = self.raft_node.client_write(request.clone()).await;
        match res {
            Ok(e) => {
                if e.data.value.is_none() {
                    return Err(anyhow::format_err!("set返回信息为空"));
                }
                let bytes = e.data.value.unwrap();
                Ok(DataEntity::from_bytes(&bytes)?)
            }
            Err(e) => {
                if let Some(forward_err) = e.forward_to_leader() {
                    if let Some(leader_node) = &forward_err.leader_node {
                        let leader_addr = &leader_node.rpc_addr;
                        let leader_port = &leader_node.rpc_port;
                        return Err(anyhow::format_err!(
                            "please access leader node {}:{}",
                            leader_addr,
                            leader_port
                        ));
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

    pub async fn get_value(
        &self,
        key: &str,
        linearizable_read: bool,
    ) -> Result<Option<DataEntity>, anyhow::Error> {
        if linearizable_read {
            let linearizer_result = self
                .raft_node
                .get_read_linearizer(ReadPolicy::ReadIndex)
                .await;
            match linearizer_result {
                Ok(linearizer) => {
                    linearizer.await_ready(&self.raft_node).await?;
                }
                Err(e) => {
                    error!("Failed to get linearizer for linearizable read: {}", e);
                    return Err(anyhow::format_err!(e))
                }
            }
        }
        let res = self.state_machine.get_value(&key.to_string());
        res?.map(|bytes| DataEntity::from_bytes(&bytes)).transpose()
    }

    pub async fn del_value(&self, key: &str) -> Result<Option<DataEntity>, anyhow::Error> {
        let request = RaftRpcRequest::new(RaftRpcRequestType::KvDelete, key.into());
        let res = self.raft_node.client_write(request.clone()).await;
        match res {
            Ok(e) => {
                if let Some(bytes) = e.data.value {
                    Ok(Some(DataEntity::from_bytes(&bytes)?))
                } else {
                    Ok(None)
                }
            }
            Err(e) => {
                if let Some(forward_err) = e.forward_to_leader() {
                    if let Some(leader_node) = &forward_err.leader_node {
                        let leader_addr = &leader_node.rpc_addr;
                        let leader_port = &leader_node.rpc_port;
                        return Err(anyhow::format_err!(
                            "please access leader node {}:{}",
                            leader_addr,
                            leader_port
                        ));
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
}
