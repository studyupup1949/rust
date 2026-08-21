//! Configuration management for Aang

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::path::PathBuf;
use std::str::FromStr;

/// Main configuration for Aang
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Solana network configuration
    pub network: NetworkConfig,
    /// Operator configuration
    pub operator: OperatorConfig,
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
    /// Alert configuration
    pub alerts: AlertConfig,
    /// Storage configuration
    pub storage: StorageConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
            operator: OperatorConfig::default(),
            monitoring: MonitoringConfig::default(),
            alerts: AlertConfig::default(),
            storage: StorageConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// RPC endpoint URL
    pub rpc_url: String,
    /// WebSocket endpoint URL (for subscriptions)
    pub ws_url: Option<String>,
    /// Commitment level
    pub commitment: String,
    /// Request timeout in seconds
    pub timeout_secs: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://api.devnet.solana.com".to_string(),
            ws_url: Some("wss://api.devnet.solana.com".to_string()),
            commitment: "confirmed".to_string(),
            timeout_secs: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorConfig {
    /// Operator's fee payer public key (Kora node signer)
    pub fee_payer: Option<String>,
    /// Treasury address where reclaimed rent is sent
    pub treasury: Option<String>,
    /// Path to keypair file for signing reclaim transactions
    pub keypair_path: Option<PathBuf>,
    /// Dry run mode - simulate but don't execute reclaims
    pub dry_run: bool,
}

impl Default for OperatorConfig {
    fn default() -> Self {
        Self {
            fee_payer: None,
            treasury: None,
            keypair_path: None,
            dry_run: true, // Safe default
        }
    }
}

impl OperatorConfig {
    pub fn fee_payer_pubkey(&self) -> Option<Pubkey> {
        self.fee_payer
            .as_ref()
            .and_then(|s| Pubkey::from_str(s).ok())
    }

    pub fn treasury_pubkey(&self) -> Option<Pubkey> {
        self.treasury
            .as_ref()
            .and_then(|s| Pubkey::from_str(s).ok())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Interval for periodic scans in seconds
    pub scan_interval_secs: u64,
    /// Enable automatic background scanning
    pub auto_scan: bool,
    /// Inactivity threshold in days before marking account as inactive
    pub inactivity_threshold_days: u64,
    /// Minimum lamports to consider for reclaim (filter dust)
    pub min_reclaim_lamports: u64,
    /// Maximum accounts to process per scan
    pub max_accounts_per_scan: usize,
    /// Programs to monitor (empty = all)
    pub program_filter: Vec<String>,
    /// Account whitelist (never reclaim these)
    pub whitelist: Vec<String>,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            scan_interval_secs: 300, // 5 minutes
            auto_scan: false,
            inactivity_threshold_days: 30,
            min_reclaim_lamports: 890_880, // ~minimum rent for basic account
            max_accounts_per_scan: 1000,
            program_filter: vec![],
            whitelist: vec![],
        }
    }
}

impl MonitoringConfig {
    pub fn is_whitelisted(&self, pubkey: &Pubkey) -> bool {
        let pubkey_str = pubkey.to_string();
        self.whitelist.iter().any(|w| w == &pubkey_str)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Enable alerts
    pub enabled: bool,
    /// Threshold for "large idle rent" alert in SOL
    pub large_idle_threshold_sol: f64,
    /// Number of idle accounts to trigger alert
    pub idle_count_threshold: usize,
    /// Alert on reclaim failures
    pub alert_on_failure: bool,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            large_idle_threshold_sol: 1.0, // 1 SOL
            idle_count_threshold: 10,
            alert_on_failure: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Path to data directory
    pub data_dir: Option<PathBuf>,
    /// Path to accounts database file
    pub accounts_file: Option<PathBuf>,
    /// Path to reclaims history file
    pub reclaims_file: Option<PathBuf>,
    /// Maximum log entries to keep
    pub max_log_entries: usize,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: None,
            accounts_file: None,
            reclaims_file: None,
            max_log_entries: 1000,
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let config: Config =
            toml::from_str(&content).with_context(|| "Failed to parse config file")?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(self).with_context(|| "Failed to serialize config")?;
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
        Ok(())
    }

    /// Get default config file path
    pub fn default_path() -> PathBuf {
        directories::ProjectDirs::from("com", "aang", "aang")
            .map(|dirs| dirs.config_dir().join("aang.toml"))
            .unwrap_or_else(|| PathBuf::from("aang.toml"))
    }

    /// Get default data directory
    pub fn default_data_dir() -> PathBuf {
        directories::ProjectDirs::from("com", "aang", "aang")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".aang"))
    }

    /// Generate example config content
    pub fn example_toml() -> String {
        r#"# Aang Configuration
# Kora Rent Reclaim Bot

[network]
# Solana RPC endpoint
rpc_url = "https://api.devnet.solana.com"
# WebSocket endpoint for subscriptions
ws_url = "wss://api.devnet.solana.com"
# Commitment level: processed, confirmed, finalized
commitment = "confirmed"
# Request timeout in seconds
timeout_secs = 30

[operator]
# Your Kora node's fee payer public key
# fee_payer = "YourKoraFeePayer..."
# Treasury address for reclaimed rent
# treasury = "YourTreasuryAddress..."
# Path to keypair file for signing transactions
# keypair_path = "/path/to/keypair.json"
# Dry run mode - simulate without executing
dry_run = true

[monitoring]
# Scan interval in seconds (default: 5 minutes)
scan_interval_secs = 300
# Enable automatic background scanning
auto_scan = false
# Days of inactivity before marking account inactive
inactivity_threshold_days = 30
# Minimum lamports to consider for reclaim
min_reclaim_lamports = 890880
# Maximum accounts per scan
max_accounts_per_scan = 1000
# Programs to monitor (empty = all)
program_filter = []
# Accounts to never reclaim
whitelist = []

[alerts]
# Enable alert system
enabled = true
# SOL threshold for large idle rent alert
large_idle_threshold_sol = 1.0
# Number of idle accounts to trigger alert
idle_count_threshold = 10
# Alert on reclaim failures
alert_on_failure = true

[storage]
# Data directory (default: platform-specific)
# data_dir = "/path/to/data"
# Maximum log entries to keep
max_log_entries = 1000
"#
        .to_string()
    }
}
