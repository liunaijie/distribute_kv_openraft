use anyhow::Result;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing() -> Result<()> {
    let filter = EnvFilter::new("off,distribute_kv_openraft=info");

    let file_layer = fmt::layer()
        .with_timer(fmt::time::LocalTime::rfc_3339())
        .with_ansi(false)
        .with_target(true)
        .with_file(true)
        .with_thread_names(true)
        .with_line_number(true)
        .with_level(true);

    tracing_subscriber::registry()
        .with(file_layer)
        .with(filter)
        .init();

    Ok(())
}
