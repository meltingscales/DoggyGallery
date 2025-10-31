use askama::Template;
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response, Json},
};
use axum::http::header::CONTENT_SECURITY_POLICY;
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

use crate::constants;
use crate::models::{DirectoryEntry, DirectoryListing, EntryType};
use crate::templates::GalleryTemplate;

#[derive(Clone)]
pub struct AppState {
    pub media_dir: PathBuf,
}

/// Handler for the root path - shows the media directory
pub async fn index_handler(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    list_directory_handler(State(state), Path("".to_string())).await
}

/// Handler for listing directories
pub async fn list_directory_handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<Html<String>, AppError> {
    // Decode the URL-encoded path
    let decoded_path = percent_decode_str(&path)
        .decode_utf8()
        .map_err(|_| AppError::InvalidPath)?;

    // Construct the full path
    let full_path = state.media_dir.join(decoded_path.as_ref());

    // Canonicalize to prevent path traversal attacks
    let canonical_path = full_path
        .canonicalize()
        .map_err(|_| AppError::NotFound)?;

    // Ensure the path is within the media directory
    if !canonical_path.starts_with(&state.media_dir) {
        return Err(AppError::Forbidden);
    }

    // Check if it's a directory
    if !canonical_path.is_dir() {
        return Err(AppError::NotFound);
    }

    // Read directory contents
    let mut entries = Vec::new();
    let mut read_dir = fs::read_dir(&canonical_path)
        .await
        .map_err(|_| AppError::InternalError)?;

    while let Some(entry) = read_dir
        .next_entry()
        .await
        .map_err(|_| AppError::InternalError)?
    {
        let metadata = entry.metadata().await.map_err(|_| AppError::InternalError)?;
        let file_name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files (starting with .)
        if file_name.starts_with('.') {
            continue;
        }

        let entry_type = if metadata.is_dir() {
            EntryType::Directory
        } else if is_image(&file_name) {
            EntryType::Image
        } else if is_video(&file_name) {
            EntryType::Video
        } else if is_audio(&file_name) {
            EntryType::Audio
        } else {
            continue; // Skip non-media files
        };

        // Build relative path for URL
        let relative_path = if path.is_empty() {
            file_name.clone()
        } else {
            format!("{}/{}", path, file_name)
        };

        entries.push(DirectoryEntry {
            name: file_name,
            path: relative_path,
            entry_type,
            size: metadata.len(),
        });
    }

    // Sort: directories first, then by name
    entries.sort_by(|a, b| {
        match (&a.entry_type, &b.entry_type) {
            (EntryType::Directory, EntryType::Directory) => a.name.cmp(&b.name),
            (EntryType::Directory, _) => std::cmp::Ordering::Less,
            (_, EntryType::Directory) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });

    let listing = DirectoryListing {
        current_path: path.clone(),
        parent_path: if path.is_empty() {
            None
        } else {
            Some(
                PathBuf::from(&path)
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default(),
            )
        },
        entries,
    };

    let template = GalleryTemplate {
        listing,
        emoji_prefix: constants::EMOJI_PREFIX,
    };
    Ok(Html(template.render().map_err(|_| AppError::InternalError)?))
}

/// Handler for serving media files
pub async fn serve_media_handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<Response, AppError> {
    // Decode the URL-encoded path
    let decoded_path = percent_decode_str(&path)
        .decode_utf8()
        .map_err(|_| AppError::InvalidPath)?;

    // Construct the full path
    let full_path = state.media_dir.join(decoded_path.as_ref());

    // Canonicalize to prevent path traversal attacks
    let canonical_path = full_path
        .canonicalize()
        .map_err(|_| AppError::NotFound)?;

    // Ensure the path is within the media directory
    if !canonical_path.starts_with(&state.media_dir) {
        return Err(AppError::Forbidden);
    }

    // Check if it's a file
    if !canonical_path.is_file() {
        return Err(AppError::NotFound);
    }

    // Only serve image, video, and audio files
    let file_name = canonical_path.file_name()
        .and_then(|n| n.to_str())
        .ok_or(AppError::InvalidPath)?;

    if !is_image(file_name) && !is_video(file_name) && !is_audio(file_name) {
        return Err(AppError::Forbidden);
    }

    // Read the file
    let contents = fs::read(&canonical_path)
        .await
        .map_err(|_| AppError::InternalError)?;

    // Validate MIME type from file contents (magic bytes)
    // This prevents serving malicious files with fake extensions
    let detected_type = infer::get(&contents);

    if let Some(file_type) = detected_type {
        let mime = file_type.mime_type();

        // Validate the detected MIME type matches the expected category
        let is_valid = if is_image(file_name) {
            mime.starts_with("image/")
        } else if is_video(file_name) {
            mime.starts_with("video/")
        } else if is_audio(file_name) {
            mime.starts_with("audio/")
        } else {
            false
        };

        if !is_valid {
            tracing::warn!(
                file = %file_name,
                detected_mime = %mime,
                "MIME type validation failed - file extension doesn't match content"
            );
            return Err(AppError::Forbidden);
        }

        tracing::debug!(
            file = %file_name,
            detected_mime = %mime,
            "MIME type validation passed"
        );
    } else {
        // If we can't detect the type, reject for safety
        tracing::warn!(
            file = %file_name,
            "Could not detect MIME type from file contents"
        );
        return Err(AppError::Forbidden);
    }

    // Determine MIME type for response
    let mime_type = mime_guess::from_path(&canonical_path)
        .first_or_octet_stream()
        .to_string();

    // Special handling for SVG files to prevent XSS
    // SVG files can contain JavaScript, so we sandbox them
    let mut response_builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CACHE_CONTROL, "public, max-age=3600");

    if file_name.to_lowercase().ends_with(".svg") {
        // Serve SVG with restrictive CSP to prevent script execution
        response_builder = response_builder
            .header(header::CONTENT_TYPE, "image/svg+xml")
            .header(
                CONTENT_SECURITY_POLICY,
                "default-src 'none'; style-src 'unsafe-inline'; sandbox",
            );
        tracing::debug!("Serving SVG file with sandboxed CSP: {}", file_name);
    } else {
        response_builder = response_builder.header(header::CONTENT_TYPE, mime_type);
    }

    // Return the file with appropriate headers
    Ok(response_builder.body(Body::from(contents)).unwrap())
}

