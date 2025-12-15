use axum::{
    middleware,
    routing::get,
    Router,
};

use crate::handlers::{
    health_check, iss_trend, last_iss, osdr_list, osdr_sync, space_latest, space_refresh,
    space_summary, trigger_iss,
};
use crate::middleware::rate_limit::rate_limit_middleware;
use crate::services::scheduler::AppState;

/// Create the main application router with all routes
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        // ISS endpoints
        .route("/last", get(last_iss))
        .route("/fetch", get(trigger_iss))
        .route("/iss/trend", get(iss_trend))
        // OSDR endpoints
        .route("/osdr/sync", get(osdr_sync))
        .route("/osdr/list", get(osdr_list))
        // Space endpoints
        .route("/space/:src/latest", get(space_latest))
        .route("/space/refresh", get(space_refresh))
        .route("/space/summary", get(space_summary))
        // Apply middleware
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ))
        .with_state(state)
}
