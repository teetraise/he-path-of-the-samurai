use chrono::NaiveDate;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use tracing::{error, info, warn};

pub struct NasaClient {
    client: Client,
}

impl NasaClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Cassiopeia-ISS/1.0")
            .build()
            .expect("Failed to build reqwest client");

        Self { client }
    }

    /// Fetch OSDR datasets
    pub async fn fetch_osdr(&self, url: &str, api_key: &str) -> Result<Value, reqwest::Error> {
        info!("Fetching OSDR data from: {}", url);

        fetch_with_retry(
            || async {
                self.client
                    .get(url)
                    .query(&[("api_key", api_key)])
                    .send()
                    .await?
                    .json::<Value>()
                    .await
            },
            3,
        )
        .await
    }

    /// Fetch Astronomy Picture of the Day (APOD)
    pub async fn fetch_apod(&self, api_key: &str) -> Result<Value, reqwest::Error> {
        let url = "https://api.nasa.gov/planetary/apod";
        info!("Fetching APOD from: {}", url);

        fetch_with_retry(
            || async {
                self.client
                    .get(url)
                    .query(&[("api_key", api_key)])
                    .send()
                    .await?
                    .json::<Value>()
                    .await
            },
            3,
        )
        .await
    }

    /// Fetch Near-Earth Object (NEO) feed
    pub async fn fetch_neo_feed(
        &self,
        api_key: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Value, reqwest::Error> {
        let url = "https://api.nasa.gov/neo/rest/v1/feed";
        info!(
            "Fetching NEO feed from {} to {}",
            start_date, end_date
        );

        fetch_with_retry(
            || async {
                self.client
                    .get(url)
                    .query(&[
                        ("api_key", api_key),
                        ("start_date", &start_date.to_string()),
                        ("end_date", &end_date.to_string()),
                    ])
                    .send()
                    .await?
                    .json::<Value>()
                    .await
            },
            3,
        )
        .await
    }

    /// Fetch DONKI Solar Flare (FLR) events
    pub async fn fetch_donki_flr(
        &self,
        api_key: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Value, reqwest::Error> {
        let url = "https://api.nasa.gov/DONKI/FLR";
        info!(
            "Fetching DONKI FLR events from {} to {}",
            start_date, end_date
        );

        fetch_with_retry(
            || async {
                self.client
                    .get(url)
                    .query(&[
                        ("api_key", api_key),
                        ("startDate", &start_date.to_string()),
                        ("endDate", &end_date.to_string()),
                    ])
                    .send()
                    .await?
                    .json::<Value>()
                    .await
            },
            3,
        )
        .await
    }

    /// Fetch DONKI Coronal Mass Ejection (CME) events
    pub async fn fetch_donki_cme(
        &self,
        api_key: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Value, reqwest::Error> {
        let url = "https://api.nasa.gov/DONKI/CME";
        info!(
            "Fetching DONKI CME events from {} to {}",
            start_date, end_date
        );

        fetch_with_retry(
            || async {
                self.client
                    .get(url)
                    .query(&[
                        ("api_key", api_key),
                        ("startDate", &start_date.to_string()),
                        ("endDate", &end_date.to_string()),
                    ])
                    .send()
                    .await?
                    .json::<Value>()
                    .await
            },
            3,
        )
        .await
    }
}

impl Default for NasaClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Generic retry logic with exponential backoff
async fn fetch_with_retry<F, Fut, T>(f: F, max_retries: usize) -> Result<T, reqwest::Error>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, reqwest::Error>>,
{
    let mut delay = 1;
    for attempt in 1..=max_retries {
        match f().await {
            Ok(result) => {
                if attempt > 1 {
                    info!("Request succeeded on retry attempt {}/{}", attempt, max_retries);
                }
                return Ok(result);
            }
            Err(e) if attempt < max_retries => {
                warn!(
                    "Request failed, retry attempt {}/{}: {:?}",
                    attempt, max_retries, e
                );
                tokio::time::sleep(Duration::from_secs(delay)).await;
                delay *= 2;
            }
            Err(e) => {
                error!("Request failed after {} attempts: {:?}", max_retries, e);
                return Err(e);
            }
        }
    }
    unreachable!()
}
