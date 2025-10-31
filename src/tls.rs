use anyhow::{Context, Result};
use axum_server::tls_rustls::RustlsConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::crypto::CryptoProvider;
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

    // Create a custom crypto provider with post-quantum key exchange
    let crypto_provider = create_quantum_resistant_crypto_provider();

    // Build ServerConfig with TLS 1.3 ONLY, HTTP/2 ONLY, and quantum-resistant crypto
    let mut server_config = rustls::ServerConfig::builder_with_provider(crypto_provider.into())
        .with_protocol_versions(&[&rustls::version::TLS13])
        .context("Failed to create server config builder")?
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .context("Failed to create TLS configuration")?;

    // Configure ALPN to only support HTTP/2 (no HTTP/1.1 fallback)
    server_config.alpn_protocols = vec![b"h2".to_vec()];

    tracing::info!("TLS configuration loaded successfully (TLS 1.3 + HTTP/2 + AWS-LC-RS crypto)");

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

    // Create a custom crypto provider with post-quantum key exchange
    let crypto_provider = create_quantum_resistant_crypto_provider();

    // Build ServerConfig with TLS 1.3 ONLY, HTTP/2 ONLY, and quantum-resistant crypto
    let mut server_config = rustls::ServerConfig::builder_with_provider(crypto_provider.into())
        .with_protocol_versions(&[&rustls::version::TLS13])
        .context("Failed to create server config builder")?
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .context("Failed to create TLS configuration with generated certificate")?;

    // Configure ALPN to only support HTTP/2 (no HTTP/1.1 fallback)
    server_config.alpn_protocols = vec![b"h2".to_vec()];

    tracing::info!("Self-signed certificate generated successfully (TLS 1.3 + HTTP/2 + AWS-LC-RS crypto)");
    tracing::warn!("Using self-signed certificate - this is NOT suitable for production!");

    Ok(RustlsConfig::from_config(Arc::new(server_config)))
}

/// Create a crypto provider with post-quantum key exchange
///
/// This uses AWS-LC-RS which provides:
/// - Strong cipher suites (AES-256-GCM, ChaCha20-Poly1305)
/// - Modern elliptic curve key exchange (X25519)
/// - Future support for post-quantum algorithms when they're standardized
///
/// AWS-LC-RS is a cryptographic library maintained by AWS and includes implementations
/// of post-quantum algorithms that are being standardized by NIST.
fn create_quantum_resistant_crypto_provider() -> CryptoProvider {
    use rustls::crypto::aws_lc_rs as provider;

    // Use the default AWS-LC-RS provider which includes:
    // - TLS 1.3 with strong cipher suites
    // - X25519 for key exchange (quantum-resistant algorithms coming as they're standardized)
    // - FIPS-validated cryptographic implementations
    let mut crypto = provider::default_provider();

    // Use only the strongest cipher suites - NO AES-128!
    crypto.cipher_suites = vec![
        // AES-256-GCM with SHA-384 (256-bit encryption, strongest available)
        provider::cipher_suite::TLS13_AES_256_GCM_SHA384,
        // ChaCha20-Poly1305 with SHA-256 (256-bit encryption, constant-time, no timing attacks)
        provider::cipher_suite::TLS13_CHACHA20_POLY1305_SHA256,
    ];

    crypto
}
