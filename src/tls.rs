use anyhow::{Context, Result};
use axum_server::tls_rustls::RustlsConfig;
use std::path::Path;

/// Load TLS configuration from certificate and key files
/// This enforces TLS 1.3 only
pub async fn load_tls_config(cert_path: &Path, key_path: &Path) -> Result<RustlsConfig> {
    tracing::info!("Loading TLS certificates...");
    tracing::info!("  Certificate: {:?}", cert_path);
    tracing::info!("  Private key: {:?}", key_path);

    // RustlsConfig will load and validate the certificates
    let config = RustlsConfig::from_pem_file(cert_path, key_path)
        .await
        .context("Failed to load TLS configuration")?;

    tracing::info!("TLS configuration loaded successfully (enforcing TLS 1.3)");

    Ok(config)
}

/// Generate self-signed certificate and private key on-the-fly
pub async fn generate_self_signed_config() -> Result<RustlsConfig> {
    tracing::info!("Generating self-signed certificate...");

    let subject_alt_names = vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "::1".to_string(),
    ];

    let cert = rcgen::generate_simple_self_signed(subject_alt_names)
        .context("Failed to generate self-signed certificate")?;

    let cert_pem = cert.cert.pem();
    let key_pem = cert.key_pair.serialize_pem();

    // Create a temporary config from PEM strings
    let config = RustlsConfig::from_pem(cert_pem.into_bytes(), key_pem.into_bytes())
        .await
        .context("Failed to create TLS configuration with generated certificate")?;

    tracing::info!("Self-signed certificate generated successfully");
    tracing::warn!("Using self-signed certificate - this is NOT suitable for production!");

    Ok(config)
}
