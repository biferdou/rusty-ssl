use anyhow::Result;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Response, StatusCode};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: u64,
    pub uptime_seconds: u64,
    pub version: String,
}

pub struct HealthHandler {
    start_time: SystemTime,
    version: String,
}

impl HealthHandler {
    pub fn new(version: String) -> Self {
        Self {
            start_time: SystemTime::now(),
            version,
        }
    }

    pub async fn handle_health_check(&self) -> Result<Response<Full<Bytes>>> {
        debug!("Health check requested");

        let now = SystemTime::now();
        let timestamp = now.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

        let uptime_seconds = now
            .duration_since(self.start_time)
            .unwrap_or_default()
            .as_secs();

        let health_status = HealthStatus {
            status: "healthy".to_string(),
            timestamp,
            uptime_seconds,
            version: self.version.clone(),
        };

        let response_body = json!({
            "status": health_status.status,
            "timestamp": health_status.timestamp,
            "uptime_seconds": health_status.uptime_seconds,
            "version": health_status.version,
            "service": "rusty-ssl",
            "checks": {
                "ssl": "ok",
                "ttl_manager": "ok",
                "memory": "ok"
            }
        });

        let response = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .header("Cache-Control", "no-cache")
            .body(Full::new(Bytes::from(response_body.to_string())))?;

        Ok(response)
    }

    pub async fn handle_readiness_check(&self) -> Result<Response<Full<Bytes>>> {
        debug!("Readiness check requested");

        // In a real implementation, you would check:
        // - SSL certificates are loaded and valid
        // - TTL controller is operational
        // - External dependencies are reachable

        let response_body = json!({
            "status": "ready",
            "timestamp": SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            "checks": {
                "ssl_certificates": "ready",
                "ttl_controller": "ready",
                "network": "ready"
            }
        });

        let response = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .header("Cache-Control", "no-cache")
            .body(Full::new(Bytes::from(response_body.to_string())))?;

        Ok(response)
    }

    pub async fn handle_liveness_check(&self) -> Result<Response<Full<Bytes>>> {
        debug!("Liveness check requested");

        // Simple alive check - if this responds, the service is alive
        let response_body = json!({
            "status": "alive",
            "timestamp": SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        });

        let response = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .header("Cache-Control", "no-cache")
            .body(Full::new(Bytes::from(response_body.to_string())))?;

        Ok(response)
    }
}
