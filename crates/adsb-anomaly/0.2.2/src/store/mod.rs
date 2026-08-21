// ABOUTME: Database connection and migration management for SQLite
// ABOUTME: Sets up WAL mode for concurrent access and runs embedded migrations

pub mod observations;
pub mod sessions;

#[cfg(test)]
mod simple_test;

use crate::error::Result;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::str::FromStr;
use tracing::{info, warn};

pub async fn connect_and_migrate(db_path: &str, wal: bool) -> Result<SqlitePool> {
    info!("Connecting to SQLite database: {}", db_path);

    // Build connection options
    let mut options = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path))?
        .create_if_missing(true)
        .busy_timeout(std::time::Duration::from_secs(30));

    // Enable WAL mode if requested
    if wal {
        options = options.pragma("journal_mode", "WAL");
        info!("WAL mode enabled for concurrent access");
    }

    // Create connection pool
    let pool = SqlitePool::connect_with(options).await?;

    // Run migrations
    info!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("Database migrations completed successfully");

    // Verify WAL mode is active (if requested)
    if wal {
        verify_wal_mode(&pool).await?;
    }

    Ok(pool)
}

async fn verify_wal_mode(pool: &SqlitePool) -> Result<()> {
    let result: (String,) = sqlx::query_as("PRAGMA journal_mode")
        .fetch_one(pool)
        .await?;

    match result.0.to_lowercase().as_str() {
        "wal" => {
            info!("WAL mode confirmed active");
            Ok(())
        }
        mode => {
            warn!("Expected WAL mode but got: {}", mode);
            Ok(()) // Don't fail, just warn
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_connect_and_migrate() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let pool = connect_and_migrate(db_path.to_str().unwrap(), true)
            .await
            .unwrap();

        // Verify tables exist
        let table_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        // Should have 4 tables: aircraft_observations, aircraft_sessions, anomaly_detections, _sqlx_migrations
        assert_eq!(table_count.0, 4);
    }

    #[tokio::test]
    async fn test_wal_mode_verification() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_wal.db");

        let pool = connect_and_migrate(db_path.to_str().unwrap(), true)
            .await
            .unwrap();

        // Check journal mode
        let result: (String,) = sqlx::query_as("PRAGMA journal_mode")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(result.0.to_lowercase(), "wal");
    }

    #[tokio::test]
    async fn test_table_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_tables.db");

        let pool = connect_and_migrate(db_path.to_str().unwrap(), false)
            .await
            .unwrap();

        // Check that all expected tables exist
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        let table_names: Vec<String> = tables.into_iter().map(|t| t.0).collect();
        assert_eq!(
            table_names,
            vec![
                "_sqlx_migrations".to_string(),
                "aircraft_observations".to_string(),
                "aircraft_sessions".to_string(),
                "anomaly_detections".to_string()
            ]
        );
    }
}
