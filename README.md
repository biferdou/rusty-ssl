# 🦀 Rusty-SSL: A Secure, High-Performance HTTPS Server

Rusty-SSL is a secure, high-performance HTTPS server built in Rust that combines modern SSL/TLS capabilities with intelligent connection management. Designed for production environments where security, performance, and reliability are paramount.

## 🌟 Features

### 🔒 Security First

- **TLS 1.3 Support**: Latest encryption standards with secure cipher suites
- **Let's Encrypt Integration**: Automatic certificate loading and monitoring
- **Memory Safety**: Built with Rust's memory-safe guarantees
- **No Client Certificates Required**: Simplified deployment model
- **Secure Defaults**: Production-ready security configuration out of the box

### ⚡ High Performance

- **Async I/O**: Non-blocking request handling with Tokio runtime
- **Hyper HTTP Engine**: Industry-leading HTTP/1.1 and HTTP/2 support
- **Connection Pooling**: Efficient resource management
- **Low Latency**: Optimized for sub-100ms response times
- **High Throughput**: Handles thousands of concurrent connections

### 🧠 Intelligent TTL Management

- **IP-Based Connection Tracking**: Individual TTL per client IP address
- **Adaptive TTL Extension**: High-activity connections get extended lifetimes
- **Automatic Cleanup**: Background tasks remove expired connections
- **Connection Metrics**: Real-time statistics and monitoring
- **Rate Limiting**: Natural rate limiting through TTL expiration

### 📊 Monitoring & Observability

- **Health Check Endpoints**: Kubernetes-ready liveness and readiness probes
- **SSL Certificate Status**: Real-time certificate validity monitoring
- **Connection Metrics**: Detailed statistics on active and expired connections
- **Structured Logging**: JSON and pretty-print logging formats
- **Automatic Certificate Monitoring**: Proactive expiration warnings

### 🔧 Production Ready

- **Systemd Integration**: Native Linux service support
- **Graceful Shutdown**: Proper connection cleanup on termination
- **Configuration Management**: Layered config with environment variable support
- **Error Handling**: Comprehensive error reporting and recovery
- **Resource Monitoring**: Built-in performance metrics

## 🏗️ Architecture

### 1. **HTTP Router**

- Route-based request handling
- RESTful API endpoint management
- Request/response lifecycle management
- Error handling and status codes

### 2. **SSL Manager**

- Certificate loading and validation
- TLS handshake management
- Automatic certificate monitoring
- Expiration alerting system

### 3. **TTL Controller**

- Per-IP connection tracking
- Adaptive lifetime management
- Background cleanup processes
- Connection statistics collection

### 4. **Configuration System**

- Layered configuration loading
- Environment variable overrides
- Runtime configuration validation
- Development/production profiles

### Data Flow

1. **Client Request** → TLS Handshake → HTTP Request Processing
2. **Router** → Route Matching → Handler Execution
3. **TTL Controller** → IP Registration → Activity Tracking
4. **Response Generation** → TTL Update → Client Response
5. **Background Tasks** → Certificate Monitoring → Connection Cleanup

## 🚀 Future Features & Roadmap

### Version 0.2.0 - Enhanced Security

- [ ] **mTLS Support**: Mutual TLS authentication
- [ ] **OCSP Stapling**: Real-time certificate validation
- [ ] **Certificate Pinning**: Enhanced security for known clients
- [ ] **IP Whitelisting/Blacklisting**: Advanced access control
- [ ] **Request Rate Limiting**: Per-IP rate limiting beyond TTL

### Version 0.3.0 - Performance & Scalability

- [ ] **HTTP/2 & HTTP/3**: Next-generation protocol support
- [ ] **Connection Pooling**: Advanced connection management
- [ ] **Load Balancing**: Built-in load balancing capabilities
- [ ] **Caching Layer**: Response caching for static content
- [ ] **Compression**: Gzip/Brotli response compression

### Version 0.4.0 - Observability & DevOps

- [ ] **Prometheus Metrics**: Native metrics export
- [ ] **Distributed Tracing**: OpenTelemetry integration
- [ ] **Health Check Framework**: Advanced health monitoring
- [ ] **Configuration Hot-Reload**: Runtime configuration updates
- [ ] **Admin API**: Runtime management endpoints

### Version 0.5.0 - Cloud Native

- [ ] **Docker Support**: Official container images
- [ ] **Kubernetes Operators**: Native K8s integration
- [ ] **Service Mesh**: Istio/Envoy compatibility
- [ ] **Cloud Provider Integration**: AWS/GCP/Azure support
- [ ] **Auto-scaling**: Dynamic resource management

### Long-term Vision

