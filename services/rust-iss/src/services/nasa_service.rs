use chrono::{NaiveDate, Utc};
use redis::AsyncCommands;
use serde_json::Value;
use sqlx::PgPool;
use tracing::{error, info, warn};

use crate::clients::{NasaClient, SpaceXClient};
use crate::config::Config;
use crate::domain::OsdrInsert;
use crate::errors::ApiError;
use crate::repo::{OsdrRepo, SpaceRepo};

const OSDR_CACHE_KEY: &str = "nasa:osdr";
const OSDR_CACHE_TTL: u64 = 600; // 10 minutes

pub struct NasaService;

impl NasaService {
    /// Fetch and store OSDR datasets
    pub async fn fetch_and_store_osdr(
        pool: &PgPool,
        redis: &mut redis::aio::ConnectionManager,
        config: &Config,
    ) -> Result<usize, ApiError> {
        info!("Starting OSDR fetch and store operation");

        // Fetch from external API
        let client = NasaClient::new();
        let response = client
            .fetch_osdr(&config.nasa_osdr_url, &config.nasa_api_key)
            .await?;

        // Cache full response in Redis
        let cache_value = serde_json::to_string(&response)
            .map_err(|e| ApiError::internal(format!("Failed to serialize OSDR data: {}", e)))?;

        redis
            .set_ex::<_, _, ()>(OSDR_CACHE_KEY, cache_value, OSDR_CACHE_TTL)
            .await?;

        info!(
            "Cached OSDR response in Redis with TTL {} seconds",
            OSDR_CACHE_TTL
        );

        // Parse items from response
        let items = response
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ApiError::external_api("OSDR response missing 'items' array"))?;

        info!("Found {} OSDR items to process", items.len());

        // Upsert items to database
        let repo = OsdrRepo::new(pool.clone());
        let mut count = 0;

        for item in items {
            let insert = parse_osdr_item(item)?;
            match repo.upsert_item(insert).await {
                Ok(id) => {
                    count += 1;
                    if count <= 5 {
                        info!("Upserted OSDR item with id: {}", id);
                    }
                }
                Err(e) => {
                    warn!("Failed to upsert OSDR item: {:?}", e);
                }
            }
        }

        info!("Successfully processed {} OSDR items", count);
        Ok(count)
    }

    /// Fetch and store APOD (Astronomy Picture of the Day)
    pub async fn fetch_apod(pool: &PgPool, config: &Config) -> Result<Value, ApiError> {
        info!("Fetching APOD");

        let client = NasaClient::new();
        let payload = client.fetch_apod(&config.nasa_api_key).await?;

        // Store in space_cache
        let repo = SpaceRepo::new(pool.clone());
        let id = repo.insert_cache("apod", payload.clone()).await?;

        info!("Stored APOD in space_cache with id: {}", id);
        Ok(payload)
    }

    /// Fetch and store NEO (Near-Earth Objects) feed
    pub async fn fetch_neo(pool: &PgPool, config: &Config) -> Result<Value, ApiError> {
        info!("Fetching NEO feed");

        let client = NasaClient::new();
        let today = Utc::now().date_naive();
        let end_date = today + chrono::Duration::days(7);

        let payload = client
            .fetch_neo_feed(&config.nasa_api_key, today, end_date)
            .await?;

        // Store in space_cache
        let repo = SpaceRepo::new(pool.clone());
        let id = repo.insert_cache("neo", payload.clone()).await?;

        info!("Stored NEO feed in space_cache with id: {}", id);
        Ok(payload)
    }

    /// Fetch and store DONKI Solar Flare events
    pub async fn fetch_donki_flr(pool: &PgPool, config: &Config) -> Result<Value, ApiError> {
        info!("Fetching DONKI FLR events");

        let client = NasaClient::new();
        let today = Utc::now().date_naive();
        let start_date = today - chrono::Duration::days(30);

        let payload = client
            .fetch_donki_flr(&config.nasa_api_key, start_date, today)
            .await?;

        // Store in space_cache
        let repo = SpaceRepo::new(pool.clone());
        let id = repo.insert_cache("donki_flr", payload.clone()).await?;

        info!("Stored DONKI FLR in space_cache with id: {}", id);
        Ok(payload)
    }

    /// Fetch and store DONKI Coronal Mass Ejection events
    pub async fn fetch_donki_cme(pool: &PgPool, config: &Config) -> Result<Value, ApiError> {
        info!("Fetching DONKI CME events");

        let client = NasaClient::new();
        let today = Utc::now().date_naive();
        let start_date = today - chrono::Duration::days(30);

        let payload = client
            .fetch_donki_cme(&config.nasa_api_key, start_date, today)
            .await?;

        // Store in space_cache
        let repo = SpaceRepo::new(pool.clone());
        let id = repo.insert_cache("donki_cme", payload.clone()).await?;

        info!("Stored DONKI CME in space_cache with id: {}", id);
        Ok(payload)
    }

    /// Fetch and store SpaceX next launch
    pub async fn fetch_spacex(pool: &PgPool, config: &Config) -> Result<Value, ApiError> {
        info!("Fetching SpaceX next launch");

        let client = SpaceXClient::new();
        let payload = client.fetch_next_launch().await?;

        // Store in space_cache
        let repo = SpaceRepo::new(pool.clone());
        let id = repo.insert_cache("spacex", payload.clone()).await?;

        info!("Stored SpaceX launch in space_cache with id: {}", id);
        Ok(payload)
    }
}

/// Parse OSDR item from JSON
fn parse_osdr_item(item: &Value) -> Result<OsdrInsert, ApiError> {
    let dataset_id = item
        .get("dataset_id")
        .or_else(|| item.get("accession"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::external_api("OSDR item missing dataset_id"))?
        .to_string();

    let title = item
        .get("title")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let status = item
        .get("status")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let updated_at = item
        .get("updated_at")
        .or_else(|| item.get("release_date"))
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    Ok(OsdrInsert {
        dataset_id,
        title,
        status,
        updated_at,
        raw: item.clone(),
    })
}
