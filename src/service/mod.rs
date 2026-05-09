use std::sync::{Arc, OnceLock};

use anyhow::Ok;

use crate::{
    raft::manager::OpenRaftManager,
    service::storage_service::{OpenRaftStorageServiceImpl, StorageService},
    utils::config::AppConfig,
};

mod storage_service;

pub static APP_STATE: OnceLock<AppServiceState> = OnceLock::new();

#[derive(Clone)]
pub struct AppServiceState {
    pub raft_manager: Arc<OpenRaftManager>,
    pub storage: Arc<dyn StorageService + 'static + Send + Sync>,
}

pub async fn init_service(config: &AppConfig) -> Result<(), anyhow::Error> {
    let raft_manager = OpenRaftManager::new(config.node_id, &config.storage_path, 10).await?;
    raft_manager.start_raft_node(&config.member_list).await?;

    let storage_service = Arc::new(OpenRaftStorageServiceImpl::new(raft_manager.clone()));

    let state = AppServiceState {
        raft_manager: Arc::new(raft_manager),
        storage: storage_service,
    };

    APP_STATE
        .set(state)
        .map_err(|_| anyhow::anyhow!("APP_STATE 初始化失败，重复设置"))?;

    Ok(())
}
