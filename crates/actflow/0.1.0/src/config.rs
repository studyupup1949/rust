use std::{fs, path::Path};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// store config
    pub store: StoreConfig,
    /// number of async worker threads, range [1, 32768), defaults to 16
    pub async_worker_thread_number: u16,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct StoreConfig {
    /// store type
    pub store_type: StoreType,
    /// postgres config
    pub postgres: Option<PostgresConfig>,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StoreType {
    #[default]
    Mem,
    Postgres,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostgresConfig {
    /// postgres database url
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
    pub fn create<T: AsRef<Path>>(path: T) -> Self {
        let data = fs::read_to_string(path.as_ref()).expect(&format!("failed to load config file {:?}", path.as_ref()));

        Self::load_from_str(data.as_str())
    }

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
