use axum::{
    middleware,
    routing::get,
    Router,
};
use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use rate_limiter::AuthRateLimiter;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod api;
mod archives;
mod auth;
mod config;
mod constants;
mod embedded;
mod handlers;
mod models;
mod rate_limiter;
mod security_headers;
mod templates;
mod tls;

use auth::{basic_auth_middleware, AuthConfig};
use config::Config;
use handlers::AppState;

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::filter_handler,
        handlers::random_media_handler,
        api::config_handler,
    ),
    components(
        schemas(
            handlers::FilterResponse,
            handlers::FilterResult,
            handlers::RandomMediaResponse,
            api::ConfigInfo,
        )
    ),
    tags(
        (name = "media", description = "Media file operations"),
        (name = "info", description = "Server configuration and information")
    ),
    info(
        title = "DoggyGallery API",
        version = "0.5.3",
        description = "A secure media gallery server with TLS 1.3 + HTTP/2, lazy loading, pagination, and random media selection",
    )
)]
struct ApiDoc;

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
        "Listening on: https://{}:{} ({} + {})",
        config.host,
        config.port,
        constants::TLS_VERSION,
        constants::HTTP_VERSION
    );

    // Initialize media cache
    let media_dir_canonical = config.media_dir.clone().canonicalize()?;
    tracing::info!("Building initial media cache...");
    let initial_cache = handlers::refresh_media_cache(&media_dir_canonical)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to build initial media cache: {:?}", e))?;
    let media_cache = Arc::new(RwLock::new(initial_cache));

    // Create application state
    let app_state = AppState {
        media_dir: media_dir_canonical.clone(),
        media_cache: media_cache.clone(),
    };

    // Start cache refresh task (refresh every 5 minutes)
    let cache_refresh_dir = media_dir_canonical.clone();
    let cache_refresh_cache = media_cache.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(300)); // Refresh every 5 minutes
        loop {
            interval.tick().await;
            match handlers::refresh_media_cache(&cache_refresh_dir).await {
                Ok(new_cache) => {
                    let mut cache = cache_refresh_cache.write().await;
                    *cache = new_cache;
                }
                Err(e) => {
                    tracing::error!("Failed to refresh media cache: {:?}", e);
                }
            }
        }
    });

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
        .route("/browse", get(handlers::browse_redirect_handler))
        .route("/browse/", get(handlers::browse_redirect_handler))
        .route("/browse/*path", get(handlers::list_directory_handler))
        .route("/music", get(handlers::music_index_handler))
        .route("/music/", get(handlers::music_redirect_handler))
        .route("/music/*path", get(handlers::music_list_handler))
        .route("/music-archive/*path", get(handlers::music_archive_handler))
        .route("/media/*path", get(handlers::serve_media_handler))
        .route("/thumbnail/*path", get(handlers::serve_thumbnail_handler))
        .route("/media-archive/*path", get(handlers::serve_archive_file_handler))
        .route("/album-art/*path", get(handlers::serve_album_art_handler))
        .route("/api/filter", get(handlers::filter_handler))
        .route("/api/random", get(handlers::random_media_handler))
        .route("/api/config", get(api::config_handler))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/static/*path", get(embedded::serve_static))
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
