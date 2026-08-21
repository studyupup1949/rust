//! Common utilities for integration tests.

use adbc_core::{Database, Driver, Optionable};
use adbc_taos::{TaosDriver, TaosConnection};

/// Test configuration loaded from environment.
pub struct TestConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: Option<String>,
}

impl TestConfig {
    /// Loads test configuration from environment variables.
    ///
    /// Falls back to defaults if variables are not set.
    pub fn from_env() -> Self {
        // Try to load .env file first
        let _ = dotenvy::dotenv();

        Self {
            host: std::env::var("TAOS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("TAOS_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(6030),
            user: std::env::var("TAOS_USER").unwrap_or_else(|_| "root".to_string()),
            password: std::env::var("TAOS_PASSWORD").unwrap_or_else(|_| "taosdata".to_string()),
            database: std::env::var("TAOS_DATABASE").ok(),
        }
    }

    /// Builds the DSN connection string.
    pub fn dsn(&self) -> String {
        let mut dsn = format!("taos://{}:{}@{}:{}",
            self.user, self.password, self.host, self.port);
        if let Some(db) = &self.database {
            dsn.push('/');
            dsn.push_str(db);
        }
        dsn
    }
}

impl Default for TestConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

/// Creates a new test database and connection.
///
/// Returns the connection and a unique database name for testing.
pub fn create_test_connection() -> (TaosConnection, String) {
    let config = TestConfig::default();
    let mut driver = TaosDriver::default();

    let mut db = driver.new_database().expect("Failed to create database");
    db.set_option(
        adbc_core::options::OptionDatabase::Uri,
        adbc_core::options::OptionValue::String(config.dsn())
    ).expect("Failed to set URI");

    let conn = db.new_connection().expect("Failed to create connection");

    // Generate unique database name using timestamp and thread ID
    let db_name = format!("test_db_{}_{}", std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos());

    (conn, db_name)
}
