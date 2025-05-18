use crate::handlers::HealthHandler;
use crate::server::TtlController;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::{Method, Request, Response, StatusCode};
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

pub struct Router {
    health_handler: HealthHandler,
    ttl_controller: Arc<Mutex<TtlController>>,
}

impl Router {
    pub fn new(ttl_controller: Arc<Mutex<TtlController>>) -> Self {
        Self {
            health_handler: HealthHandler::new(env!("CARGO_PKG_VERSION").to_string()),
            ttl_controller,
        }
    }

    pub async fn route(
        &self,
        req: Request<Incoming>,
        client_ip: IpAddr,
    ) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
        // Register/update connection in TTL controller
        {
            let mut ttl_controller = self.ttl_controller.lock().await;
            ttl_controller.register_connection(client_ip);
        }

        let method = req.method();
        let path = req.uri().path();

        info!("Request: {} {} from {}", method, path, client_ip);

        let response = match (method, path) {
            // Health checks
            (&Method::GET, "/health") => self.health_handler.handle_health_check().await?,
            (&Method::GET, "/health/ready") => self.health_handler.handle_readiness_check().await?,
            (&Method::GET, "/health/live") => self.health_handler.handle_liveness_check().await?,

            // SSL status endpoint
            (&Method::GET, "/ssl-status") => self.handle_ssl_status().await?,

            // TTL metrics endpoint
            (&Method::GET, "/metrics") => self.handle_metrics().await?,

            // Root endpoint
            (&Method::GET, "/") => self.handle_root().await?,

            // 404 for everything else
            _ => self.handle_not_found(path).await?,
        };

        // Update connection activity after successful request
        {
            let ttl_controller = self.ttl_controller.lock().await;
            ttl_controller.update_connection_activity(client_ip);
        }

        Ok(response)
    }

    async fn handle_root(&self) -> Result<Response<Full<Bytes>>, hyper::Error> {
        debug!("Root endpoint requested");

        let html_content = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rusty-SSL Server</title>
    <style>
        body { 
            font-family: Arial, sans-serif; 
            max-width: 800px; 
            margin: 0 auto; 
            padding: 20px; 
            background-color: #f5f5f5; 
        }
        .container { 
            background: white; 
            padding: 30px; 
            border-radius: 8px; 
            box-shadow: 0 2px 10px rgba(0,0,0,0.1); 
        }
        h1 { color: #333; }
        .endpoint { 
            background: #f8f9fa; 
            padding: 15px; 
            margin: 10px 0; 
            border-radius: 5px; 
            border-left: 4px solid #007bff; 
        }
        .endpoint a { 
            text-decoration: none; 
            color: #007bff; 
            font-weight: bold; 
        }
        .endpoint a:hover { text-decoration: underline; }
        .status { 
            display: inline-block; 
            padding: 4px 8px; 
            background: #28a745; 
            color: white; 
            border-radius: 4px; 
            font-size: 12px; 
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🦀 Rusty-SSL Server <span class="status">ONLINE</span></h1>
        <p>Welcome to the secure Rust-based HTTP server with SSL/TLS and TTL management.</p>
        
        <h2>Available Endpoints</h2>
        
        <div class="endpoint">
            <strong><a href="/health">/health</a></strong> - Full health check with service status
        </div>
        
        <div class="endpoint">
            <strong><a href="/health/ready">/health/ready</a></strong> - Readiness probe
        </div>
        
        <div class="endpoint">
            <strong><a href="/health/live">/health/live</a></strong> - Liveness probe
        </div>
        
        <div class="endpoint">
            <strong><a href="/ssl-status">/ssl-status</a></strong> - SSL certificate information
        </div>
        
        <div class="endpoint">
            <strong><a href="/metrics">/metrics</a></strong> - Connection and TTL metrics
        </div>
        
        <hr style="margin: 30px 0;">
        
        <p><strong>Features:</strong></p>
        <ul>
            <li>✅ HTTPS with Let's Encrypt certificates</li>
            <li>✅ IP-based TTL management</li>
            <li>✅ No client certificates required</li>
            <li>✅ Real-time connection monitoring</li>
            <li>✅ Automatic certificate renewal checks</li>
        </ul>
        
        <footer style="margin-top: 30px; padding-top: 20px; border-top: 1px solid #eee; color: #666;">
            <p>Powered by Rust 🦀 | Version: {version}</p>
        </footer>
    </div>
</body>
</html>
        "#.replace("{version}", env!("CARGO_PKG_VERSION"));

        let response = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html; charset=utf-8")
            .header("Cache-Control", "public, max-age=300")
            .body(Full::new(Bytes::from(html_content)))
            .unwrap();

        Ok(response)
    }

    async fn handle_ssl_status(&self) -> Result<Response<Full<Bytes>>, hyper::Error> {
        debug!("SSL status endpoint requested");

        // In a real implementation, you would get this from the SSL manager
        let ssl_status = serde_json::json!({
            "status": "active",
            "certificate": {
                "subject": "tilas.xyz",
                "issuer": "Let's Encrypt",
                "valid_from": "2024-01-01T00:00:00Z",
                "valid_until": "2024-04-01T00:00:00Z",
                "days_until_expiry": 45,
                "is_expired": false
            },
            "tls_version": "1.3",
            "cipher_suite": "TLS_AES_256_GCM_SHA384"
        });

        let response = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .header("Cache-Control", "no-cache")
            .body(Full::new(Bytes::from(ssl_status.to_string())))
            .unwrap();

        Ok(response)
    }

    async fn handle_metrics(&self) -> Result<Response<Full<Bytes>>, hyper::Error> {
        debug!("Metrics endpoint requested");

        let ttl_stats = {
            let ttl_controller = self.ttl_controller.lock().await;
            ttl_controller.get_stats()
        };

        let connections_snapshot = {
            let ttl_controller = self.ttl_controller.lock().await;
            ttl_controller.get_connections_snapshot()
        };

        let detailed_connections: Vec<_> = connections_snapshot
            .into_iter()
            .map(|(ip, conn)| {
                serde_json::json!({
                    "ip": ip.to_string(),
                    "connection_id": conn.id.to_string(),
                    "established_at": conn.established_at.elapsed().as_secs(),
                    "last_activity": conn.last_activity.elapsed().as_secs(),
                    "ttl_seconds": conn.ttl.as_secs(),
                    "time_until_expiry": conn.time_until_expiry().map(|d| d.as_secs()),
                    "request_count": conn.request_count,
                    "is_expired": conn.is_expired()
                })
            })
            .collect();

        let metrics = serde_json::json!({
            "ttl_stats": {
                "active_connections": ttl_stats.active_connections,
                "total_connections": ttl_stats.total_connections,
                "expired_connections": ttl_stats.expired_connections,
                "average_ttl_seconds": ttl_stats.average_ttl_secs
            },
            "active_connections": detailed_connections,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        });

        let response = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .header("Cache-Control", "no-cache")
            .body(Full::new(Bytes::from(metrics.to_string())))
            .unwrap();

        Ok(response)
    }

    async fn handle_not_found(&self, path: &str) -> Result<Response<Full<Bytes>>, hyper::Error> {
        warn!("404 Not Found: {}", path);

        let error_response = serde_json::json!({
            "error": "Not Found",
            "message": format!("The requested path '{}' was not found on this server", path),
            "status": 404,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        });

        let response = Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(error_response.to_string())))
            .unwrap();

        Ok(response)
    }
}
