use axum::{extract::{Path, State}, Json};
use axum::extract::Query;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::domain::SpaceCache;
use crate::errors::ApiError;
use crate::repo::SpaceRepo;
use crate::services::scheduler::AppState;
use crate::services::NasaService;

#[derive(Debug, Deserialize)]
pub struct RefreshQuery {
    pub sources: Option<String>, // Comma-separated list
}

#[derive(Debug, Serialize)]
pub struct SpaceSummary {
    pub sources: HashMap<String, Option<SpaceCache>>,
    pub total: usize,
}

/// Get latest data for a specific source
pub async fn space_latest(
    Path(source): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Option<SpaceCache>>, ApiError> {
    let repo = SpaceRepo::new(state.pool.clone());
    let latest = repo.get_latest(&source).await?;
    Ok(Json(latest))
}

/// Refresh data for specified sources
pub async fn space_refresh(
    Query(query): Query<RefreshQuery>,
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let sources_str = query.sources.unwrap_or_else(|| "apod,neo,donki_flr,donki_cme,spacex".to_string());
    let sources: Vec<&str> = sources_str.split(',').map(|s| s.trim()).collect();

    let mut results = HashMap::new();

    for source in sources {
        let result = match source {
            "apod" => NasaService::fetch_apod(&state.pool, &state.config).await,
            "neo" => NasaService::fetch_neo(&state.pool, &state.config).await,
            "donki_flr" => NasaService::fetch_donki_flr(&state.pool, &state.config).await,
            "donki_cme" => NasaService::fetch_donki_cme(&state.pool, &state.config).await,
            "spacex" => NasaService::fetch_spacex(&state.pool, &state.config).await,
            _ => {
                results.insert(source.to_string(), json!({
                    "error": format!("Unknown source: {}", source)
                }));
                continue;
            }
        };

        match result {
            Ok(data) => {
                results.insert(source.to_string(), json!({
                    "ok": true,
                    "data": data
                }));
            }
            Err(e) => {
                results.insert(source.to_string(), json!({
                    "ok": false,
                    "error": e.message
                }));
            }
        }
    }

    Ok(Json(json!({
        "ok": true,
        "results": results
    })))
}

/// Get summary of all space data sources
pub async fn space_summary(State(state): State<AppState>) -> Result<Json<SpaceSummary>, ApiError> {
    let repo = SpaceRepo::new(state.pool.clone());
    let sources = vec!["apod", "neo", "donki_flr", "donki_cme", "spacex"];

    let all_latest = repo.get_all_latest(sources.clone()).await?;

    let mut sources_map = HashMap::new();
    for source in &sources {
        let found = all_latest.iter().find(|item| item.source == *source).cloned();
        sources_map.insert(source.to_string(), found);
    }

    Ok(Json(SpaceSummary {
        total: all_latest.len(),
        sources: sources_map,
    }))
}
