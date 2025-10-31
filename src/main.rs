use axum::{
    middleware,
    routing::get,
    Router,
};
use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_governor::{
    governor::GovernorConfigBuilder,
    GovernorLayer,
};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod config;
mod constants;
mod handlers;
mod models;
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

    // Create authentication config
    let auth_config = AuthConfig {
        username: config.username.clone(),
        password: config.password.clone(),
    };

    // Configure rate limiting: 5 requests per second, burst of 10
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(5)
        .burst_size(10)
        .finish()
        .ok_or_else(|| anyhow::anyhow!("Failed to build rate limit config"))?;

    let rate_limit_layer = GovernorLayer {
        config: Arc::new(governor_conf),
    };

    // Build the application router
    let app = Router::new()
        .route("/", get(handlers::index_handler))
        .route("/browse/*path", get(handlers::list_directory_handler))
        .route("/media/*path", get(handlers::serve_media_handler))
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(security_headers::add_security_headers))
                .layer(rate_limit_layer)
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