- **Plugin System**: Extensible middleware architecture
- **WebAssembly Support**: WASM-based request processing
- **Multi-protocol**: Support for additional protocols
- **AI-powered Security**: ML-based threat detection
- **Global Load Balancing**: Multi-region deployment

## 🧪 Testing Deployment

### Prerequisites

- **Rust 1.70+**
- **OpenSSL** (for certificate generation)
- **curl** (for testing)

### Quick Start

1. **Clone and Build**

   ```bash
   git clone https://github.com/biferdou/rusty-ssl.git
   cd rusty-ssl
   cargo build --release
   ```

2. **Generate Test Certificates**

   ```bash
   mkdir test-certs
   openssl genrsa -out test-certs/key.pem 2048
   openssl req -new -x509 -key test-certs/key.pem -out test-certs/cert.pem -days 365 \
     -subj "/C=US/ST=Test/L=Test/O=Test/CN=localhost"
   ```

3. **Create Test Configuration**

   ```bash
   mkdir -p configs
   cat > configs/test.toml << 'EOF'
   [server]
   host = "127.0.0.1"
   port = 8443
   max_connections = 100
   request_timeout_secs = 30

   [ssl]
   cert_path = "test-certs/cert.pem"
   key_path = "test-certs/key.pem"
   cert_check_interval_secs = 3600

   [ttl]
   default_ttl_secs = 60
   max_ttl_secs = 300
   cleanup_interval_secs = 10

   [logging]
   level = "debug"
   format = "pretty"
   EOF
   ```

4. **Start the Server**

   ```bash
   RUSTY_SSL_CONFIG_PATH=configs/test.toml ./target/release/rusty-ssl
   ```

5. **Test Endpoints**

   ```bash
   # Health check
   curl -k https://localhost:8443/health

   # SSL status
   curl -k https://localhost:8443/ssl-status

   # Connection metrics
   curl -k https://localhost:8443/metrics

   # Homepage
   curl -k https://localhost:8443/
   ```

### Testing Scenarios

#### Basic Functionality

```bash
# Test all endpoints
for endpoint in health health/ready health/live ssl-status metrics; do
  echo "Testing /$endpoint"
  curl -k -s https://localhost:8443/$endpoint | jq '.status' || echo "Failed"
done
```

#### TTL Management

```bash
# Generate multiple requests to test TTL
for i in {1..10}; do
  curl -k -s https://localhost:8443/health > /dev/null
  sleep 1
done

# Check connection metrics
curl -k -s https://localhost:8443/metrics | jq '.ttl_stats'
```

#### Load Testing

```bash
# Install siege if not available
# sudo apt-get install siege

# Basic load test
siege -c 10 -t 30s -k https://localhost:8443/health
```

## 🏭 Production Deployment

### 1. Server Preparation

#### System Requirements

- **Ubuntu 20.04+ / CentOS 8+ / RHEL 8+**
- **RAM**: Minimum 512MB, Recommended 2GB+
- **CPU**: 1+ cores
- **Storage**: 100MB for binary + logs
- **Network**: Ports 80, 443 accessible

#### User Setup

```bash
# Create dedicated user
sudo useradd -r -s /bin/false -d /opt/rusty-ssl rusty-ssl

# Create directories
sudo mkdir -p /opt/rusty-ssl
sudo mkdir -p /var/log/rusty-ssl
sudo mkdir -p /etc/rusty-ssl

# Set ownership
sudo chown -R rusty-ssl:rusty-ssl /opt/rusty-ssl /var/log/rusty-ssl
```

### 2. SSL Certificate Setup

#### Let's Encrypt (Recommended)

```bash
# Install certbot
sudo apt-get install certbot

# Obtain certificate
sudo certbot certonly --standalone -d yourdomain.com

# Set certificate permissions
sudo chown -R rusty-ssl:rusty-ssl /etc/letsencrypt/
sudo chmod -R 755 /etc/letsencrypt/
```

#### Manual Certificate

```bash
# Place certificates in secure location
sudo cp your-cert.pem /etc/rusty-ssl/cert.pem
sudo cp your-key.pem /etc/rusty-ssl/key.pem

# Secure permissions
sudo chown rusty-ssl:rusty-ssl /etc/rusty-ssl/*.pem
sudo chmod 644 /etc/rusty-ssl/cert.pem
sudo chmod 600 /etc/rusty-ssl/key.pem
```

### 3. Configuration

#### Production Configuration

