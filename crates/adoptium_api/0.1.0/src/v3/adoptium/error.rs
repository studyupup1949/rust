//! Defines the [`Error`] type used across the Adoptium API client.
//!
//! This module centralizes all error variants that can occur when making requests.

/// Represents all possible errors returned by the Adoptium API client.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Returned when JSON deserialization fails.
    #[error("Serde error: {0}")]
    Serde(String),
    /// Returned when the HTTP client (`reqwest`) encounters an error.
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    /// Returned when a URL fails to parse.
    #[error("Url parse error: {0}")]
    Url(#[from] url::ParseError)
}
