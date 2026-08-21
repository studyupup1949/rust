//! ACE SDK Core - Rust client for the ACE API
//!
//! This crate provides a complete Rust implementation of the ACE (Agentic Context Engineering)
//! client SDK, including HTTP client, SQLite caching, device authentication, and configuration
//! management.
//!
//! # Example
//!
//! ```rust,no_run
//! use ace_sdk_core::{AceClient, AceClientOptions, AceConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), ace_sdk_core::AceError> {
//!     let config = AceConfig {
//!         server_url: "https://ace-api.code-engine.app".to_string(),
//!         api_token: "ace_user_xxx".to_string(),
//!         project_id: "my-project".to_string(),
//!         cache_ttl_minutes: 120,
//!         ..Default::default()
//!     };
//!
//!     let client = AceClient::new(config, AceClientOptions::default())?;
//!     let playbook = client.get_playbook(false).await?;
//!     println!("Total patterns: {}", playbook.total_bullets);
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod cache;
pub mod client;
pub mod config;
pub mod devices;
pub mod errors;
pub mod logger;
pub mod projects;
pub mod services;
pub mod types;
pub mod usage;
pub mod utils;

// Re-export primary types at crate root
pub use client::AceClient;
pub use client::AceClientOptions;
pub use config::{load_config, ConfigOverrides};
pub use errors::AceError;
pub use logger::{ILogger, NoopLogger};
pub use types::*;
pub use usage::{
    UsageBucket, UsageGranularity, UsageHistoryBucket, UsageHistoryGranularity,
    UsageHistoryResponse, UsageHistoryTotals, UsageHistoryWindow, UsageWindow,
};

// ACE 1.5 graph cache — re-exported so callers can use it without the cache module path.
pub use cache::GraphCache;