fn is_image(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    constants::IMAGE_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
}

fn is_video(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    constants::VIDEO_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
}

fn is_audio(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    constants::AUDIO_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
}

/// Filter query parameters
#[derive(Debug, Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct FilterQuery {
    /// Filter by file type (image, video, or audio)
    #[serde(rename = "type")]
    file_type: Option<String>,
    /// Filter by file extension (e.g., .jpg, .mp4)
    extension: Option<String>,
    /// Fuzzy match on file name
    name: Option<String>,
}

/// Filter response
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct FilterResponse {
    /// List of matching files
    results: Vec<FilterResult>,
    /// Total number of results
    total: usize,
}

/// Individual filter result
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct FilterResult {
    /// Relative path to the file
    path: String,
    /// File name
    name: String,
    /// File size in bytes
    size: u64,
    /// File type (image, video, or audio)
    file_type: String,
}

/// Search and filter media files
#[utoipa::path(
    get,
    path = "/api/filter",
    params(FilterQuery),
    responses(
        (status = 200, description = "List of matching files", body = FilterResponse)
    ),
    tag = "media"
)]
pub async fn filter_handler(
    State(state): State<AppState>,
    Query(query): Query<FilterQuery>,
) -> Result<Json<FilterResponse>, AppError> {
    let mut results = Vec::new();

    // Recursively search all files
    search_directory(&state.media_dir, "", &query, &mut results).await?;

    // Sort by name
    results.sort_by(|a, b| a.name.cmp(&b.name));

    let total = results.len();

    Ok(Json(FilterResponse { results, total }))
}

/// Recursively search directory for matching files
async fn search_directory(
    base_path: &PathBuf,
    relative_path: &str,
    query: &FilterQuery,
    results: &mut Vec<FilterResult>,
) -> Result<(), AppError> {
    let current_path = if relative_path.is_empty() {
        base_path.clone()
    } else {
        base_path.join(relative_path)
    };

    let mut read_dir = fs::read_dir(&current_path)
        .await
        .map_err(|_| AppError::InternalError)?;

    while let Some(entry) = read_dir
        .next_entry()
        .await
        .map_err(|_| AppError::InternalError)?
    {
        let metadata = entry.metadata().await.map_err(|_| AppError::InternalError)?;
        let file_name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files
        if file_name.starts_with('.') {
            continue;
        }

        let entry_relative_path = if relative_path.is_empty() {
            file_name.clone()
        } else {
            format!("{}/{}", relative_path, file_name)
        };

        if metadata.is_dir() {
            // Recurse into subdirectory
            Box::pin(search_directory(
                base_path,
                &entry_relative_path,
                query,
                results,
            ))
            .await?;
        } else {
            // Check if file matches filters
            let file_type = if is_image(&file_name) {
                "image"
            } else if is_video(&file_name) {
                "video"
            } else if is_audio(&file_name) {
                "audio"
            } else {
                continue; // Skip non-media files
            };

            // Apply filters
            if let Some(ref type_filter) = query.file_type {
                if file_type != type_filter {
                    continue;
                }
            }

            if let Some(ref ext_filter) = query.extension {
                let file_ext = file_name.to_lowercase();
                if !file_ext.ends_with(&ext_filter.to_lowercase()) {
                    continue;
                }
            }

            if let Some(ref name_filter) = query.name {
                // Fuzzy matching: check if filter is contained in filename (case insensitive)
                if !file_name.to_lowercase().contains(&name_filter.to_lowercase()) {
                    continue;
                }
            }

            results.push(FilterResult {
                path: entry_relative_path.clone(),
                name: file_name.clone(),
                size: metadata.len(),
                file_type: file_type.to_string(),
            });
        }
    }

    Ok(())
}

/// Application error types
#[derive(Debug)]
pub enum AppError {
    NotFound,
    Forbidden,
    InvalidPath,
    InternalError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found"),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden"),
            AppError::InvalidPath => (StatusCode::BAD_REQUEST, "Invalid path"),
            AppError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        (status, message).into_response()
    }
}
