use reqwest::StatusCode;
use thiserror::Error;
use url::ParseError as UrlParseError;

/// Error type for acme-dns-client.
#[derive(Debug, Error)]
pub enum Error {
    #[error("URL parse error: {0}")]
    Url(#[from] UrlParseError),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("server returned unexpected status {status}: {body}")]
    UnexpectedStatus { status: StatusCode, body: String },

    #[error("missing required environment variable {0}")]
    MissingEnv(&'static str),
}
