# DoggyGallery 🐕

A secure, fast, and beautiful media gallery server built with Rust. Serve your images and videos over HTTPS with HTTP Basic Authentication.

## Features

- **HTTPS Only**: Forces TLS 1.3 for secure connections
- **Authentication**: HTTP Basic Authentication with rate limiting (10 attempts per minute)
- **Media Support**: Serves images, videos, AND audio files (MP3, FLAC, WAV, etc.)
- **Beautiful UI**: Modern, responsive gallery interface with lightbox viewer
- **Dark Mode**: Automatic light/dark theme based on system preferences
- **Filter & Search**: Advanced filtering by type, extension, and fuzzy name matching
- **Directory Browsing**: Navigate through subdirectories seamlessly
- **OpenAPI/Swagger**: Interactive API documentation at `/docs`
- **Self-Signed Certs**: Optional on-the-fly certificate generation for development
- **Compression**: Automatic gzip/brotli compression for faster loading
- **Security**: MIME validation, path traversal protection, security headers, SVG sandboxing

## Quick Start

### Prerequisites

- Rust 1.70 or later
- OpenSSL (for certificate generation)
- [just](https://github.com/casey/just) command runner (optional but recommended)

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd DoggyGallery

# Build the project
cargo build --release

# Or use just
just build
```

### Development

The easiest way to run DoggyGallery in development mode is with the provided `just` recipes:

```bash
# Generate self-signed certificates and run with default settings
just dev

# Run with a custom media directory
just dev-custom /path/to/your/media
```

This will:
- Generate self-signed certificates in `certs/`
- Create a `media/` directory if it doesn't exist
- Start the server on `https://127.0.0.1:7833` (port 7833 = "RUFF" on phone keypad! 🐕)
- Use default credentials: `admin`/`admin`

### Production

#### Option 1: Using provided certificates

```bash
./target/release/doggygallery \
  --cert /path/to/cert.pem \
  --key /path/to/key.pem \
  --media-dir /path/to/media \
  --username your_username \
  --password your_secure_password \
  --host 0.0.0.0 \
  --port 7833
```

#### Option 2: Using on-the-fly self-signed certificates (development only)

```bash
./target/release/doggygallery \
  --self-signed-certs-on-the-fly \
  --media-dir /path/to/media \
  --username your_username \
  --password your_password \
  --host 127.0.0.1 \
  --port 7833
```

**Warning**: Self-signed certificates are NOT suitable for production! Use proper certificates from a trusted CA like Let's Encrypt.

## Configuration

### Command Line Arguments

```
Options:
  --cert <PATH>                    Path to TLS certificate file
  --key <PATH>                     Path to TLS private key file
  --self-signed-certs-on-the-fly   Generate self-signed certificates on-the-fly
  --media-dir <PATH>               Directory containing media files to serve
  --username <USERNAME>            Username for HTTP Basic Authentication
  --password <PASSWORD>            Password for HTTP Basic Authentication
  --host <HOST>                    Host to bind to [default: 0.0.0.0]
  --port <PORT>                    Port to listen on [default: 7833]
  -h, --help                       Print help
```

### Environment Variables

All options can also be set via environment variables with the `DOGGYGALLERY_` prefix:

```bash
export DOGGYGALLERY_CERT=/path/to/cert.pem
export DOGGYGALLERY_KEY=/path/to/key.pem
export DOGGYGALLERY_MEDIA_DIR=/path/to/media
export DOGGYGALLERY_USERNAME=admin
export DOGGYGALLERY_PASSWORD=secure_password
export DOGGYGALLERY_HOST=0.0.0.0
export DOGGYGALLERY_PORT=7833
export DOGGYGALLERY_SELF_SIGNED=true

./target/release/doggygallery
```

## Generating Certificates

### Self-Signed Certificates (Development)

```bash
# Using just
just gen-certs

# Or manually with OpenSSL
mkdir -p certs
openssl req -x509 -newkey rsa:4096 -nodes \
  -keyout certs/key.pem \
  -out certs/cert.pem \
  -days 365 \
  -subj "/C=US/ST=State/L=City/O=DoggyGallery/CN=localhost"
```

### Production Certificates (Let's Encrypt)

For production, use Let's Encrypt with Certbot:

```bash
# Install certbot
sudo apt-get install certbot

# Generate certificate
sudo certbot certonly --standalone -d yourdomain.com

# Certificates will be in:
# /etc/letsencrypt/live/yourdomain.com/fullchain.pem
# /etc/letsencrypt/live/yourdomain.com/privkey.pem
```

## Just Commands

```bash
just                    # List all available commands
just build              # Build for production
just dev                # Run development server
just dev-custom <DIR>   # Run with custom media directory
just test               # Run tests
just check              # Run linter and type checks
just fmt                # Format code
just gen-certs          # Generate self-signed certificates
just audit              # Security audit of dependencies
just trivy-fs           # Scan for vulnerabilities
```

## Security Considerations

1. **TLS 1.3 Only**: The server only accepts TLS 1.3 connections for maximum security
2. **Authentication Required**: All routes require HTTP Basic Authentication
3. **Path Traversal Protection**: Directory traversal attacks are prevented via path canonicalization
4. **Hidden Files**: Files starting with `.` are not served
5. **Media Files Only**: Only image and video files are served
6. **Self-Signed Certificates**: The `--self-signed-certs-on-the-fly` option is for development only

## Media Support

### Supported Image Formats
- JPEG (.jpg, .jpeg)
- PNG (.png)
- GIF (.gif)
- WebP (.webp)
- BMP (.bmp)
- SVG (.svg)

### Supported Video Formats
- MP4 (.mp4)
- WebM (.webm)
- Matroska (.mkv)
- AVI (.avi)
- QuickTime (.mov)
- Flash Video (.flv)
- Windows Media Video (.wmv)

## Architecture

DoggyGallery is built with modern Rust technologies:

- **[Axum](https://github.com/tokio-rs/axum)**: Web framework
- **[Tokio](https://tokio.rs/)**: Async runtime
- **[Rustls](https://github.com/rustls/rustls)**: TLS implementation (enforcing TLS 1.3)
- **[Askama](https://github.com/djc/askama)**: Template engine
- **[Tower](https://github.com/tower-rs/tower)**: Middleware
- **[Clap](https://github.com/clap-rs/clap)**: CLI argument parsing

## Project Structure

```
DoggyGallery/
├── src/
│   ├── main.rs        # Application entry point
│   ├── config.rs      # Configuration and CLI parsing
│   ├── auth.rs        # Authentication middleware
│   ├── handlers.rs    # HTTP request handlers
│   ├── models.rs      # Data models
│   ├── templates.rs   # Template definitions
│   └── tls.rs         # TLS configuration
├── templates/
│   └── gallery.html   # Gallery UI template
├── Cargo.toml         # Dependencies
├── justfile           # Build automation
└── README.md          # This file
```

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
