// ABOUTME: Data retention service for cleaning up old observations and maintaining database health
// ABOUTME: Handles periodic deletion of old data, VACUUM operations, and database maintenance

#![allow(dead_code)]

use crate::error::Result;
use sqlx::SqlitePool;
use std::time::Duration;
use tokio::time::{interval, Instant};
use tracing::{debug, error, info};

/// Configuration for data retention
#[derive(Debug, Clone)]
pub struct RetentionConfig {
    /// How often to run retention cleanup (default: 24 hours)
    pub cleanup_interval: Duration,
    /// How long to keep detailed observations (default: 30 days)
    pub observation_retention_days: u32,
    /// Whether to run VACUUM after cleanup (default: true)
    pub vacuum_after_cleanup: bool,
    /// Maximum number of observations to delete in one batch (default: 10000)
    pub max_delete_batch_size: u32,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            cleanup_interval: Duration::from_secs(24 * 60 * 60), // 24 hours
            observation_retention_days: 30,
            vacuum_after_cleanup: true,
            max_delete_batch_size: 10000,
        }
    }
}

/// Retention service that runs periodic cleanup
pub struct RetentionService {
    pool: SqlitePool,
    config: RetentionConfig,
}

impl RetentionService {
    /// Create new retention service
    pub fn new(pool: SqlitePool, config: RetentionConfig) -> Self {
        Self { pool, config }
    }

    /// Start the retention service background task
    pub async fn run(&self) {
        let mut interval = interval(self.config.cleanup_interval);

        info!(
            "Starting retention service, cleanup every {} seconds, keeping {} days of observations",
            self.config.cleanup_interval.as_secs(),
            self.config.observation_retention_days
        );

        loop {
            interval.tick().await;

            let start = Instant::now();
            match self.run_cleanup().await {
                Ok(deleted_count) => {
                    let duration = start.elapsed();
                    info!(
                        "Retention cleanup completed: deleted {} observations in {:.2}s",
                        deleted_count,
                        duration.as_secs_f64()
                    );

                    // Record metrics
                    let metrics = crate::metrics::AppMetrics::global();
                    metrics.record_db_operation("retention_cleanup", duration.as_millis() as f64);
                }
                Err(e) => {
                    error!("Retention cleanup failed: {}", e);
                }
            }
        }
    }

    /// Run a single cleanup cycle
    pub async fn run_cleanup(&self) -> Result<u64> {
        let cutoff_ms = self.calculate_cutoff_timestamp();

        debug!(
            "Running retention cleanup with cutoff timestamp: {}",
            cutoff_ms
        );

        // Count how many records will be deleted
        let count_to_delete = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM aircraft_observations WHERE ts_ms < ?",
        )
        .bind(cutoff_ms)
        .fetch_one(&self.pool)
        .await?;

        if count_to_delete == 0 {
            debug!("No observations to clean up");
            return Ok(0);
        }

        info!(
            "Found {} observations older than {} days to clean up",
            count_to_delete, self.config.observation_retention_days
        );

        // Delete in batches to avoid locking the database for too long
        let mut total_deleted = 0u64;
        let batch_size = self.config.max_delete_batch_size as i64;

        loop {
            let deleted_in_batch = sqlx::query(
                "DELETE FROM aircraft_observations
                 WHERE id IN (
                     SELECT id FROM aircraft_observations
                     WHERE ts_ms < ?
                     LIMIT ?
                 )",
            )
            .bind(cutoff_ms)
            .bind(batch_size)
            .execute(&self.pool)
            .await?
            .rows_affected();

            total_deleted += deleted_in_batch;

            if deleted_in_batch == 0 {
                break; // No more rows to delete
            }

            debug!(
                "Deleted {} observations in batch, total deleted: {}",
                deleted_in_batch, total_deleted
            );

            // Small delay between batches to allow other operations
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        // Run VACUUM if configured
        if self.config.vacuum_after_cleanup && total_deleted > 0 {
            info!(
                "Running VACUUM to reclaim space after deleting {} observations",
                total_deleted
            );

            let vacuum_start = Instant::now();
            sqlx::query("VACUUM").execute(&self.pool).await?;

            let vacuum_duration = vacuum_start.elapsed();
            info!("VACUUM completed in {:.2}s", vacuum_duration.as_secs_f64());
        }

        Ok(total_deleted)
    }

    /// Calculate the cutoff timestamp for retention
    fn calculate_cutoff_timestamp(&self) -> i64 {
        let now = chrono::Utc::now().timestamp_millis();
        let retention_ms = self.config.observation_retention_days as i64 * 24 * 60 * 60 * 1000;
        now - retention_ms
    }

    /// Run cleanup once immediately (useful for testing and manual operations)
    pub async fn run_once(&self) -> Result<u64> {
        self.run_cleanup().await
    }
}

/// Graceful shutdown handler
pub struct ShutdownHandler {
    shutdown_rx: tokio::sync::broadcast::Receiver<()>,
}

