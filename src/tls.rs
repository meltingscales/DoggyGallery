use anyhow::{Context, Result};
use axum_server::tls_rustls::RustlsConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::path::Path;
use std::sync::Arc;

/// Load TLS configuration from certificate and key files
/// This enforces TLS 1.3 only
pub async fn load_tls_config(cert_path: &Path, key_path: &Path) -> Result<RustlsConfig> {
    tracing::info!("Loading TLS certificates...");
    tracing::info!("  Certificate: {:?}", cert_path);
    tracing::info!("  Private key: {:?}", key_path);

    // Read certificate and key files
    let cert_file = tokio::fs::read(cert_path)
        .await
        .with_context(|| format!("Failed to read certificate file: {:?}", cert_path))?;

    let key_file = tokio::fs::read(key_path)
        .await
        .with_context(|| format!("Failed to read private key file: {:?}", key_path))?;

    // Parse certificates
    let certs: Vec<CertificateDer> = rustls_pemfile::certs(&mut &cert_file[..])
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to parse certificate file")?;

    if certs.is_empty() {
        anyhow::bail!("No certificates found in certificate file");
    }

    // Parse private key
    let key = rustls_pemfile::private_key(&mut &key_file[..])
        .context("Failed to parse private key file")?
        .context("No private key found in key file")?;

    // Build ServerConfig with TLS 1.3 ONLY
    let server_config = rustls::ServerConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .context("Failed to create TLS configuration")?;

    tracing::info!("TLS configuration loaded successfully (enforcing TLS 1.3 ONLY)");

    // Convert to RustlsConfig
    Ok(RustlsConfig::from_config(Arc::new(server_config)))
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

    let cert_der = cert.cert.der().to_vec();
    let key_der = cert.key_pair.serialize_der();

    let certs = vec![CertificateDer::from(cert_der)];
    let key = PrivateKeyDer::try_from(key_der)
        .map_err(|e| anyhow::anyhow!("Failed to parse generated private key: {}", e))?;

    // Build ServerConfig with TLS 1.3 ONLY
    let server_config = rustls::ServerConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .context("Failed to create TLS configuration with generated certificate")?;

    tracing::info!("Self-signed certificate generated successfully (TLS 1.3 ONLY)");
    tracing::warn!("Using self-signed certificate - this is NOT suitable for production!");

    Ok(RustlsConfig::from_config(Arc::new(server_config)))
}
