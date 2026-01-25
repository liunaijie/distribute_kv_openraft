use anyhow::{Error, Result};
use tracing::info;

mod api;
mod storage;
mod utils;

use storage::StorageService;

use crate::storage::rocksdb::RocksdbStorage;

#[tokio::main]
async fn main() -> Result<(), Error> {
    utils::tracking_utils::init_tracing()?;
    let storage: &dyn StorageService<String> = &RocksdbStorage::new("rocksdb/app_storage.db")?;

    // storage.set("test", "test2")?;
    let value = storage.get("test")?;
    info!("get value: {} from key: test", value);

    // api::start_http_server(8080).await.unwrap();
    // info!("HTTP server started");
    Ok(())
}
