use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use base64::Engine;
use subtle::ConstantTimeEq;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct AuthConfig {
    #[zeroize(skip)]
    pub username: String,
    pub password: String,
}

/// Middleware for HTTP Basic Authentication
pub async fn basic_auth_middleware(
    auth_config: axum::extract::State<AuthConfig>,
    request: Request,
    next: Next,
) -> Response {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    if let Some(auth_value) = auth_header {
        if let Some(credentials) = auth_value.strip_prefix("Basic ") {
            // Decode base64 credentials
            if let Ok(decoded) = base64::prelude::BASE64_STANDARD.decode(credentials) {
                if let Ok(credentials_str) = String::from_utf8(decoded) {
                    // Parse username:password
                    if let Some((username, password)) = credentials_str.split_once(':') {
                        // Use constant-time comparison to prevent timing attacks
                        let username_match = username.as_bytes().ct_eq(auth_config.username.as_bytes());
                        let password_match = password.as_bytes().ct_eq(auth_config.password.as_bytes());

                        if bool::from(username_match & password_match) {
                            return next.run(request).await;
                        }
                    }
                }
            }
        }
    }

    // Authentication failed - return 401 with WWW-Authenticate header
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(
            header::WWW_AUTHENTICATE,
            "Basic realm=\"DoggyGallery\", charset=\"UTF-8\"",
        )
        .body(Body::from("Authentication required"))
        .unwrap()
}
