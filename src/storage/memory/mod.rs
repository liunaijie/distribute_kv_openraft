use crate::storage::StorageService;
use crate::storage::entity::DataEntity;
use anyhow::Error;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;

pub(crate) struct MemoryStorage {
    map: RwLock<HashMap<String, DataEntity>>,
}

impl MemoryStorage {
    pub fn new() -> Result<MemoryStorage, anyhow::Error> {
        Ok(MemoryStorage {
            map: RwLock::new(HashMap::new()),
        })
    }
}

#[async_trait]
impl StorageService for MemoryStorage {
    async fn set(&self, entity: DataEntity) -> Result<(), Error> {
        let mut map = self.map.write().await;
        map.insert(entity.key.clone(), entity);
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<DataEntity>, Error> {
        let map = self.map.read().await;
        Ok(map.get(key).cloned())
    }

    async fn delete(&self, key: &str) -> Result<(), Error> {
        let mut map = self.map.write().await;
        map.remove(key);
        Ok(())
    }
}
