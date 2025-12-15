use chrono::{DateTime, Utc};
use rand::Rng;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Telemetry Generator starting...");

    // Load configuration from environment
    let csv_out_dir = env::var("CSV_OUT_DIR").unwrap_or_else(|_| "/data/csv".to_string());
    let period_sec = env::var("GEN_PERIOD_SEC")
        .unwrap_or_else(|_| "300".to_string())
        .parse::<u64>()
        .unwrap_or(300);

    let db_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        env::var("PGUSER").unwrap_or_else(|_| "cassiopeia_user".to_string()),
        env::var("PGPASSWORD").unwrap_or_else(|_| "cassiopeia_pass".to_string()),
        env::var("PGHOST").unwrap_or_else(|_| "db".to_string()),
        env::var("PGPORT").unwrap_or_else(|_| "5432".to_string()),
        env::var("PGDATABASE").unwrap_or_else(|_| "cassiopeia".to_string()),
    );

    // Connect to PostgreSQL
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    info!("Connected to PostgreSQL");

    // Main loop
    loop {
        match generate_and_insert(&pool, &csv_out_dir).await {
            Ok(()) => info!("Telemetry data generated successfully"),
            Err(e) => error!("Legacy error: {}", e),
        }

        tokio::time::sleep(Duration::from_secs(period_sec)).await;
    }
}

async fn generate_and_insert(pool: &sqlx::PgPool, csv_out_dir: &str) -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    // Generate timestamp
    let now: DateTime<Utc> = Utc::now();
    let ts_filename = now.format("%Y%m%d_%H%M%S").to_string();
    let filename = format!("telemetry_{}.csv", ts_filename);

    // Generate random telemetry data
    let voltage = rng.gen_range(3.2..12.6);
    let temp = rng.gen_range(-50.0..80.0);

    // Create CSV file
    std::fs::create_dir_all(csv_out_dir)?;
    let filepath = Path::new(csv_out_dir).join(&filename);
    let mut file = File::create(&filepath)?;

    // Write CSV header and data
    writeln!(file, "recorded_at,voltage,temp,source_file")?;
    writeln!(
        file,
        "{},{:.2},{:.2},{}",
        now.format("%Y-%m-%d %H:%M:%S"),
        voltage,
        temp,
        filename
    )?;

    info!("CSV file created: {:?}", filepath);

    // Insert into PostgreSQL
    sqlx::query(
        "INSERT INTO telemetry_legacy (recorded_at, voltage, temp, source_file) VALUES ($1, $2, $3, $4)"
    )
    .bind(now)
    .bind(voltage)
    .bind(temp)
    .bind(&filename)
    .execute(pool)
    .await?;

    info!("Data inserted into PostgreSQL");

    Ok(())
}
