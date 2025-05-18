use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub ssl: SslConfig,
    pub ttl: TtlConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub request_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslConfig {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub cert_check_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtlConfig {
    pub default_ttl_secs: u64,
    pub max_ttl_secs: u64,
    pub cleanup_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String, // "json" or "pretty"
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8443,
                max_connections: 1000,
                request_timeout_secs: 30,
            },
            ssl: SslConfig {
                cert_path: PathBuf::from("/etc/letsencrypt/live/tilas.xyz/fullchain.pem"),
                key_path: PathBuf::from("/etc/letsencrypt/live/tilas.xyz/privkey.pem"),
                cert_check_interval_secs: 3600, // Check every hour
            },
            ttl: TtlConfig {
                default_ttl_secs: 300,     // 5 minutes
                max_ttl_secs: 3600,        // 1 hour
                cleanup_interval_secs: 60, // Cleanup every minute
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "pretty".to_string(),
            },
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        let settings = config::Config::builder()
            .add_source(config::Config::try_from(&AppConfig::default())?)
            .add_source(config::File::with_name("configs/default").required(false))
            .add_source(config::File::with_name("configs/production").required(false))
            .add_source(config::Environment::with_prefix("RUSTY_SSL"))
            .build()?;

        settings.try_deserialize()
    }

    pub fn server_addr(&self) -> Result<SocketAddr, std::net::AddrParseError> {
        format!("{}:{}", self.server.host, self.server.port).parse()
    }

    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.server.request_timeout_secs)
    }

    pub fn default_ttl(&self) -> Duration {
        Duration::from_secs(self.ttl.default_ttl_secs)
    }

    pub fn max_ttl(&self) -> Duration {
        Duration::from_secs(self.ttl.max_ttl_secs)
    }

    pub fn cleanup_interval(&self) -> Duration {
        Duration::from_secs(self.ttl.cleanup_interval_secs)
    }

    pub fn cert_check_interval(&self) -> Duration {
        Duration::from_secs(self.ssl.cert_check_interval_secs)
    }
}
