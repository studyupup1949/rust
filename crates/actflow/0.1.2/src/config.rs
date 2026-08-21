//! Configuration types for Actflow engine.
//!
//! Configuration can be loaded from TOML files or created programmatically.

use std::{fs, path::Path};

use serde::Deserialize;

/// Main configuration for the Actflow engine.
///
/// # Example TOML
///
/// ```toml
/// async_worker_thread_number = 16
///
/// [store]
/// store_type = "postgres"
///
/// [store.postgres]
/// database_url = "postgresql://user:pass@localhost/db"
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Storage backend configuration.
    pub store: StoreConfig,
    /// Number of async worker threads (range: 1-32768, default: 16).
    pub async_worker_thread_number: u16,
}

/// Storage backend configuration.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct StoreConfig {
    /// Type of storage backend to use.
    pub store_type: StoreType,
    /// PostgreSQL configuration (required when store_type is Postgres).
    pub postgres: Option<PostgresConfig>,
}

/// Available storage backend types.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StoreType {
    /// In-memory storage (for testing, data is lost on restart).
    #[default]
    Mem,
    /// PostgreSQL database (for production, persistent storage).
    Postgres,
}

/// PostgreSQL connection configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct PostgresConfig {
    /// PostgreSQL connection URL.
    pub database_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            store: StoreConfig::default(),
            async_worker_thread_number: 16,
        }
    }
}

impl Config {
    /// Loads configuration from a TOML file.
    ///
    /// # Panics
    ///
    /// Panics if the file cannot be read or parsed.
    pub fn create<T: AsRef<Path>>(path: T) -> Self {
        let data = fs::read_to_string(path.as_ref()).expect(&format!("failed to load config file {:?}", path.as_ref()));

        Self::load_from_str(data.as_str())
    }

    /// Parses configuration from a TOML string.
    ///
    /// # Panics
    ///
    /// Panics if the string cannot be parsed as valid TOML.
    pub fn load_from_str(toml_str: &str) -> Self {
        let config = toml::from_str::<Config>(toml_str).expect("failed to parse the toml str");
        config
    }
}

#[cfg(test)]
mod test {
    use crate::{Config, StoreType};

    #[test]
    fn test_config_deserialize() {
        let toml_str = r#"
        async_worker_thread_number = 10
        [store]
        store_type = "postgres"

        [store.postgres]
        database_url = "postgresql://postgres:postgres@localhost:5432/postgres"
        "#;
        let config = Config::load_from_str(toml_str);
        assert_eq!(config.async_worker_thread_number, 10);
        assert_eq!(config.store.store_type, StoreType::Postgres);
        assert_eq!(
            config.store.postgres.unwrap().database_url,
            "postgresql://postgres:postgres@localhost:5432/postgres"
        );
    }
}