```bash
sudo cat > /etc/rusty-ssl/production.toml << 'EOF'
[server]
host = "0.0.0.0"
port = 443
max_connections = 5000
request_timeout_secs = 60

[ssl]
cert_path = "/etc/letsencrypt/live/yourdomain.com/fullchain.pem"
key_path = "/etc/letsencrypt/live/yourdomain.com/privkey.pem"
cert_check_interval_secs = 1800  # Check every 30 minutes

[ttl]
default_ttl_secs = 600      # 10 minutes
max_ttl_secs = 7200         # 2 hours
cleanup_interval_secs = 30  # Cleanup every 30 seconds

[logging]
level = "info"
format = "json"
EOF

sudo chown rusty-ssl:rusty-ssl /etc/rusty-ssl/production.toml
```

### 4. Binary Deployment

```bash
# Copy binary to production location
sudo cp target/release/rusty-ssl /opt/rusty-ssl/
sudo chown rusty-ssl:rusty-ssl /opt/rusty-ssl/rusty-ssl
sudo chmod +x /opt/rusty-ssl/rusty-ssl

# Test binary
sudo -u rusty-ssl /opt/rusty-ssl/rusty-ssl --version
```

### 5. Systemd Service Setup

#### Service File

```bash
sudo cat > /etc/systemd/system/rusty-ssl.service << 'EOF'
[Unit]
Description=Rusty-SSL Secure HTTP Server
After=network.target
StartLimitIntervalSec=0

[Service]
Type=simple
Restart=always
RestartSec=1
User=rusty-ssl
Group=rusty-ssl
WorkingDirectory=/opt/rusty-ssl

# Configuration
Environment=RUSTY_SSL_CONFIG_PATH=/etc/rusty-ssl/production.toml
Environment=RUSTY_SSL_LOGGING__FORMAT=json

# Binary location
ExecStart=/opt/rusty-ssl/rusty-ssl

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectHome=true
ProtectSystem=strict
ReadWritePaths=/var/log/rusty-ssl

# Required for binding to port 443
AmbientCapabilities=CAP_NET_BIND_SERVICE
CapabilityBoundingSet=CAP_NET_BIND_SERVICE

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
EOF
```

#### Service Management

```bash
# Reload systemd and enable service
sudo systemctl daemon-reload
sudo systemctl enable rusty-ssl

# Start the service
sudo systemctl start rusty-ssl

# Check status
sudo systemctl status rusty-ssl

# View logs
sudo journalctl -u rusty-ssl -f
```

### 6. Firewall Configuration

```bash
# UFW (Ubuntu)
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# firewalld (CentOS/RHEL)
sudo firewall-cmd --permanent --add-port=80/tcp
sudo firewall-cmd --permanent --add-port=443/tcp
sudo firewall-cmd --reload

# iptables (manual)
sudo iptables -A INPUT -p tcp --dport 80 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 443 -j ACCEPT
```

### 7. Monitoring Setup

#### Log Rotation

```bash
sudo cat > /etc/logrotate.d/rusty-ssl << 'EOF'
/var/log/rusty-ssl/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    copytruncate
    postrotate
        systemctl reload rusty-ssl
    endscript
}
EOF
```

#### Health Monitoring Script

```bash
sudo cat > /opt/rusty-ssl/health-check.sh << 'EOF'
#!/bin/bash
DOMAIN="yourdomain.com"
HEALTH_URL="https://$DOMAIN/health"

if curl -f -s "$HEALTH_URL" > /dev/null; then
    echo "$(date): Health check passed"
    exit 0
else
    echo "$(date): Health check failed"
    # Optional: restart service
    # systemctl restart rusty-ssl
    exit 1
fi
EOF

sudo chmod +x /opt/rusty-ssl/health-check.sh

# Add to crontab for regular checks
echo "*/5 * * * * /opt/rusty-ssl/health-check.sh >> /var/log/rusty-ssl/health.log 2>&1" | sudo crontab -u rusty-ssl -
```

## 📊 Usage Examples

### Health Monitoring

```bash
# Basic health check
curl https://yourdomain.com/health

# Kubernetes readiness probe
curl https://yourdomain.com/health/ready

# Kubernetes liveness probe
curl https://yourdomain.com/health/live
```

### SSL Certificate Monitoring

```bash
# Check certificate status
curl https://yourdomain.com/ssl-status | jq '.certificate'

# Monitor certificate expiration
curl -s https://yourdomain.com/ssl-status | jq '.certificate.days_until_expiry'
```

### Connection Metrics

```bash
# Get current connection statistics
curl https://yourdomain.com/metrics | jq '.ttl_stats'

# Monitor active connections
curl https://yourdomain.com/metrics | jq '.active_connections | length'

# Check connection details
curl https://yourdomain.com/metrics | jq '.active_connections[0]'
```

### Performance Testing

```bash
# Response time testing
curl -w "@curl-format.txt" -s -o /dev/null https://yourdomain.com/health

# Where curl-format.txt contains:
# time_total: %{time_total}s
# time_connect: %{time_connect}s
# time_appconnect: %{time_appconnect}s
```