impl ShutdownHandler {
    /// Create new shutdown handler
    pub fn new() -> (Self, tokio::sync::broadcast::Sender<()>) {
        let (tx, rx) = tokio::sync::broadcast::channel(1);
        (Self { shutdown_rx: rx }, tx)
    }

    /// Wait for shutdown signal
    pub async fn wait_for_shutdown(&mut self) {
        let _ = self.shutdown_rx.recv().await;
        info!("Shutdown signal received");
    }
}

/// Set up Ctrl+C handler for graceful shutdown
pub fn setup_ctrl_c_handler(shutdown_tx: tokio::sync::broadcast::Sender<()>) {
    tokio::spawn(async move {
        if let Err(err) = tokio::signal::ctrl_c().await {
            error!("Failed to listen for Ctrl+C: {}", err);
            return;
        }

        info!("Received Ctrl+C, initiating graceful shutdown...");

        if let Err(err) = shutdown_tx.send(()) {
            error!("Failed to send shutdown signal: {}", err);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::AircraftObservation;
    use crate::store::connect_and_migrate;
    use std::time::Duration;
    use tempfile::TempDir;

    async fn create_test_pool() -> (SqlitePool, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = connect_and_migrate(db_path.to_str().unwrap(), false)
            .await
            .unwrap();
        (pool, temp_dir)
    }

    #[tokio::test]
    async fn test_retention_cleanup() {
        let (pool, _temp_dir) = create_test_pool().await;

        // Insert some old and new observations
        let now = chrono::Utc::now().timestamp_millis();
        let old_ts = now - (35 * 24 * 60 * 60 * 1000); // 35 days ago
        let recent_ts = now - (10 * 24 * 60 * 60 * 1000); // 10 days ago

        let old_obs = AircraftObservation {
            id: None,
            ts_ms: old_ts,
            hex: "OLD123".to_string(),
            flight: Some("OLD001".to_string()),
            lat: Some(40.0),
            lon: Some(-74.0),
            altitude: Some(30000),
            gs: Some(400.0),
            rssi: Some(-45.0),
            msg_count_total: Some(1000),
            raw_json: "{}".to_string(),
            msg_rate_hz: None,
        };

        let recent_obs = AircraftObservation {
            id: None,
            ts_ms: recent_ts,
            hex: "NEW123".to_string(),
            flight: Some("NEW001".to_string()),
            lat: Some(41.0),
            lon: Some(-75.0),
            altitude: Some(35000),
            gs: Some(450.0),
            rssi: Some(-40.0),
            msg_count_total: Some(2000),
            raw_json: "{}".to_string(),
            msg_rate_hz: None,
        };

        // Insert observations
        crate::store::observations::insert_observations(&pool, &[old_obs, recent_obs])
            .await
            .unwrap();

        // Verify we have 2 observations
        let count_before =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM aircraft_observations")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count_before, 2);

        // Create retention service with 30-day retention
        let config = RetentionConfig {
            cleanup_interval: Duration::from_secs(1), // Not used in manual cleanup
            observation_retention_days: 30,
            vacuum_after_cleanup: true,
            max_delete_batch_size: 1000,
        };

        let retention_service = RetentionService::new(pool.clone(), config);

        // Run cleanup
        let deleted_count = retention_service.run_once().await.unwrap();
        assert_eq!(deleted_count, 1, "Should delete 1 old observation");

        // Verify only 1 observation remains
        let count_after =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM aircraft_observations")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count_after, 1);

        // Verify the remaining observation is the recent one
        let remaining_hex =
            sqlx::query_scalar::<_, String>("SELECT hex FROM aircraft_observations")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(remaining_hex, "NEW123");
    }

    #[tokio::test]
    async fn test_retention_no_old_data() {
        let (pool, _temp_dir) = create_test_pool().await;

        // Insert only recent observation
        let now = chrono::Utc::now().timestamp_millis();
        let recent_ts = now - (10 * 24 * 60 * 60 * 1000); // 10 days ago

        let recent_obs = AircraftObservation {
            id: None,
            ts_ms: recent_ts,
            hex: "RECENT".to_string(),
            flight: Some("RCT001".to_string()),
            lat: Some(42.0),
            lon: Some(-76.0),
            altitude: Some(32000),
            gs: Some(420.0),
            rssi: Some(-42.0),
            msg_count_total: Some(1500),
            raw_json: "{}".to_string(),
            msg_rate_hz: None,
        };

        crate::store::observations::insert_observations(&pool, &[recent_obs])
            .await
            .unwrap();

        let config = RetentionConfig::default();
        let retention_service = RetentionService::new(pool.clone(), config);

        // Run cleanup - should delete nothing
        let deleted_count = retention_service.run_once().await.unwrap();
        assert_eq!(deleted_count, 0, "Should delete no recent observations");

        // Verify observation still exists
        let count_after =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM aircraft_observations")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count_after, 1);
    }

    #[tokio::test]
    async fn test_shutdown_handler() {
        let (mut handler, tx) = ShutdownHandler::new();

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            let _ = tx.send(());
        });

        // This should complete when the signal is sent
        handler.wait_for_shutdown().await;
    }
}
