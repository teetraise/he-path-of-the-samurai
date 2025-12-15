use sqlx::{PgPool, Row};
use std::time::Duration;
use tracing::{error, info, warn};

use crate::config::Config;
use crate::services::{IssService, NasaService};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub redis: redis::aio::ConnectionManager,
    pub config: Config,
}

/// Start background tasks with PostgreSQL Advisory Locks
pub fn start_background_tasks(state: AppState) {
    info!("Starting background tasks");

    // ISS position sync task
    spawn_iss_sync_task(state.clone());

    // OSDR sync task
    spawn_osdr_sync_task(state.clone());

    // APOD sync task
    spawn_apod_sync_task(state.clone());

    // NEO sync task
    spawn_neo_sync_task(state.clone());

    // DONKI FLR sync task
    spawn_donki_flr_sync_task(state.clone());

    // DONKI CME sync task
    spawn_donki_cme_sync_task(state.clone());

    // SpaceX sync task
    spawn_spacex_sync_task(state);

    info!("All background tasks started");
}

fn spawn_iss_sync_task(state: AppState) {
    tokio::spawn(async move {
        let lock_id = 12345_i64; // Unique lock ID for ISS
        let interval = state.config.iss_sync_interval;

        info!(
            "ISS sync task started with interval {} seconds",
            interval
        );

        loop {
            // Try to acquire PostgreSQL advisory lock
            match try_acquire_lock(&state.pool, lock_id).await {
                Ok(true) => {
                    info!("Acquired advisory lock for ISS sync (lock_id={})", lock_id);

                    // Perform work
                    let mut redis = state.redis.clone();
                    match IssService::fetch_and_store(&state.pool, &mut redis, &state.config).await
                    {
                        Ok(_) => info!("ISS sync completed successfully"),
                        Err(e) => error!("ISS sync failed: {:?}", e),
                    }

                    // Release lock
                    if let Err(e) = release_lock(&state.pool, lock_id).await {
                        error!("Failed to release ISS advisory lock: {:?}", e);
                    }
                }
                Ok(false) => {
                    warn!("Could not acquire advisory lock for ISS sync, skipping this cycle");
                }
                Err(e) => {
                    error!("Error checking advisory lock for ISS: {:?}", e);
                }
            }

            tokio::time::sleep(Duration::from_secs(interval)).await;
        }
    });
}

fn spawn_osdr_sync_task(state: AppState) {
    tokio::spawn(async move {
        let lock_id = 12346_i64; // Unique lock ID for OSDR
        let interval = state.config.osdr_sync_interval;

        info!(
            "OSDR sync task started with interval {} seconds",
            interval
        );

        loop {
            match try_acquire_lock(&state.pool, lock_id).await {
                Ok(true) => {
                    info!("Acquired advisory lock for OSDR sync (lock_id={})", lock_id);

                    let mut redis = state.redis.clone();
                    match NasaService::fetch_and_store_osdr(
                        &state.pool,
                        &mut redis,
                        &state.config,
                    )
                    .await
                    {
                        Ok(count) => info!("OSDR sync completed, processed {} items", count),
                        Err(e) => error!("OSDR sync failed: {:?}", e),
                    }

                    if let Err(e) = release_lock(&state.pool, lock_id).await {
                        error!("Failed to release OSDR advisory lock: {:?}", e);
                    }
                }
                Ok(false) => {
                    warn!("Could not acquire advisory lock for OSDR sync, skipping this cycle");
                }
                Err(e) => {
                    error!("Error checking advisory lock for OSDR: {:?}", e);
                }
            }

            tokio::time::sleep(Duration::from_secs(interval)).await;
        }
    });
}

fn spawn_apod_sync_task(state: AppState) {
    tokio::spawn(async move {
        let lock_id = 12347_i64;
        let interval = state.config.apod_sync_interval;

        info!(
            "APOD sync task started with interval {} seconds",
            interval
        );

        loop {
            match try_acquire_lock(&state.pool, lock_id).await {
                Ok(true) => {
                    info!("Acquired advisory lock for APOD sync (lock_id={})", lock_id);

                    match NasaService::fetch_apod(&state.pool, &state.config).await {
                        Ok(_) => info!("APOD sync completed successfully"),
                        Err(e) => error!("APOD sync failed: {:?}", e),
                    }

                    if let Err(e) = release_lock(&state.pool, lock_id).await {
                        error!("Failed to release APOD advisory lock: {:?}", e);
                    }
                }
                Ok(false) => {
                    warn!("Could not acquire advisory lock for APOD sync, skipping this cycle");
                }
                Err(e) => {
                    error!("Error checking advisory lock for APOD: {:?}", e);
                }
            }

            tokio::time::sleep(Duration::from_secs(interval)).await;
        }
    });
}

