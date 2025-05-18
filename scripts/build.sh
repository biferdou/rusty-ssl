#!/bin/bash

echo "🦀 Building Rusty-SSL..."

# Navigate to project directory
if [ ! -d "rusty-ssl" ]; then
    echo "❌ Project directory 'rusty-ssl' not found!"
    echo "Please run this script from the parent directory."
    exit 1
fi

cd rusty-ssl

# Check for SSL certificates
if [ ! -f "/etc/letsencrypt/live/tilas.xyz/fullchain.pem" ]; then
    echo "⚠️  Warning: SSL certificate not found at expected location"
    echo "   Expected: /etc/letsencrypt/live/tilas.xyz/fullchain.pem"
    echo "   Make sure Let's Encrypt certificates are properly installed"
fi

# Clean previous builds
echo "Cleaning previous builds..."
cargo clean

# Build in debug mode first
echo "Building in debug mode..."
if cargo build; then
    echo "✅ Debug build successful!"
else
    echo "❌ Debug build failed!"
    exit 1
fi

# Run quick tests
echo "Running tests..."
if cargo test; then
    echo "✅ Tests passed!"
else
    echo "⚠️  Some tests failed, but continuing..."
fi

# Build in release mode
echo "Building in release mode..."
if cargo build --release; then
    echo "✅ Release build successful!"
else
    echo "❌ Release build failed!"
    exit 1
fi

echo ""
echo "🎉 Build complete!"
echo ""
echo "Binary location: target/release/rusty-ssl"
echo "Size: $(du -h target/release/rusty-ssl | cut -f1)"
echo ""
echo "Next steps:"
echo ""
echo "1. Test the configuration:"
echo "   ./target/release/rusty-ssl --version"
echo ""
echo "2. Run in development mode (port 8443):"
echo "   ./target/release/rusty-ssl"
echo ""
echo "3. Test endpoints:"
echo "   curl -k https://tilas.xyz:8443/health"
echo "   curl -k https://tilas.xyz:8443/ssl-status"
echo "   curl -k https://tilas.xyz:8443/metrics"
echo "   curl -k https://tilas.xyz:8443/"
echo ""
echo "4. For production deployment (requires sudo):"
echo "   sudo RUSTY_SSL_SERVER__PORT=443 ./target/release/rusty-ssl"
echo ""
echo "5. Install as a systemd service:"
echo "   sudo cp scripts/rusty-ssl.service /etc/systemd/system/"
echo "   sudo systemctl daemon-reload"
echo "   sudo systemctl enable rusty-ssl"
echo "   sudo systemctl start rusty-ssl"