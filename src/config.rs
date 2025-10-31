use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(name = "doggygallery")]
#[command(version)]
#[command(about = "🐕 A secure media gallery server with TLS 1.3 and HTTP Basic Auth")]
#[command(long_about = "\
DoggyGallery - A Beautiful, Secure Media Gallery Server

Serve your images, videos, and audio files over HTTPS with authentication.
Features automatic light/dark mode, advanced filtering, and OpenAPI docs.

EXAMPLES:
    # Development with self-signed certificates
    doggygallery --self-signed-certs-on-the-fly \\
        --media-dir ./media \\
        --username admin \\
        --password secret

    # Production with Let's Encrypt certificates
    doggygallery --cert /etc/letsencrypt/live/example.com/fullchain.pem \\
        --key /etc/letsencrypt/live/example.com/privkey.pem \\
        --media-dir /var/media \\
        --username admin \\
        --password secure_password \\
        --host 0.0.0.0 \\
        --port 7833

    # Using environment variables
    export DOGGYGALLERY_MEDIA_DIR=/path/to/media
    export DOGGYGALLERY_USERNAME=admin
    export DOGGYGALLERY_PASSWORD=secret
    export DOGGYGALLERY_SELF_SIGNED=true
    doggygallery

For more information: https://github.com/meltingscales/DoggyGallery
")]
pub struct Config {
    /// Path to TLS certificate file (PEM format)
    ///
    /// Required for production. For development, use --self-signed-certs-on-the-fly instead.
    /// Example: /etc/letsencrypt/live/example.com/fullchain.pem
    #[arg(long, env = "DOGGYGALLERY_CERT", value_name = "CERT_PATH")]
    pub cert: Option<PathBuf>,

    /// Path to TLS private key file (PEM format)
    ///
    /// Required for production. For development, use --self-signed-certs-on-the-fly instead.
    /// Example: /etc/letsencrypt/live/example.com/privkey.pem
    #[arg(long, env = "DOGGYGALLERY_KEY", value_name = "KEY_PATH")]
    pub key: Option<PathBuf>,

    /// Generate self-signed certificates on-the-fly
    ///
    /// WARNING: For development only! Not suitable for production use.
    /// Self-signed certificates will trigger browser warnings.
    #[arg(long, env = "DOGGYGALLERY_SELF_SIGNED")]
    pub self_signed_certs_on_the_fly: bool,

    /// Directory containing media files to serve (images, videos, audio)
    ///
    /// The server will recursively serve all supported media files from this directory.
    /// Supported formats: JPG, PNG, GIF, WebP, MP4, WebM, MKV, MP3, FLAC, WAV, and more.
    #[arg(long, env = "DOGGYGALLERY_MEDIA_DIR", value_name = "DIR")]
    pub media_dir: PathBuf,

    /// Username for HTTP Basic Authentication
    ///
    /// All requests must provide this username. Choose a strong username.
    #[arg(long, env = "DOGGYGALLERY_USERNAME", value_name = "USERNAME")]
    pub username: String,

    /// Password for HTTP Basic Authentication
    ///
    /// All requests must provide this password. Use a strong, randomly generated password.
    /// Consider using a password manager to generate secure passwords.
    #[arg(long, env = "DOGGYGALLERY_PASSWORD", value_name = "PASSWORD")]
    pub password: String,

    /// Host/IP address to bind to
    ///
    /// Use 0.0.0.0 to listen on all interfaces (public access).
    /// Use 127.0.0.1 to listen only on localhost (local access only).
    #[arg(long, default_value = "0.0.0.0", env = "DOGGYGALLERY_HOST", value_name = "HOST")]
    pub host: String,

    /// Port to listen on (default: 7833 - "RUFF" on phone keypad! 🐕)
    ///
    /// Standard HTTPS port is 443, but requires root privileges.
    /// Port 7833 is memorable (spells RUFF) and doesn't require root.
    #[arg(long, default_value = "7833", env = "DOGGYGALLERY_PORT", value_name = "PORT")]
    pub port: u16,
}

impl Config {
    pub fn validate(&self) -> anyhow::Result<()> {
        // Validate certificate configuration
        if !self.self_signed_certs_on_the_fly {
            // If not using self-signed on-the-fly, both cert and key must be provided
            match (&self.cert, &self.key) {
                (Some(cert), Some(key)) => {
                    if !cert.exists() {
                        anyhow::bail!("Certificate file does not exist: {:?}", cert);
                    }
                    if !key.exists() {
                        anyhow::bail!("Private key file does not exist: {:?}", key);
                    }
                }
                _ => {
                    anyhow::bail!(
                        "Either provide --cert and --key, or use --self-signed-certs-on-the-fly"
                    );
                }
            }
        } else if self.cert.is_some() || self.key.is_some() {
            tracing::warn!(
                "--self-signed-certs-on-the-fly is set, ignoring --cert and --key arguments"
            );
        }

        if !self.media_dir.exists() {
            anyhow::bail!("Media directory does not exist: {:?}", self.media_dir);
        }

        if !self.media_dir.is_dir() {
            anyhow::bail!("Media path is not a directory: {:?}", self.media_dir);
        }

        if self.username.is_empty() {
            anyhow::bail!("Username cannot be empty");
        }

        if self.password.is_empty() {
            anyhow::bail!("Password cannot be empty");
        }

        Ok(())
    }
}
