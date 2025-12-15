use axum::Json;
use chrono::Utc;
use serde_json::{json, Value};

use crate::errors::ApiError;

/// Health check endpoint
pub async fn health_check() -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({
        "status": "ok",
        "now": Utc::now(),
        "service": "rust-iss"
    })))
}
