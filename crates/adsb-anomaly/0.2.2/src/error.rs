// ABOUTME: Error types and Result alias for the ADS-B anomaly detection system
// ABOUTME: Uses thiserror for ergonomic error handling with proper error chains

use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)] // Foundation types will be used in later prompts
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("HTTP server error: {0}")]
    Http(#[from] axum::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid hex code: {hex}")]
    InvalidHex { hex: String },

    #[error("Invalid aircraft data: {reason}")]
    InvalidAircraftData { reason: String },

    #[error("Anomaly detection error: {reason}")]
    AnomalyDetection { reason: String },

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Internal error: {message}")]
    Internal { message: String },

    #[error("Generic error: {0}")]
    Generic(String),
}

#[allow(dead_code)] // Will be used in later prompts
pub type Result<T> = std::result::Result<T, Error>;

impl From<Box<dyn std::error::Error + Send + Sync>> for Error {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Error::Generic(err.to_string())
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Generic(err.to_string())
    }
}
