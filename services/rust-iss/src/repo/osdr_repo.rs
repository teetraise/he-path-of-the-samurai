use sqlx::{PgPool, Row};
use tracing::error;

use crate::domain::{OsdrInsert, OsdrItem};

pub struct OsdrRepo {
    pool: PgPool,
}

impl OsdrRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Initialize OSDR tables
    pub async fn init_tables(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS osdr_items (
                id BIGSERIAL PRIMARY KEY,
                dataset_id TEXT UNIQUE,
                title TEXT,
                status TEXT,
                updated_at TIMESTAMPTZ,
                inserted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                raw JSONB NOT NULL
            )"
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Upsert OSDR item by dataset_id
    pub async fn upsert_item(&self, item: OsdrInsert) -> Result<i64, sqlx::Error> {
        let row = sqlx::query(
            "INSERT INTO osdr_items (dataset_id, title, status, updated_at, raw)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (dataset_id)
             DO UPDATE SET
                title = EXCLUDED.title,
                status = EXCLUDED.status,
                updated_at = EXCLUDED.updated_at,
                raw = EXCLUDED.raw,
                inserted_at = now()
             RETURNING id"
        )
        .bind(&item.dataset_id)
        .bind(&item.title)
        .bind(&item.status)
        .bind(item.updated_at)
        .bind(&item.raw)
        .fetch_one(&self.pool)
        .await;

        match row {
            Ok(r) => Ok(r.get("id")),
            Err(e) => {
                error!("Failed to upsert OSDR item: {:?}", e);
                Err(e)
            }
        }
    }

    /// List OSDR items with limit
    pub async fn list_items(&self, limit: i64) -> Result<Vec<OsdrItem>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, dataset_id, title, status, updated_at, inserted_at, raw
             FROM osdr_items
             ORDER BY inserted_at DESC
             LIMIT $1"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await;

        match rows {
            Ok(rs) => Ok(rs
                .into_iter()
                .map(|row| OsdrItem {
                    id: row.get("id"),
                    dataset_id: row.get("dataset_id"),
                    title: row.get("title"),
                    status: row.get("status"),
                    updated_at: row.get("updated_at"),
                    inserted_at: row.get("inserted_at"),
                    raw: row.try_get("raw").unwrap_or(serde_json::json!({})),
                })
                .collect()),
            Err(e) => {
                error!("Failed to list OSDR items: {:?}", e);
                Err(e)
            }
        }
    }

    /// Count total OSDR items
    pub async fn count_items(&self) -> Result<i64, sqlx::Error> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM osdr_items")
            .fetch_one(&self.pool)
            .await;

        match row {
            Ok(r) => Ok(r.get("count")),
            Err(e) => {
                error!("Failed to count OSDR items: {:?}", e);
                Err(e)
            }
        }
    }
}
