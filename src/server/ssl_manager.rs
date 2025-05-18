use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::time::{Interval, interval};
use tracing::{error, info, warn};

#[derive(Error, Debug)]
pub enum SslError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("TLS error: {0}")]
    Tls(#[from] rustls::Error),
    #[error("Certificate not found: {cert_path}")]
    CertificateNotFound { cert_path: String },
    #[error("Private key not found: {key_path}")]
    PrivateKeyNotFound { key_path: String },
    #[error("No valid certificates found in file")]
    NoCertificatesFound,
    #[error("No valid private keys found in file")]
    NoPrivateKeysFound,
}

#[derive(Debug, Clone)]
pub struct CertificateInfo {
    pub not_before: SystemTime,
    pub not_after: SystemTime,
    pub is_expired: bool,
    pub days_until_expiry: i64,
}

pub struct SslManager {
    config: Arc<ServerConfig>,
    cert_path: std::path::PathBuf,
    key_path: std::path::PathBuf,
    cert_info: Option<CertificateInfo>,
    check_interval: Interval,
}

impl SslManager {
    pub fn new(
        cert_path: impl AsRef<Path>,
        key_path: impl AsRef<Path>,
        check_interval: Duration,
    ) -> Result<Self, SslError> {
        let cert_path = cert_path.as_ref().to_path_buf();
        let key_path = key_path.as_ref().to_path_buf();

        info!(
            "Loading SSL certificates from: {} and {}",
            cert_path.display(),
            key_path.display()
        );

        let config = Self::load_certificates(&cert_path, &key_path)?;
        let cert_info = Self::extract_certificate_info(&cert_path)?;

        info!(
            "SSL certificates loaded successfully. Expires: {:?}",
            cert_info.as_ref().map(|info| info.not_after)
        );

        Ok(Self {
            config: Arc::new(config),
            cert_path,
            key_path,
            cert_info,
            check_interval: interval(check_interval),
        })
    }

    fn load_certificates(cert_path: &Path, key_path: &Path) -> Result<ServerConfig, SslError> {
        // Load certificate chain
        let cert_file = File::open(cert_path).map_err(|_| SslError::CertificateNotFound {
            cert_path: cert_path.display().to_string(),
        })?;
        let mut cert_reader = BufReader::new(cert_file);
        let cert_chain: Vec<Certificate> = certs(&mut cert_reader)?
            .into_iter()
            .map(Certificate)
            .collect();

        if cert_chain.is_empty() {
            return Err(SslError::NoCertificatesFound);
        }

        // Load private key
        let key_file = File::open(key_path).map_err(|_| SslError::PrivateKeyNotFound {
            key_path: key_path.display().to_string(),
        })?;
        let mut key_reader = BufReader::new(key_file);
        let mut keys: Vec<PrivateKey> = pkcs8_private_keys(&mut key_reader)?
            .into_iter()
            .map(PrivateKey)
            .collect();

        if keys.is_empty() {
            return Err(SslError::NoPrivateKeysFound);
        }

        // Configure TLS
        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(cert_chain, keys.remove(0))?;

        Ok(config)
    }

    fn extract_certificate_info(cert_path: &Path) -> Result<CertificateInfo, SslError> {
        // This is a simplified version - in production you'd parse the X.509 certificate
        // For now, we'll just check file modification time as a proxy
        let metadata = std::fs::metadata(cert_path)?;
        let modified = metadata.modified()?;

        // Let's Encrypt certificates are valid for 90 days
        let expires_in = Duration::from_secs(90 * 24 * 60 * 60);
        let not_after = modified + expires_in;

        let now = SystemTime::now();
        let is_expired = now > not_after;

        let days_until_expiry = if let Ok(duration) = not_after.duration_since(now) {
            duration.as_secs() as i64 / (24 * 60 * 60)
        } else {
            -1 // Expired
        };

        Ok(CertificateInfo {
            not_before: modified,
            not_after,
            is_expired,
            days_until_expiry,
        })
    }

    pub fn get_config(&self) -> Arc<ServerConfig> {
        self.config.clone()
    }

    pub fn get_certificate_info(&self) -> Option<&CertificateInfo> {
        self.cert_info.as_ref()
    }

    pub async fn start_certificate_monitoring(&mut self) {
        info!("Starting certificate monitoring");

        loop {
            self.check_interval.tick().await;

            match Self::extract_certificate_info(&self.cert_path) {
                Ok(cert_info) => {
                    if cert_info.is_expired {
                        error!("Certificate has expired!");
                    } else if cert_info.days_until_expiry <= 7 {
                        warn!(
                            "Certificate expires in {} days",
                            cert_info.days_until_expiry
                        );
                    } else {
                        info!(
                            "Certificate is valid, expires in {} days",
                            cert_info.days_until_expiry
                        );
                    }

                    self.cert_info = Some(cert_info);
                }
                Err(e) => {
                    error!("Failed to check certificate: {}", e);
                }
            }
        }
    }

    pub async fn reload_certificates(&mut self) -> Result<(), SslError> {
        info!("Reloading SSL certificates");

        let new_config = Self::load_certificates(&self.cert_path, &self.key_path)?;
        let new_cert_info = Self::extract_certificate_info(&self.cert_path)?;

        self.config = Arc::new(new_config);
        self.cert_info = Some(new_cert_info);

        info!("SSL certificates reloaded successfully");
        Ok(())
    }
}
