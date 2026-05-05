use std::sync::{Arc, OnceLock};

use async_trait::async_trait;

use anyhow::Error;

use crate::storage::memory::MemoryStorage;
use crate::storage::{entity::DataEntity, rocksdb::RocksdbStorage};

pub mod entity;
mod memory;
mod raft;
mod rocksdb;
/*
* 存储的接口
*/
#[async_trait]
pub trait StorageService {
    async fn set(&self, entity: DataEntity) -> Result<(), Error>;
    async fn get(&self, key: &str) -> Result<Option<DataEntity>, Error>;
    async fn delete(&self, key: &str) -> Result<(), Error>;
}

static STORAGE: OnceLock<Arc<dyn StorageService + Send + Sync>> = OnceLock::new();

pub fn init_storage(storage_type: &str, storage_path: String) {
    STORAGE.get_or_init(|| create_storage(storage_type, storage_path));
}

pub fn get_storage() -> &'static Arc<dyn StorageService + Send + Sync> {
    STORAGE
        .get()
        .expect("Storage not initialized. Call init_storage() first.")
}

fn create_storage(
    storage_type: &str,
    storage_path: String,
) -> Arc<dyn StorageService + Send + Sync> {
    match storage_type {
        "rocksdb" => {
            let rocksdb = RocksdbStorage::new(storage_path).unwrap();
            Arc::new(rocksdb)
        }
        "memory" => {
            let memory = MemoryStorage::new().unwrap();
            Arc::new(memory)
        }
        _ => {
            panic!("Unsupported storage type: {}", storage_type);
        }
    }
}
