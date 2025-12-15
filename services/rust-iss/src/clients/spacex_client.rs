use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use tracing::{error, info, warn};

pub struct SpaceXClient {
    client: Client,
}

impl SpaceXClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Cassiopeia-ISS/1.0")
            .build()
            .expect("Failed to build reqwest client");

        Self { client }
    }

    /// Fetch next SpaceX launch
    pub async fn fetch_next_launch(&self) -> Result<Value, reqwest::Error> {
        let url = "https://api.spacexdata.com/v5/launches/next";
        info!("Fetching next SpaceX launch from: {}", url);

        fetch_with_retry(
            || async {
                self.client
                    .get(url)
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

impl Default for SpaceXClient {
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
