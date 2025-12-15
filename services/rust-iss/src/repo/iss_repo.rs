use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgPool, Row};

use crate::domain::{IssFetchLog, IssTrend, haversine_km};
use crate::errors::ApiError;

pub struct IssRepo {
    pool: PgPool,
}

impl IssRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Initialize ISS tables
    pub async fn init_tables(&self) -> Result<(), ApiError> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS iss_fetch_log(
                id BIGSERIAL PRIMARY KEY,
                fetched_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                source_url TEXT NOT NULL,
                payload JSONB NOT NULL
            )"
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Insert new ISS fetch record
    pub async fn insert(&self, source_url: &str, payload: Value) -> Result<i64, ApiError> {
        let row = sqlx::query(
            "INSERT INTO iss_fetch_log (source_url, payload)
             VALUES ($1, $2)
             RETURNING id"
        )
        .bind(source_url)
        .bind(&payload)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("id"))
    }

    /// Get the last ISS fetch record
    pub async fn get_last(&self) -> Result<Option<IssFetchLog>, ApiError> {
        let row_opt = sqlx::query(
            "SELECT id, fetched_at, source_url, payload
             FROM iss_fetch_log
             ORDER BY id DESC
             LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row_opt.map(|row| IssFetchLog {
            id: row.get("id"),
            fetched_at: row.get("fetched_at"),
            source_url: row.get("source_url"),
            payload: row.try_get("payload").unwrap_or(serde_json::json!({})),
        }))
    }

    /// Get the last N ISS fetch records
    pub async fn get_last_n(&self, n: i64) -> Result<Vec<IssFetchLog>, ApiError> {
        let rows = sqlx::query(
            "SELECT id, fetched_at, source_url, payload
             FROM iss_fetch_log
             ORDER BY id DESC
             LIMIT $1"
        )
        .bind(n)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| IssFetchLog {
                id: row.get("id"),
                fetched_at: row.get("fetched_at"),
                source_url: row.get("source_url"),
                payload: row.try_get("payload").unwrap_or(serde_json::json!({})),
            })
            .collect())
    }

    /// Calculate ISS trend from last 2 records
    pub async fn calculate_trend(&self) -> Result<IssTrend, ApiError> {
        let rows = sqlx::query(
            "SELECT fetched_at, payload
             FROM iss_fetch_log
             ORDER BY id DESC
             LIMIT 2"
        )
        .fetch_all(&self.pool)
        .await?;

        if rows.len() < 2 {
            return Ok(IssTrend::no_movement());
        }

        let t2: DateTime<Utc> = rows[0].get("fetched_at");
        let t1: DateTime<Utc> = rows[1].get("fetched_at");
        let p2: Value = rows[0].get("payload");
        let p1: Value = rows[1].get("payload");

        let lat1 = extract_number(&p1, "latitude");
        let lon1 = extract_number(&p1, "longitude");
        let lat2 = extract_number(&p2, "latitude");
        let lon2 = extract_number(&p2, "longitude");
        let v2 = extract_number(&p2, "velocity");

        let mut delta_km = 0.0;
        let mut movement = false;
        if let (Some(a1), Some(o1), Some(a2), Some(o2)) = (lat1, lon1, lat2, lon2) {
            delta_km = haversine_km(a1, o1, a2, o2);
            movement = delta_km > 0.1;
        }

        let dt_sec = (t2 - t1).num_milliseconds() as f64 / 1000.0;

        Ok(IssTrend {
            movement,
            delta_km,
            dt_sec,
            velocity_kmh: v2,
            from_time: Some(t1),
            to_time: Some(t2),
            from_lat: lat1,
            from_lon: lon1,
            to_lat: lat2,
            to_lon: lon2,
        })
    }
}

fn extract_number(v: &Value, key: &str) -> Option<f64> {
    if let Some(x) = v.get(key) {
        if let Some(f) = x.as_f64() {
            return Some(f);
        }
        if let Some(s) = x.as_str() {
            return s.parse::<f64>().ok();
        }
    }
    None
}