## ⚙️ Configuration Reference

### Server Configuration

```toml
[server]
host = "0.0.0.0"              # Listen address
port = 443                    # Listen port
max_connections = 5000        # Maximum concurrent connections
request_timeout_secs = 60     # Request timeout in seconds
```

### SSL Configuration

```toml
[ssl]
cert_path = "/path/to/cert.pem"           # Certificate file path
key_path = "/path/to/key.pem"             # Private key file path
cert_check_interval_secs = 1800           # Certificate monitoring interval
```

### TTL Configuration

```toml
[ttl]
default_ttl_secs = 600        # Default connection TTL (10 minutes)
max_ttl_secs = 7200          # Maximum TTL (2 hours)
cleanup_interval_secs = 30    # Cleanup task interval
```

### Logging Configuration

```toml
[logging]
level = "info"                # Log level: error, warn, info, debug, trace
format = "json"               # Format: json, pretty
```

### Environment Variable Overrides

```bash
# Override any configuration value
RUSTY_SSL_SERVER__PORT=8443
RUSTY_SSL_SSL__CERT_PATH=/custom/path/cert.pem
RUSTY_SSL_LOGGING__LEVEL=debug
RUSTY_SSL_LOGGING__FORMAT=pretty
```

## 🔧 Maintenance & Operations

### Regular Tasks

#### Certificate Renewal

```bash
# Check certificate expiration
curl -s https://yourdomain.com/ssl-status | jq '.certificate.days_until_expiry'

# Renew Let's Encrypt certificate
sudo certbot renew --quiet

# Restart service if certificate renewed
sudo systemctl restart rusty-ssl
```

#### Log Management

```bash
# View real-time logs
sudo journalctl -u rusty-ssl -f

# Check for errors
sudo journalctl -u rusty-ssl --since="1 hour ago" | grep ERROR

# Analyze connection patterns
sudo journalctl -u rusty-ssl --since="1 day ago" | grep "Request:"
```

#### Performance Monitoring

```bash
# Monitor resource usage
ps aux | grep rusty-ssl
top -p $(pgrep rusty-ssl)

# Check network connections
ss -tlnp | grep :443
netstat -tlnp | grep :443

# Monitor file descriptors
lsof -p $(pgrep rusty-ssl) | wc -l
```

### Troubleshooting

#### Common Issues

1. **Certificate Loading Failures**

   ```bash
   # Check certificate validity
   openssl x509 -in /path/to/cert.pem -text -noout
   
   # Verify certificate-key match
   openssl x509 -noout -modulus -in cert.pem | openssl md5
   openssl rsa -noout -modulus -in key.pem | openssl md5
   ```

2. **Port Binding Issues**

   ```bash
   # Check if port is in use
   sudo lsof -i :443
   
   # Verify user has NET_BIND_SERVICE capability
   sudo getcap /opt/rusty-ssl/rusty-ssl
   ```

3. **Performance Issues**

   ```bash
   # Check system limits
   ulimit -n
   cat /proc/sys/fs/file-max
   
   # Monitor system resources
   iostat -x 1
   vmstat 1
   ```

#### Debug Mode

```bash
# Run with debug logging
sudo systemctl stop rusty-ssl
sudo -u rusty-ssl RUSTY_SSL_LOGGING__LEVEL=debug /opt/rusty-ssl/rusty-ssl
```

### Backup & Recovery

#### Configuration Backup

```bash
# Backup configuration
sudo tar -czf rusty-ssl-config-$(date +%Y%m%d).tar.gz /etc/rusty-ssl/

# Backup certificates
sudo tar -czf rusty-ssl-certs-$(date +%Y%m%d).tar.gz /etc/letsencrypt/
```

#### Service Recovery

```bash
# Quick service restart
sudo systemctl restart rusty-ssl

# Full service reset
sudo systemctl stop rusty-ssl
sudo systemctl reset-failed rusty-ssl
sudo systemctl start rusty-ssl
```

## 🤝 Contributing

### Development Setup

```bash
# Clone repository
git clone https://github.com/yourusername/rusty-ssl.git
cd rusty-ssl

# Install development dependencies
cargo install cargo-watch cargo-audit

# Run tests
cargo test

# Run with file watching
cargo watch -x run
```

### Code Standards

- **Rust Edition**: 2021
- **MSRV**: 1.70.0
- **Formatting**: `cargo fmt`
- **Linting**: `cargo clippy`
- **Security**: `cargo audit`

### Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Write comprehensive tests
4. Update documentation
5. Run all checks (`cargo test`, `cargo clippy`, `cargo fmt`)
6. Submit pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
