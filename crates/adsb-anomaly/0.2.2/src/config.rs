// ABOUTME: Configuration management with TOML file + env vars + CLI precedence
// ABOUTME: Defines strongly-typed config structs for all system components

use crate::circuit_breaker::CircuitBreakerConfig as CBConfig;
use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[cfg(test)]
use std::sync::Mutex;

#[cfg(test)]
static ENV_TEST_MUTEX: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdsbConfig {
    pub piaware_url: String,
    pub poll_interval_ms: u64,
    pub circuit_breaker: CircuitBreakerConfigSerde,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfigSerde {
    pub failure_threshold: u32,
    pub timeout_ms: i64,
    pub success_threshold: u32,
}

impl Default for AdsbConfig {
    fn default() -> Self {
        Self {
            piaware_url: "http://localhost:8080/dump1090-fa/data/aircraft.json".to_string(),
            poll_interval_ms: 1000,
            circuit_breaker: CircuitBreakerConfigSerde::default(),
        }
    }
}

impl Default for CircuitBreakerConfigSerde {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout_ms: 30_000, // 30 seconds
            success_threshold: 2,
        }
    }
}

impl From<CircuitBreakerConfigSerde> for CBConfig {
    fn from(config: CircuitBreakerConfigSerde) -> Self {
        CBConfig {
            failure_threshold: config.failure_threshold,
            timeout_ms: config.timeout_ms,
            success_threshold: config.success_threshold,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
    pub wal_mode: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: "adsb.db".to_string(),
            wal_mode: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    pub max_messages_per_second: f64,
    pub min_message_interval_ms: u64,
    pub max_session_gap_seconds: u64,
    pub min_rssi_units: f64,
    pub max_rssi_units: f64,
    pub suspicious_rssi_units: f64,
    pub suspicious_callsigns: Vec<String>,
    pub invalid_hex_patterns: Vec<String>,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            max_messages_per_second: 10.0,
            min_message_interval_ms: 50,
            max_session_gap_seconds: 600,
            min_rssi_units: -120.0,
            max_rssi_units: -10.0,
            suspicious_rssi_units: -20.0,
            suspicious_callsigns: vec![
                "TEST.*".to_string(),
                "FAKE.*".to_string(),
                "ANOM.*".to_string(),
            ],
            invalid_hex_patterns: vec![
                "000000".to_string(),
                "FFFFFF".to_string(),
                "AAAAAA".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertsConfig {
    pub confidence_threshold: f64,
    pub max_alerts_per_hour: u32,
    pub webhook_url: Option<String>,
}

impl Default for AlertsConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.7,
            max_alerts_per_hour: 100,
            webhook_url: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub port: u16,
    pub dashboard_title: String,
    pub map_center_lat: f64,
    pub map_center_lon: f64,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            dashboard_title: "ADS-B Anomaly Monitor".to_string(),
            map_center_lat: 41.8781, // Chicago area default
            map_center_lon: -87.6298,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub adsb: AdsbConfig,
    pub database: DatabaseConfig,
    pub analysis: AnalysisConfig,
    pub alerts: AlertsConfig,
    pub web: WebConfig,
}

impl AppConfig {
    pub fn load<P: AsRef<Path>>(config_path: Option<P>) -> Result<Self, ConfigError> {
        let mut builder = Config::builder()
            // Start with defaults
            .add_source(Config::try_from(&AppConfig::default())?);

        // Add TOML file - check provided path first, then default config.toml
        let config_file_loaded = if let Some(path) = config_path {
            if path.as_ref().exists() {
                builder = builder.add_source(File::from(path.as_ref()));
                true
            } else {
                false
            }
        } else {
            // Try default config.toml if no path provided
            if Path::new("config.toml").exists() {
                builder = builder.add_source(File::from(Path::new("config.toml")));
                true
            } else {
                false
            }
        };

        // Only warn if no config file was loaded
        if !config_file_loaded {
            eprintln!("Warning: No config.toml found, using defaults");
        }

        // Add environment variables with ADSB_ prefix
        builder = builder.add_source(
            Environment::with_prefix("ADSB")
                .prefix_separator("_")
                .separator("__"),
        );

        let config = builder.build()?;
        config.try_deserialize()
    }

    pub fn merge_cli_overrides(
        &mut self,
        _config_path_override: Option<String>,
        db_path_override: Option<String>,
        port_override: Option<u16>,
    ) {
        if let Some(db_path) = db_path_override {
            self.database.path = db_path;
        }
        if let Some(port) = port_override {
            self.web.port = port;
        }
        // config_path_override is handled during initial load
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.adsb.poll_interval_ms, 1000);
        assert_eq!(config.database.wal_mode, true);
        assert_eq!(config.web.port, 8080);
        assert_eq!(config.analysis.max_messages_per_second, 10.0);
        assert_eq!(config.alerts.confidence_threshold, 0.7);
    }

    #[test]
    fn test_load_from_toml() {
        let _lock = ENV_TEST_MUTEX.lock().unwrap();
        // Save and clear any environment variables that might interfere
        let original_path = env::var("ADSB_DATABASE__PATH").ok();
        let original_port = env::var("ADSB_WEB__PORT").ok();

        env::remove_var("ADSB_DATABASE__PATH");
        env::remove_var("ADSB_WEB__PORT");

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let toml_content = r#"
[adsb]
piaware_url = "http://test.com/data"
poll_interval_ms = 2000

[database]
path = "test.db"
wal_mode = false

[web]
port = 9090
dashboard_title = "Test Dashboard"
"#;

        fs::write(&config_path, toml_content).unwrap();

        let config = AppConfig::load(Some(&config_path)).unwrap();
        assert_eq!(config.adsb.piaware_url, "http://test.com/data");
        assert_eq!(config.adsb.poll_interval_ms, 2000);
        assert_eq!(config.database.path, "test.db");
        assert_eq!(config.database.wal_mode, false);
        assert_eq!(config.web.port, 9090);
        assert_eq!(config.web.dashboard_title, "Test Dashboard");

        // Restore original environment variables
        if let Some(path) = original_path {
            env::set_var("ADSB_DATABASE__PATH", path);
        }
        if let Some(port) = original_port {
            env::set_var("ADSB_WEB__PORT", port);
        }
    }

    #[test]
    fn test_env_override() {
        let _lock = ENV_TEST_MUTEX.lock().unwrap();
        use std::sync::atomic::{AtomicU16, Ordering};
        // Use a different starting number each time to avoid conflicts
        static TEST_COUNTER: AtomicU16 = AtomicU16::new(0);

        // Use unique port to avoid conflicts between parallel tests
        let test_num = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let test_port = 7777 + test_num;
        let test_path = format!("env_test_{}.db", test_port);

        // Clear any existing env vars first to ensure clean state
        let original_port = env::var("ADSB_WEB__PORT").ok();
        let original_path = env::var("ADSB_DATABASE__PATH").ok();

        env::remove_var("ADSB_WEB__PORT");
        env::remove_var("ADSB_DATABASE__PATH");

        // Set environment variable with unique values
        env::set_var("ADSB_WEB__PORT", test_port.to_string());
        env::set_var("ADSB_DATABASE__PATH", &test_path);

        let config = AppConfig::load::<String>(None).unwrap();
        assert_eq!(config.web.port, test_port);
        assert_eq!(config.database.path, test_path);

        // Restore original values or remove if they weren't set
        env::remove_var("ADSB_WEB__PORT");
        env::remove_var("ADSB_DATABASE__PATH");

        if let Some(port) = original_port {
            env::set_var("ADSB_WEB__PORT", port);
        }
        if let Some(path) = original_path {
            env::set_var("ADSB_DATABASE__PATH", path);
        }
    }

    #[test]
    fn test_cli_override() {
        let mut config = AppConfig::default();
        config.merge_cli_overrides(None, Some("cli_override.db".to_string()), Some(5555));

        assert_eq!(config.database.path, "cli_override.db");
        assert_eq!(config.web.port, 5555);
    }

    #[test]
    fn test_circuit_breaker_config_conversion() {
        let serde_config = CircuitBreakerConfigSerde {
            failure_threshold: 3,
            timeout_ms: 60_000,
            success_threshold: 1,
        };

        let cb_config: CBConfig = serde_config.into();
        assert_eq!(cb_config.failure_threshold, 3);
        assert_eq!(cb_config.timeout_ms, 60_000);
        assert_eq!(cb_config.success_threshold, 1);
    }

    #[test]
    fn test_default_circuit_breaker_config() {
        let config = AppConfig::default();
        assert_eq!(config.adsb.circuit_breaker.failure_threshold, 5);
        assert_eq!(config.adsb.circuit_breaker.timeout_ms, 30_000);
        assert_eq!(config.adsb.circuit_breaker.success_threshold, 2);
    }

    #[test]
    fn test_load_circuit_breaker_from_toml() {
        let _lock = ENV_TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config_cb.toml");

        let toml_content = r#"
[adsb]
piaware_url = "http://test.com/data"
poll_interval_ms = 2000

[adsb.circuit_breaker]
failure_threshold = 10
timeout_ms = 120000
success_threshold = 3

[database]
path = "test.db"
"#;

        fs::write(&config_path, toml_content).unwrap();

        let config = AppConfig::load(Some(&config_path)).unwrap();
        assert_eq!(config.adsb.circuit_breaker.failure_threshold, 10);
        assert_eq!(config.adsb.circuit_breaker.timeout_ms, 120_000);
        assert_eq!(config.adsb.circuit_breaker.success_threshold, 3);
    }
}
