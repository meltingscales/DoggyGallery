use axum::{
    extract::Request,
    http::header::{HeaderValue, CONTENT_SECURITY_POLICY},
    middleware::Next,
    response::Response,
};

/// Middleware to add security headers to all responses
pub async fn add_security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // X-Content-Type-Options: Prevent MIME sniffing
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );

    // X-Frame-Options: Prevent clickjacking
    headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));

    // Strict-Transport-Security (HSTS): Force HTTPS
    headers.insert(
        "Strict-Transport-Security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
    );

    // Content-Security-Policy: Restrict resource loading
    // Allow inline scripts and styles for the gallery UI, but only from same origin
    headers.insert(
        CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data:; \
             media-src 'self'; \
             font-src 'self'; \
             connect-src 'self'; \
             frame-ancestors 'none'; \
             base-uri 'self'; \
             form-action 'self'",
        ),
    );

    // X-XSS-Protection: Enable XSS filter (legacy but harmless)
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );

    // Referrer-Policy: Control referrer information
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Permissions-Policy: Disable unnecessary browser features
    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static(
            "geolocation=(), microphone=(), camera=(), payment=(), usb=(), magnetometer=(), gyroscope=()",
        ),
    );

    response
}
