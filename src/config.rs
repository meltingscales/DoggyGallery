use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(name = "doggygallery")]
#[command(about = "A secure media gallery server with HTTPS and authentication", long_about = None)]
pub struct Config {
    /// Path to TLS certificate file (required unless --self-signed-certs-on-the-fly is used)
    #[arg(long, env = "DOGGYGALLERY_CERT")]
    pub cert: Option<PathBuf>,

    /// Path to TLS private key file (required unless --self-signed-certs-on-the-fly is used)
    #[arg(long, env = "DOGGYGALLERY_KEY")]
    pub key: Option<PathBuf>,

    /// Generate self-signed certificates on-the-fly (development only, not for production)
    #[arg(long, env = "DOGGYGALLERY_SELF_SIGNED")]
    pub self_signed_certs_on_the_fly: bool,

    /// Directory containing media files to serve
    #[arg(long, env = "DOGGYGALLERY_MEDIA_DIR")]
    pub media_dir: PathBuf,

    /// Username for HTTP Basic Authentication
    #[arg(long, env = "DOGGYGALLERY_USERNAME")]
    pub username: String,

    /// Password for HTTP Basic Authentication
    #[arg(long, env = "DOGGYGALLERY_PASSWORD")]
    pub password: String,

    /// Host to bind to
    #[arg(long, default_value = "0.0.0.0", env = "DOGGYGALLERY_HOST")]
    pub host: String,

    /// Port to listen on (default: 7833 - "RUFF" on phone keypad!)
    #[arg(long, default_value = "7833", env = "DOGGYGALLERY_PORT")]
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
