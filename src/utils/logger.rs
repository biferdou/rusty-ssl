use crate::utils::config::LoggingConfig;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging(config: &LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
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
