use rust_iss::{routes::create_router, services::start_background_tasks, AppState, Config};
use sqlx::postgres::PgPoolOptions;
use redis::Client as RedisClient;
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use crate::repo::{IssRepo, OsdrRepo, SpaceRepo};

mod repo {
    pub use rust_iss::repo::*;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Load configuration
    let config = Config::from_env()?;

    // Connect to PostgreSQL with retry logic
    tracing::info!("Connecting to PostgreSQL at {}...", &config.database_url);
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect_timeout(std::time::Duration::from_secs(10))
        .connect(&config.database_url)
        .await
        .map_err(|e| {
            tracing::error!("Failed to connect to PostgreSQL: {:?}", e);
            e
        })?;

    tracing::info!("Connected to PostgreSQL");

    // Connect to Redis
    let redis_client = RedisClient::open(config.redis_url.clone())?;
    let redis = redis::aio::ConnectionManager::new(redis_client).await?;

    tracing::info!("Connected to Redis");

    // Create application state
    let state = AppState {
        pool: pool.clone(),
        redis,
        config: config.clone(),
    };

    // Initialize database tables
    init_db(&pool).await?;

    // Start background tasks
    start_background_tasks(state.clone());

    // Create router
    let app = create_router(state);

    // Start server
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

/// Initialize database tables
async fn init_db(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    tracing::info!("Initializing database tables");

    // Initialize ISS tables
    let iss_repo = IssRepo::new(pool.clone());
    iss_repo.init_tables().await?;
    tracing::info!("ISS tables initialized");

    // Initialize OSDR tables
    let osdr_repo = OsdrRepo::new(pool.clone());
    osdr_repo.init_tables().await?;
    tracing::info!("OSDR tables initialized");

    // Initialize Space cache tables
    let space_repo = SpaceRepo::new(pool.clone());
    space_repo.init_tables().await?;
    tracing::info!("Space cache tables initialized");

    tracing::info!("All database tables initialized successfully");

    Ok(())
}
