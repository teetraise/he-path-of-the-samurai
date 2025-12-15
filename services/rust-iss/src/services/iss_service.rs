use redis::AsyncCommands;
use serde_json::Value;
use sqlx::PgPool;
use tracing::{error, info};

use crate::clients::IssClient;
use crate::config::Config;
use crate::domain::{IssFetchLog, IssTrend};
use crate::errors::ApiError;
use crate::repo::IssRepo;

const ISS_CACHE_KEY: &str = "iss:last";
const ISS_CACHE_TTL: u64 = 60; // 60 seconds

pub struct IssService;

impl IssService {
    /// Fetch ISS position from API, cache in Redis, and store in DB
    pub async fn fetch_and_store(
        pool: &PgPool,
        redis: &mut redis::aio::ConnectionManager,
        config: &Config,
    ) -> Result<Value, ApiError> {
        info!("Starting ISS fetch and store operation");

        // Fetch from external API
        let client = IssClient::new();
        let payload = client.fetch_position(&config.iss_url).await?;

        // Cache in Redis with TTL
        let cache_value = serde_json::to_string(&payload)
            .map_err(|e| ApiError::internal(format!("Failed to serialize ISS data: {}", e)))?;

        redis
            .set_ex::<_, _, ()>(ISS_CACHE_KEY, cache_value, ISS_CACHE_TTL)
            .await?;

        info!("Cached ISS position in Redis with TTL {} seconds", ISS_CACHE_TTL);

        // Store in database
        let repo = IssRepo::new(pool.clone());
        let id = repo.insert(&config.iss_url, payload.clone()).await?;

        info!("Stored ISS position in DB with id: {}", id);

        Ok(payload)
    }

    /// Get last ISS position - check Redis first, then DB
    pub async fn get_last(
        pool: &PgPool,
        redis: &mut redis::aio::ConnectionManager,
    ) -> Result<IssFetchLog, ApiError> {
        info!("Getting last ISS position");

        // Try Redis cache first
        match redis.get::<_, String>(ISS_CACHE_KEY).await {
            Ok(cached) => {
                info!("Cache hit: Retrieved ISS position from Redis");
                match serde_json::from_str::<Value>(&cached) {
                    Ok(payload) => {
                        // Return as IssFetchLog with dummy values for id and timestamp
                        return Ok(IssFetchLog {
                            id: 0,
                            fetched_at: chrono::Utc::now(),
                            source_url: "cache".to_string(),
                            payload,
                        });
                    }
                    Err(e) => {
                        error!("Failed to deserialize cached ISS data: {:?}", e);
                        // Fall through to DB query
                    }
                }
            }
            Err(e) => {
                info!("Cache miss: {:?}. Falling back to database", e);
            }
        }

        // Cache miss or error - query database
        let repo = IssRepo::new(pool.clone());
        match repo.get_last().await? {
            Some(log) => {
                info!("Retrieved ISS position from database");
                Ok(log)
            }
            None => {
                error!("No ISS position found in database");
                Err(ApiError::not_found("No ISS position data available"))
            }
        }
    }

    /// Calculate ISS trend from last 2 records in database
    pub async fn calculate_trend(pool: &PgPool) -> Result<IssTrend, ApiError> {
        info!("Calculating ISS trend");

        let repo = IssRepo::new(pool.clone());
        let trend = repo.calculate_trend().await?;

        info!(
            "Calculated ISS trend: movement={}, delta_km={:.2}",
            trend.movement, trend.delta_km
        );

        Ok(trend)
    }
}
