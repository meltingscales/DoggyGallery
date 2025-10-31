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

use crate::rate_limiter::AuthRateLimiter;

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct AuthConfig {
    #[zeroize(skip)]
    pub username: String,
    pub password: String,
    #[zeroize(skip)]
    pub rate_limiter: AuthRateLimiter,
}

/// Middleware for HTTP Basic Authentication
pub async fn basic_auth_middleware(
    auth_config: axum::extract::State<AuthConfig>,
    request: Request,
    next: Next,
) -> Response {
    // Extract client IP for logging and rate limiting
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    // Check rate limit for this IP
    if auth_config.rate_limiter.is_rate_limited(client_ip).await {
        tracing::warn!(
            client_ip = %client_ip,
            "Authentication rate limited - too many failed attempts"
        );

        return Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .header(
                header::WWW_AUTHENTICATE,
                "Basic realm=\"DoggyGallery\", charset=\"UTF-8\"",
            )
            .header(header::RETRY_AFTER, "60")
            .body(Body::from("Too many failed authentication attempts. Try again later."))
            .unwrap();
    }

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
                            // Clear rate limit on successful authentication
                            auth_config.rate_limiter.clear(client_ip).await;

                            tracing::debug!(
                                client_ip = %client_ip,
                                username = %username,
                                "Authentication successful"
                            );
                            return next.run(request).await;
                        } else {
                            // Record failed attempt
                            auth_config.rate_limiter.record_failure(client_ip).await;

                            tracing::warn!(
                                client_ip = %client_ip,
                                username = %username,
                                "Authentication failed - invalid credentials"
                            );
                        }
                    }
                }
            }
        }
    }

    // Authentication failed - record and return 401 with WWW-Authenticate header
    auth_config.rate_limiter.record_failure(client_ip).await;

    tracing::warn!(
        client_ip = %client_ip,
        "Authentication failed - no valid credentials provided"
    );

    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(
            header::WWW_AUTHENTICATE,
            "Basic realm=\"DoggyGallery\", charset=\"UTF-8\"",
        )
        .body(Body::from("Authentication required"))
        .unwrap()
}
