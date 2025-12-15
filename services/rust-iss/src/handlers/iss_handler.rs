use axum::{extract::State, Json};
use serde_json::Value;

use crate::domain::{IssFetchLog, IssTrend};
use crate::errors::ApiError;
use crate::services::scheduler::AppState;
use crate::services::IssService;

/// Get last ISS position (from Redis cache or DB)
pub async fn last_iss(State(mut state): State<AppState>) -> Result<Json<IssFetchLog>, ApiError> {
    let log = IssService::get_last(&state.pool, &mut state.redis).await?;
    Ok(Json(log))
}

/// Manually trigger ISS position fetch
pub async fn trigger_iss(State(mut state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let payload = IssService::fetch_and_store(&state.pool, &mut state.redis, &state.config).await?;
    Ok(Json(payload))
}

/// Calculate ISS trend from last 2 records
pub async fn iss_trend(State(state): State<AppState>) -> Result<Json<IssTrend>, ApiError> {
    let trend = IssService::calculate_trend(&state.pool).await?;
    Ok(Json(trend))
}
