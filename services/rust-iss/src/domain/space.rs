use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceCache {
    pub id: i64,
    pub source: String,
    pub fetched_at: DateTime<Utc>,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpaceCacheResponse {
    pub source: String,
    pub fetched_at: Option<DateTime<Utc>>,
    pub payload: Option<Value>,
}
