use axum::{response::Json, extract::State};
use serde::Serialize;
use utoipa::ToSchema;

use crate::constants;
use crate::handlers::AppState;

/// Configuration information about supported file types
#[derive(Debug, Serialize, ToSchema)]
pub struct ConfigInfo {
    /// Emoji prefix displayed in the app
    pub emoji_prefix: String,
    /// Application name
    pub app_name: String,
    /// TLS version enforced
    pub tls_version: String,
    /// HTTP version enforced
    pub http_version: String,
    /// Supported image file extensions
    pub image_extensions: Vec<String>,
    /// Supported video file extensions
    pub video_extensions: Vec<String>,
    /// Supported audio file extensions
    pub audio_extensions: Vec<String>,
}

/// Get configuration information
#[utoipa::path(
    get,
    path = "/api/config",
    responses(
        (status = 200, description = "Configuration information", body = ConfigInfo)
    ),
    tag = "info"
)]
pub async fn config_handler(State(_state): State<AppState>) -> Json<ConfigInfo> {
    Json(ConfigInfo {
        emoji_prefix: constants::EMOJI_PREFIX.to_string(),
        app_name: constants::APP_NAME.to_string(),
        tls_version: constants::TLS_VERSION.to_string(),
        http_version: constants::HTTP_VERSION.to_string(),
        image_extensions: constants::IMAGE_EXTENSIONS.iter().map(|s| s.to_string()).collect(),
        video_extensions: constants::VIDEO_EXTENSIONS.iter().map(|s| s.to_string()).collect(),
        audio_extensions: constants::AUDIO_EXTENSIONS.iter().map(|s| s.to_string()).collect(),
    })
}
