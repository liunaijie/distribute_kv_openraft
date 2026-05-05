use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::OnceCell;

use async_trait::async_trait;

use anyhow::Error;

use crate::storage::multiple_node::MultipleNodeStorage;
use crate::storage::{entity::DataEntity, rocksdb::RocksdbStorage};
use crate::utils::config::StorageType;

pub mod entity;
mod memory;
mod multiple_node;
mod rocksdb;

pub use memory::MemoryStorage;

pub(crate) use multiple_node::OpenRaftManagerService;
pub(crate) use multiple_node::type_config;
/*
* 存储的接口
*/
#[async_trait]
pub trait StorageService {
    async fn set(&self, entity: DataEntity) -> Result<(), Error>;
    async fn get(&self, key: &str) -> Result<Option<DataEntity>, Error>;
    async fn delete(&self, key: &str) -> Result<(), Error>;
}

static STORAGE: OnceCell<Arc<dyn StorageService + Send + Sync>> = OnceCell::const_new();

static INIT_TAKEN: AtomicBool = AtomicBool::new(false);

pub async fn init_storage(storage_type: StorageType) {
    let already_taken = INIT_TAKEN.swap(true, Ordering::AcqRel);
    if already_taken {
        panic!("仅能初始化一次（包含并发/重复调用）");
    }

    let storage = create_storage(storage_type)
        .await
        .unwrap_or_else(|e| panic!("初始化 storage 失败: {e}"));

    if STORAGE.set(storage).is_err() {
        panic!("storage OnceCell 已被初始化（逻辑错误：与 INIT_TAKEN 不一致）");
    }
}

pub fn get_storage() -> &'static Arc<dyn StorageService + Send + Sync> {
    if !STORAGE.initialized() {
        panic!("初始化失败，无法获取storage")
    }
    STORAGE.get().unwrap()
}

async fn create_storage(
    storage_type: StorageType,
) -> Result<Arc<dyn StorageService + Send + Sync>, anyhow::Error> {
    match storage_type {
        StorageType::RocksDB(storage_path) => {
            let rocksdb = RocksdbStorage::new(storage_path)?;
            Ok(Arc::new(rocksdb) as Arc<dyn StorageService + Send + Sync>)
        }
        StorageType::Memory => {
            let memory = MemoryStorage::new()?;
            Ok(Arc::new(memory) as Arc<dyn StorageService + Send + Sync>)
        }
        StorageType::MultipleNode(node_id, storage_path, member_list) => {
            let multiple_node =
                MultipleNodeStorage::new(node_id, storage_path, member_list).await?;
            Ok(Arc::new(multiple_node) as Arc<dyn StorageService + Send + Sync>)
        }
    }
}
