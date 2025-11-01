use axum::{
    body::Body,
    http::{header, StatusCode, Response, Uri},
    response::IntoResponse,
};
use rust_embed::RustEmbed;

/// Embed static files into the binary at compile time
#[derive(RustEmbed)]
#[folder = "static/"]
pub struct StaticAssets;

/// Handler for serving embedded static files
pub async fn serve_static(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches("/static/");

    match StaticAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();

            Response::builder()
                .status(StatusCode::OK)
                .header(
                    header::CONTENT_TYPE,
                    mime.as_ref(),
                )
                .header(
                    header::CACHE_CONTROL,
                    "public, max-age=31536000, immutable",
                )
                .body(Body::from(content.data.into_owned()))
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("404 - Not Found"))
            .unwrap(),
    }
}
