use crate::utils::config::LoggingConfig;
use anyhow::Result;
use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging(config: &LoggingConfig) -> Result<()> {
    let filter =
        EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new(&config.level))?;

    let subscriber = tracing_subscriber::registry().with(filter);

    match config.format.as_str() {
        "json" => {
            subscriber
                .with(tracing_subscriber::fmt::layer().json())
                .try_init()?;
        }
        _ => {
            subscriber
                .with(tracing_subscriber::fmt::layer().pretty())
                .try_init()?;
        }
    }

    tracing::info!("Logger initialized with level: {}", config.level);
    Ok(())
}
