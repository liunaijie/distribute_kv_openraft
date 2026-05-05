mod openraft_setting;

pub(crate) use openraft_setting::manager::OpenRaftManagerService;
pub(crate) use openraft_setting::type_config;

use crate::storage::StorageService;
use crate::storage::entity::DataEntity;
use anyhow::Error;
use async_trait::async_trait;

#[derive(Debug)]
pub(crate) struct MultipleNodeStorage {
    openraft_service: OpenRaftManagerService,
}

impl MultipleNodeStorage {
    pub async fn new(
        broker_id: u16,
        storage_path: String,
        member_list: Vec<String>,
    ) -> Result<MultipleNodeStorage, anyhow::Error> {
        let service = OpenRaftManagerService::new(broker_id, storage_path).await?;
        service.start_raft_node(member_list).await.unwrap();
        Ok(MultipleNodeStorage {
            openraft_service: service,
        })
    }
}

#[async_trait]
impl StorageService for MultipleNodeStorage {
    async fn set(&self, entity: DataEntity) -> Result<(), Error> {
        self.openraft_service
            .write_value(entity.key.clone(), entity)
            .await
    }

    async fn get(&self, key: &str) -> Result<Option<DataEntity>, Error> {
        self.openraft_service.read_value(key).await
    }

    async fn delete(&self, _key: &str) -> Result<(), Error> {
        todo!()
    }
}
