use anyhow::{Error, Result};
use clap::Parser;
use tracing::info;

use crate::utils::config::CliArgs;

mod axum;
mod raft;
mod service;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Error> {
    utils::tracking_utils::init_tracing()?;
    let args = CliArgs::parse();
    let app_config = args.validate()?;
    info!("starting kv service with config :  {}", app_config);

    service::init_service(&app_config).await?;

    axum::start_axum_server(app_config.api_port).await?;
    Ok(())
}
