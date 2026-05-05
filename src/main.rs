use anyhow::{Error, Result};
use tracing::info;

mod axum;
mod raft;
mod storage;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Error> {
    utils::tracking_utils::init_tracing()?;
    let system_config = utils::config::load_config();
    info!(
        "starting with port {} and path {}",
        system_config.port, system_config.storage_path
    );
    storage::init_storage("rocksdb", format!("rocksdb/{}", system_config.storage_path));
    // storage::init_storage("memory", format!("rocksdb/{}",system_config.storage_path));

    axum::start_axum_server(system_config.port).await;
    Ok(())
}
