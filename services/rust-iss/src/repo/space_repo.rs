use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgPool, Row};
use tracing::error;

use crate::domain::SpaceCache;

pub struct SpaceRepo {
    pool: PgPool,
}

impl SpaceRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Initialize space_cache table
    pub async fn init_tables(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS space_cache (
                id BIGSERIAL PRIMARY KEY,
                source TEXT NOT NULL,
                fetched_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                payload JSONB NOT NULL
            )"
        )
        .execute(&self.pool)
        .await?;

        // Create index on source for faster lookups
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_space_cache_source
             ON space_cache(source, fetched_at DESC)"
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Insert new cache entry
    pub async fn insert_cache(&self, source: &str, payload: Value) -> Result<i64, sqlx::Error> {
        let row = sqlx::query(
            "INSERT INTO space_cache (source, payload)
             VALUES ($1, $2)
             RETURNING id"
        )
        .bind(source)
        .bind(&payload)
        .fetch_one(&self.pool)
        .await;

        match row {
            Ok(r) => Ok(r.get("id")),
            Err(e) => {
                error!("Failed to insert space cache for source {}: {:?}", source, e);
                Err(e)
            }
        }
    }

    /// Get latest cache entry for a source
    pub async fn get_latest(&self, source: &str) -> Result<Option<SpaceCache>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, source, fetched_at, payload
             FROM space_cache
             WHERE source = $1
             ORDER BY fetched_at DESC
             LIMIT 1"
        )
        .bind(source)
        .fetch_optional(&self.pool)
        .await;

        match row {
            Ok(Some(r)) => Ok(Some(SpaceCache {
                id: r.get("id"),
                source: r.get("source"),
                fetched_at: r.get("fetched_at"),
                payload: r.try_get("payload").unwrap_or(serde_json::json!({})),
            })),
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Failed to get latest cache for source {}: {:?}", source, e);
                Err(e)
            }
        }
    }

    /// Get all latest cache entries for multiple sources
    pub async fn get_all_latest(&self, sources: Vec<&str>) -> Result<Vec<SpaceCache>, sqlx::Error> {
        // Build query with DISTINCT ON for PostgreSQL
        let rows = sqlx::query(
            "SELECT DISTINCT ON (source) id, source, fetched_at, payload
             FROM space_cache
             WHERE source = ANY($1)
             ORDER BY source, fetched_at DESC"
        )
        .bind(&sources)
        .fetch_all(&self.pool)
        .await;

        match rows {
            Ok(rs) => Ok(rs
                .into_iter()
                .map(|row| SpaceCache {
                    id: row.get("id"),
                    source: row.get("source"),
                    fetched_at: row.get("fetched_at"),
                    payload: row.try_get("payload").unwrap_or(serde_json::json!({})),
                })
                .collect()),
            Err(e) => {
                error!("Failed to get all latest caches: {:?}", e);
                Err(e)
            }
        }
    }

    /// Get latest cache entry within a time window
    pub async fn get_latest_since(
        &self,
        source: &str,
        since: DateTime<Utc>,
    ) -> Result<Option<SpaceCache>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, source, fetched_at, payload
             FROM space_cache
             WHERE source = $1 AND fetched_at >= $2
             ORDER BY fetched_at DESC
             LIMIT 1"
        )
        .bind(source)
        .bind(since)
        .fetch_optional(&self.pool)
        .await;

        match row {
            Ok(Some(r)) => Ok(Some(SpaceCache {
                id: r.get("id"),
                source: r.get("source"),
                fetched_at: r.get("fetched_at"),
                payload: r.try_get("payload").unwrap_or(serde_json::json!({})),
            })),
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Failed to get latest cache since {:?} for source {}: {:?}", since, source, e);
                Err(e)
            }
        }
    }
}
