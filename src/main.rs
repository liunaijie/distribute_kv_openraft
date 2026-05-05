use anyhow::{Error, Result};
use clap::Parser;
use tracing::info;

use crate::utils::config::CliArgs;

mod axum;
mod storage;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Error> {
    utils::tracking_utils::init_tracing()?;
    info!("starting service...");
    let args = CliArgs::parse();

    let app_config = args.validate()?;

    info!(
        "starting with port {} and storage type {}",
        app_config.port, app_config.storage_type
    );

    storage::init_storage(app_config.storage_type.clone()).await;

    axum::start_axum_server(app_config.port, app_config.is_multiple_node()).await;
    Ok(())
}
