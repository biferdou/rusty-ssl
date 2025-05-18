use std::sync::Arc;

use anyhow::Result;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use rusty_ssl::{AppConfig, Router, SslManager, TtlController, init_logging};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_rustls::TlsAcceptor;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = AppConfig::load().map_err(|e| {
        eprintln!("Failed to load configuration: {}", e);
        std::process::exit(1);
    })?;

    // Initialize logging
    init_logging(&config.logging)?;
    info!("Starting Rusty-SSL server v{}", env!("CARGO_PKG_VERSION"));

    // Initialize SSL manager
    let ssl_manager = SslManager::new(
        &config.ssl.cert_path,
        &config.ssl.key_path,
        config.cert_check_interval(),
    )
    .map_err(|e| {
        error!("Failed to initialize SSL manager: {}", e);
        std::process::exit(1);
    })?;

    let tls_config = ssl_manager.get_config();
    let acceptor = TlsAcceptor::from(tls_config);

    // Initialize TTL controller
    let ttl_controller = Arc::new(Mutex::new(TtlController::new(
        config.default_ttl(),
        config.max_ttl(),
        config.cleanup_interval(),
    )));

    // Initialize router
    let router = Arc::new(Router::new(ttl_controller.clone()));

    // Bind to address
    let addr = config.server_addr()?;
    let listener = TcpListener::bind(&addr).await?;
    info!("Server listening on https://{}", addr);

    // Start background tasks
    let ssl_task = {
        let mut ssl_manager_clone = ssl_manager;
        tokio::spawn(async move {
            ssl_manager_clone.start_certificate_monitoring().await;
        })
    };

    let ttl_task = {
        let ttl_controller_clone = ttl_controller.clone();
        tokio::spawn(async move {
            let mut ttl_controller = ttl_controller_clone.lock().await;
            ttl_controller.start_cleanup_task().await;
        })
    };

    // Server loop
    let server_task = tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, remote_addr)) => {
                    let acceptor = acceptor.clone();
                    let router = router.clone();

                    tokio::spawn(async move {
                        let client_ip = remote_addr.ip();

                        // Handle TLS handshake
                        let tls_stream = match acceptor.accept(stream).await {
                            Ok(tls_stream) => tls_stream,
                            Err(e) => {
                                warn!("TLS handshake failed for {}: {}", client_ip, e);
                                return;
                            }
                        };

                        let io = TokioIo::new(tls_stream);

                        // Handle HTTP requests
                        if let Err(e) = http1::Builder::new()
                            .serve_connection(
                                io,
                                service_fn(move |req| {
                                    let router = router.clone();
                                    async move { router.route(req, client_ip).await }
                                }),
                            )
                            .await
                        {
                            warn!("HTTP connection error for {}: {}", client_ip, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    });

    // Setup graceful shutdown
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("Shutdown signal received");
    };

    // Wait for either server task completion or shutdown signal
    tokio::select! {
        _ = server_task => {
            info!("Server task completed");
        }
        _ = shutdown_signal => {
            info!("Shutting down gracefully...");
        }
    }

    // Cancel background tasks
    ssl_task.abort();
    ttl_task.abort();

    info!("Server shutdown complete");
    Ok(())
}
