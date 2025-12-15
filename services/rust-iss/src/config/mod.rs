use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub nasa_api_key: String,
    pub nasa_osdr_url: String,
    pub iss_url: String,
    pub astronomy_api_id: String,
    pub astronomy_api_secret: String,

    // Sync intervals in seconds
    pub iss_sync_interval: u64,
    pub osdr_sync_interval: u64,
    pub apod_sync_interval: u64,
    pub neo_sync_interval: u64,
    pub donki_sync_interval: u64,
    pub spacex_sync_interval: u64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://redis:6379".to_string()),
            nasa_api_key: env::var("NASA_API_KEY")
                .unwrap_or_else(|_| "DEMO_KEY".to_string()),
            nasa_osdr_url: env::var("NASA_API_URL")
                .unwrap_or_else(|_| "https://visualization.osdr.nasa.gov/biodata/api/v2/datasets/?format=json".to_string()),
            iss_url: env::var("WHERE_ISS_URL")
                .unwrap_or_else(|_| "https://api.wheretheiss.at/v1/satellites/25544".to_string()),
            astronomy_api_id: env::var("ASTRO_APP_ID")
                .unwrap_or_default(),
            astronomy_api_secret: env::var("ASTRO_APP_SECRET")
                .unwrap_or_default(),

            iss_sync_interval: Self::env_u64("ISS_SYNC_INTERVAL", 120),
            osdr_sync_interval: Self::env_u64("OSDR_SYNC_INTERVAL", 600),
            apod_sync_interval: Self::env_u64("APOD_SYNC_INTERVAL", 43200), // 12h
            neo_sync_interval: Self::env_u64("NEO_SYNC_INTERVAL", 7200), // 2h
            donki_sync_interval: Self::env_u64("DONKI_SYNC_INTERVAL", 3600), // 1h
            spacex_sync_interval: Self::env_u64("SPACEX_SYNC_INTERVAL", 3600), // 1h
        })
    }

    fn env_u64(key: &str, default: u64) -> u64 {
        env::var(key)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(default)
    }
}
