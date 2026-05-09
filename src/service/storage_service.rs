use crate::{raft::manager::OpenRaftManager, utils::entity::DataEntity};
use async_trait::async_trait;

#[async_trait]
pub trait StorageService {
    async fn set(&self, key: &str, value: &str) -> Result<DataEntity, anyhow::Error>;
    async fn get(
        &self,
        key: &str,
        linearizable_read: bool,
    ) -> Result<Option<DataEntity>, anyhow::Error>;
    async fn delete(&self, key: &str) -> Result<Option<DataEntity>, anyhow::Error>;
}

#[derive(Debug)]
pub(crate) struct OpenRaftStorageServiceImpl {
    openraft_manager: OpenRaftManager,
}

impl OpenRaftStorageServiceImpl {
    pub fn new(openraft_manager: OpenRaftManager) -> Self {
        Self { openraft_manager }
    }
}

#[async_trait]
impl StorageService for OpenRaftStorageServiceImpl {
    async fn set(&self, key: &str, value: &str) -> Result<DataEntity, anyhow::Error> {
        let entity = DataEntity::new(key.to_string(), value.to_string());
        self.openraft_manager.set_value(entity).await
    }

    async fn get(
        &self,
        key: &str,
        linearizable_read: bool,
    ) -> Result<Option<DataEntity>, anyhow::Error> {
        self.openraft_manager
            .get_value(key, linearizable_read)
            .await
    }

    async fn delete(&self, key: &str) -> Result<Option<DataEntity>, anyhow::Error> {
        self.openraft_manager.del_value(key).await
    }
}
