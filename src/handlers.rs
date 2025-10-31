use askama::Template;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
};
use axum::http::header::CONTENT_SECURITY_POLICY;
use percent_encoding::percent_decode_str;
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

    // Only serve image and video files
    let file_name = canonical_path.file_name()
        .and_then(|n| n.to_str())
        .ok_or(AppError::InvalidPath)?;

    if !is_image(file_name) && !is_video(file_name) {
        return Err(AppError::Forbidden);
    }

    // Read the file
    let contents = fs::read(&canonical_path)
        .await
        .map_err(|_| AppError::InternalError)?;

    // Determine MIME type
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
    lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".png")
        || lower.ends_with(".gif")
        || lower.ends_with(".webp")
        || lower.ends_with(".bmp")
        || lower.ends_with(".svg")
}

fn is_video(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    lower.ends_with(".mp4")
        || lower.ends_with(".webm")
        || lower.ends_with(".mkv")
        || lower.ends_with(".avi")
        || lower.ends_with(".mov")
        || lower.ends_with(".flv")
        || lower.ends_with(".wmv")
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
