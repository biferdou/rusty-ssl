use rusty_ssl::AppConfig;

#[test]
fn test_config_loading() {
    let config = AppConfig::default();
    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.server.port, 8443);
    assert_eq!(config.ttl.default_ttl_secs, 300);
    assert_eq!(config.ssl.cert_check_interval_secs, 3600);
}

#[test]
fn test_server_addr() {
    let config = AppConfig::default();
    let addr = config.server_addr().unwrap();
    assert_eq!(addr.to_string(), "0.0.0.0:8443");
}

#[test]
fn test_durations() {
    let config = AppConfig::default();
    assert_eq!(config.default_ttl().as_secs(), 300);
    assert_eq!(config.max_ttl().as_secs(), 3600);
    assert_eq!(config.request_timeout().as_secs(), 30);
}
