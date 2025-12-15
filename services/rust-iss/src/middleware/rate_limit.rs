use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use crate::errors::ApiError;
use crate::services::scheduler::AppState;

pub type RateLimiter = Arc<RwLock<HashMap<String, (u32, Instant)>>>;

const RATE_LIMIT_MAX_REQUESTS: u32 = 100;
const RATE_LIMIT_WINDOW_SECS: u64 = 60;

/// Rate limiting middleware using sliding window
pub async fn rate_limit_middleware(
    State(_state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Extract client IP from headers or connection info
    let client_ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .or_else(|| {
            req.headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
        })
        .unwrap_or("unknown")
        .to_string();

    // For now, we'll use a simple in-memory rate limiter
    // In production, you'd want to use Redis or a distributed cache
    static LIMITER: once_cell::sync::Lazy<RateLimiter> =
        once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

    let now = Instant::now();

    {
        let mut limiter = LIMITER.write().await;

        // Clean up old entries (older than window)
        limiter.retain(|_, (_, timestamp)| {
            now.duration_since(*timestamp).as_secs() < RATE_LIMIT_WINDOW_SECS
        });

        // Check and update rate limit for this client
        let entry = limiter.entry(client_ip.clone()).or_insert((0, now));

        // If the window has expired, reset the counter
        if now.duration_since(entry.1).as_secs() >= RATE_LIMIT_WINDOW_SECS {
            entry.0 = 0;
            entry.1 = now;
        }

        // Increment request count
        entry.0 += 1;

        // Check if limit exceeded
        if entry.0 > RATE_LIMIT_MAX_REQUESTS {
            return Err(ApiError::rate_limit(format!(
                "Rate limit exceeded: {} requests per {} seconds",
                RATE_LIMIT_MAX_REQUESTS, RATE_LIMIT_WINDOW_SECS
            )));
        }
    }

    // Continue to next middleware/handler
    Ok(next.run(req).await)
}