fn spawn_neo_sync_task(state: AppState) {
    tokio::spawn(async move {
        let lock_id = 12348_i64;
        let interval = state.config.neo_sync_interval;

        info!("NEO sync task started with interval {} seconds", interval);

        loop {
            match try_acquire_lock(&state.pool, lock_id).await {
                Ok(true) => {
                    info!("Acquired advisory lock for NEO sync (lock_id={})", lock_id);

                    match NasaService::fetch_neo(&state.pool, &state.config).await {
                        Ok(_) => info!("NEO sync completed successfully"),
                        Err(e) => error!("NEO sync failed: {:?}", e),
                    }

                    if let Err(e) = release_lock(&state.pool, lock_id).await {
                        error!("Failed to release NEO advisory lock: {:?}", e);
                    }
                }
                Ok(false) => {
                    warn!("Could not acquire advisory lock for NEO sync, skipping this cycle");
                }
                Err(e) => {
                    error!("Error checking advisory lock for NEO: {:?}", e);
                }
            }

            tokio::time::sleep(Duration::from_secs(interval)).await;
        }
    });
}

fn spawn_donki_flr_sync_task(state: AppState) {
    tokio::spawn(async move {
        let lock_id = 12349_i64;
        let interval = state.config.donki_sync_interval;

        info!(
            "DONKI FLR sync task started with interval {} seconds",
            interval
        );

        loop {
            match try_acquire_lock(&state.pool, lock_id).await {
                Ok(true) => {
                    info!(
                        "Acquired advisory lock for DONKI FLR sync (lock_id={})",
                        lock_id
                    );

                    match NasaService::fetch_donki_flr(&state.pool, &state.config).await {
                        Ok(_) => info!("DONKI FLR sync completed successfully"),
                        Err(e) => error!("DONKI FLR sync failed: {:?}", e),
                    }

                    if let Err(e) = release_lock(&state.pool, lock_id).await {
                        error!("Failed to release DONKI FLR advisory lock: {:?}", e);
                    }
                }
                Ok(false) => {
                    warn!(
                        "Could not acquire advisory lock for DONKI FLR sync, skipping this cycle"
                    );
                }
                Err(e) => {
                    error!("Error checking advisory lock for DONKI FLR: {:?}", e);
                }
            }

            tokio::time::sleep(Duration::from_secs(interval)).await;
        }
    });
}

fn spawn_donki_cme_sync_task(state: AppState) {
    tokio::spawn(async move {
        let lock_id = 12350_i64;
        let interval = state.config.donki_sync_interval;

        info!(
            "DONKI CME sync task started with interval {} seconds",
            interval
        );

        loop {
            match try_acquire_lock(&state.pool, lock_id).await {
                Ok(true) => {
                    info!(
                        "Acquired advisory lock for DONKI CME sync (lock_id={})",
                        lock_id
                    );

                    match NasaService::fetch_donki_cme(&state.pool, &state.config).await {
                        Ok(_) => info!("DONKI CME sync completed successfully"),
                        Err(e) => error!("DONKI CME sync failed: {:?}", e),
                    }

                    if let Err(e) = release_lock(&state.pool, lock_id).await {
                        error!("Failed to release DONKI CME advisory lock: {:?}", e);
                    }
                }
                Ok(false) => {
                    warn!(
                        "Could not acquire advisory lock for DONKI CME sync, skipping this cycle"
                    );
                }
                Err(e) => {
                    error!("Error checking advisory lock for DONKI CME: {:?}", e);
                }
            }

            tokio::time::sleep(Duration::from_secs(interval)).await;
        }
    });
}

fn spawn_spacex_sync_task(state: AppState) {
    tokio::spawn(async move {
        let lock_id = 12351_i64;
        let interval = state.config.spacex_sync_interval;

        info!(
            "SpaceX sync task started with interval {} seconds",
            interval
        );

        loop {
            match try_acquire_lock(&state.pool, lock_id).await {
                Ok(true) => {
                    info!(
                        "Acquired advisory lock for SpaceX sync (lock_id={})",
                        lock_id
                    );

                    match NasaService::fetch_spacex(&state.pool, &state.config).await {
                        Ok(_) => info!("SpaceX sync completed successfully"),
                        Err(e) => error!("SpaceX sync failed: {:?}", e),
                    }

                    if let Err(e) = release_lock(&state.pool, lock_id).await {
                        error!("Failed to release SpaceX advisory lock: {:?}", e);
                    }
                }
                Ok(false) => {
                    warn!("Could not acquire advisory lock for SpaceX sync, skipping this cycle");
                }
                Err(e) => {
                    error!("Error checking advisory lock for SpaceX: {:?}", e);
                }
            }

            tokio::time::sleep(Duration::from_secs(interval)).await;
        }
    });
}

/// Try to acquire PostgreSQL advisory lock
async fn try_acquire_lock(pool: &PgPool, lock_id: i64) -> Result<bool, sqlx::Error> {
    let row = sqlx::query("SELECT pg_try_advisory_lock($1) as acquired")
        .bind(lock_id)
        .fetch_one(pool)
        .await?;

    Ok(row.get("acquired"))
}

/// Release PostgreSQL advisory lock
async fn release_lock(pool: &PgPool, lock_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT pg_advisory_unlock($1)")
        .bind(lock_id)
        .execute(pool)
        .await?;

    Ok(())
}
