use axum::{
    middleware,
    routing::get,
    Router,
};
use clap::Parser;
use std::net::SocketAddr;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use rate_limiter::AuthRateLimiter;

mod auth;
mod config;
mod constants;
mod handlers;
mod models;
mod rate_limiter;
mod security_headers;
mod templates;
mod tls;

use auth::{basic_auth_middleware, AuthConfig};
use config::Config;
use handlers::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "doggygallery=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse configuration
    let config = Config::parse();
    config.validate()?;

    tracing::info!(
        "{} Starting {}...",
        constants::EMOJI_PREFIX,
        constants::APP_NAME
    );
    tracing::info!("Media directory: {:?}", config.media_dir);
    tracing::info!(
        "Listening on: https://{}:{} ({})",
        config.host,
        config.port,
        constants::TLS_VERSION
    );

    // Create application state
    let app_state = AppState {
        media_dir: config.media_dir.clone().canonicalize()?,
    };

    // Create rate limiter for failed auth attempts
    // Allow 10 failed attempts within a 60-second window
    let rate_limiter = AuthRateLimiter::new(10, Duration::from_secs(60));

    // Start cleanup task to remove old rate limit entries
    let cleanup_limiter = rate_limiter.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(300)); // Cleanup every 5 minutes
        loop {
            interval.tick().await;
            cleanup_limiter.cleanup().await;
        }
    });

    // Create authentication config
    let auth_config = AuthConfig {
        username: config.username.clone(),
        password: config.password.clone(),
        rate_limiter,
    };

    // Build the application router
    let app = Router::new()
        .route("/", get(handlers::index_handler))
        .route("/browse/*path", get(handlers::list_directory_handler))
        .route("/media/*path", get(handlers::serve_media_handler))
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(security_headers::add_security_headers))
                .layer(middleware::from_fn_with_state(
                    auth_config,
                    basic_auth_middleware,
                ))
                .layer(CompressionLayer::new())
                .layer(TraceLayer::new_for_http()),
        )
        .with_state(app_state);

    // Load or generate TLS configuration
    let tls_config = if config.self_signed_certs_on_the_fly {
        tls::generate_self_signed_config().await?
    } else {
        let cert_path = config.cert.as_ref().unwrap();
        let key_path = config.key.as_ref().unwrap();
        tls::load_tls_config(cert_path, key_path).await?
    };

    // Create the server address
    let addr = format!("{}:{}", config.host, config.port)
        .parse()
        .expect("Invalid address");

    tracing::info!("Server ready! Accepting connections...");

    // Start the HTTPS server with TLS 1.3
    // Use into_make_service_with_connect_info to provide SocketAddr for rate limiting
    axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}
