# DoggyGallery automation recipes
# Install just: https://github.com/casey/just

# List available recipes
default:
    @just --list

# Build for production
build:
    RUSTFLAGS="-D warnings" cargo build --release

# Run tests
test:
    cargo test

# Check code without building
check:
    cargo check
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Clean build artifacts
clean:
    cargo clean

# Run in watch mode (requires cargo-watch)
watch:
    cargo watch -x run

# Generate self-signed certificates for development
gen-certs:
    @echo "Generating self-signed certificates for development..."
    @mkdir -p certs
    openssl req -x509 -newkey rsa:4096 -nodes \
        -keyout certs/key.pem \
        -out certs/cert.pem \
        -days 365 \
        -subj "/C=US/ST=State/L=City/O=DoggyGallery/CN=localhost"
    @echo "Certificates generated in certs/"
    @echo "  cert.pem - Certificate"
    @echo "  key.pem  - Private key"

# Run the development server with default settings
dev: gen-certs
    @echo "Starting DoggyGallery in development mode..."
    @mkdir -p media
    RUST_LOG=doggygallery=debug,tower_http=debug cargo run -- \
        --cert certs/cert.pem \
        --key certs/key.pem \
        --media-dir ./media \
        --username admin \
        --password admin \
        --host 127.0.0.1 \
        --port 8443

# Run the development server with custom media directory
dev-custom MEDIA_DIR:
    @echo "Starting DoggyGallery with custom media directory..."
    RUST_LOG=doggygallery=debug,tower_http=debug cargo run -- \
        --cert certs/cert.pem \
        --key certs/key.pem \
        --media-dir {{MEDIA_DIR}} \
        --username admin \
        --password admin \
        --host 127.0.0.1 \
        --port 8443

# Run production build
run-release: build
    ./target/release/doggygallery \
        --cert certs/cert.pem \
        --key certs/key.pem \
        --media-dir ./media \
        --username admin \
        --password changeme \
        --host 0.0.0.0 \
        --port 8443

# Security audit (Rust dependencies)
audit:
    cargo audit

# Trivy security scan - filesystem
trivy-fs:
    @echo "Scanning filesystem for vulnerabilities..."
    trivy fs --scanners vuln,secret,misconfig .

# Trivy security scan - comprehensive
trivy-all:
    @echo "Running comprehensive security scan..."
    @echo "\n=== Scanning Rust dependencies ==="
    trivy fs --scanners vuln --skip-dirs target --security-checks vuln .
    @echo "\n=== Scanning for secrets ==="
    trivy fs --scanners secret .
    @echo "\n=== Scanning for misconfigurations ==="
    trivy fs --scanners misconfig .

# Install development dependencies
install-deps:
    cargo install cargo-watch
    cargo install cargo-audit
    @echo "Development dependencies installed!"

# Example: Run with environment variables
run-env:
    @echo "Example of running with environment variables..."
    @echo "DOGGYGALLERY_USERNAME=user DOGGYGALLERY_PASSWORD=pass cargo run -- --media-dir ./media"
