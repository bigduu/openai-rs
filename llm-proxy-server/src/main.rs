mod app;
mod config;

use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, fmt::format::FmtSpan, EnvFilter, Registry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,server=debug,core=debug"));

    let formatting_layer = fmt::layer()
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .pretty();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(formatting_layer)
        .init();

    // Load configuration
    let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
    info!("Loading configuration from {}", config_path);
    let config = config::Config::from_file(&config_path)?;

    // Start server
    app::run_server(config).await?;

    Ok(())
}
