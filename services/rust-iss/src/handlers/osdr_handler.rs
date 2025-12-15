use axum::{extract::State, Json};
use axum::extract::Query;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use validator::Validate;

use crate::domain::OsdrItem;
use crate::errors::ApiError;
use crate::repo::OsdrRepo;
use crate::services::scheduler::AppState;
use crate::services::NasaService;

#[derive(Debug, Deserialize, Validate)]
pub struct PaginationParams {
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct OsdrListResponse {
    pub items: Vec<OsdrItem>,
    pub total: i64,
    pub limit: i64,
}

/// Manually trigger OSDR sync
pub async fn osdr_sync(State(mut state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let count =
        NasaService::fetch_and_store_osdr(&state.pool, &mut state.redis, &state.config).await?;

    Ok(Json(json!({
        "ok": true,
        "message": "OSDR sync completed",
        "items_processed": count
    })))
}

/// List OSDR items with pagination
pub async fn osdr_list(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<OsdrListResponse>, ApiError> {
    // Validate parameters
    params
        .validate()
        .map_err(|e| ApiError::validation(format!("Invalid parameters: {}", e)))?;

    let limit = params.limit.unwrap_or(10);
    let repo = OsdrRepo::new(state.pool.clone());

    // Get items
    let items = repo.list_items(limit).await?;

    // Get total count
    let total = repo.count_items().await?;

    Ok(Json(OsdrListResponse {
        items,
        total,
        limit,
    }))
}
