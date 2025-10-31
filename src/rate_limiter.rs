use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Simple rate limiter for failed authentication attempts
/// Tracks failed login attempts per IP address
#[derive(Clone)]
pub struct AuthRateLimiter {
    attempts: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    max_attempts: usize,
    window: Duration,
}

impl AuthRateLimiter {
    /// Create a new rate limiter
    /// max_attempts: Maximum failed attempts allowed within the time window
    /// window: Time window for tracking attempts
    pub fn new(max_attempts: usize, window: Duration) -> Self {
        Self {
            attempts: Arc::new(RwLock::new(HashMap::new())),
            max_attempts,
            window,
        }
    }

    /// Check if an IP is rate limited
    /// Returns true if the IP has exceeded the rate limit
    pub async fn is_rate_limited(&self, ip: &str) -> bool {
        let mut attempts = self.attempts.write().await;

        // Get or create attempt history for this IP
        let ip_attempts = attempts.entry(ip.to_string()).or_insert_with(Vec::new);

        // Remove attempts older than the window
        let cutoff = Instant::now() - self.window;
        ip_attempts.retain(|&attempt_time| attempt_time > cutoff);

        // Check if rate limited
        ip_attempts.len() >= self.max_attempts
    }

    /// Record a failed authentication attempt
    pub async fn record_failure(&self, ip: &str) {
        let mut attempts = self.attempts.write().await;
        let ip_attempts = attempts.entry(ip.to_string()).or_insert_with(Vec::new);
        ip_attempts.push(Instant::now());

        tracing::debug!(
            ip = %ip,
            attempts = ip_attempts.len(),
            "Recorded failed auth attempt"
        );
    }

    /// Clear attempts for an IP (called on successful authentication)
    pub async fn clear(&self, ip: &str) {
        let mut attempts = self.attempts.write().await;
        if attempts.remove(ip).is_some() {
            tracing::debug!(ip = %ip, "Cleared rate limit history after successful auth");
        }
    }

    /// Cleanup old entries (call periodically)
    pub async fn cleanup(&self) {
        let mut attempts = self.attempts.write().await;
        let cutoff = Instant::now() - self.window;

        // Remove IPs with no recent attempts
        attempts.retain(|_, ip_attempts| {
            ip_attempts.retain(|&attempt_time| attempt_time > cutoff);
            !ip_attempts.is_empty()
        });

        tracing::debug!(
            tracked_ips = attempts.len(),
            "Cleaned up rate limiter"
        );
    }
}
