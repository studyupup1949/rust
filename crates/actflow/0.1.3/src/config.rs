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
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Number of async worker threads (range: 1-32768, default: 16).
    pub async_worker_thread_number: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
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
