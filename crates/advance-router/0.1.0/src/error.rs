use std::time::Duration;

/// Unified error type for all advance-router operations.
#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error from {provider} (status {status}): {message}")]
    Api {
        provider: String,
        status: u16,
        message: String,
        error_type: Option<String>,
        raw: serde_json::Value,
        retryable: bool,
    },

    #[error("Authentication error for {provider}: {message}")]
    Auth {
        provider: String,
        message: String,
    },

    #[error("Model not found: {model}")]
    ModelNotFound { model: String },

    #[error("Provider not configured: {provider}")]
    ProviderNotConfigured { provider: String },

    #[error("Rate limited by {provider}, retry after {retry_after:?}")]
    RateLimited {
        provider: String,
        retry_after: Option<Duration>,
    },

    #[error("Stream error: {0}")]
    Stream(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Max tool-call rounds exceeded ({rounds})")]
    MaxRoundsExceeded { rounds: usize },

    #[error("All fallback providers failed")]
    AllFallbacksFailed { errors: Vec<RouterError> },

    #[error("Timeout after {0:?}")]
    Timeout(Duration),
}

impl RouterError {
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Api { retryable, .. } => *retryable,
            Self::RateLimited { .. } => true,
            Self::Http(e) => e.is_timeout() || e.is_connect(),
            Self::Timeout(_) => true,
            _ => false,
        }
    }

    pub fn api_error(
        provider: impl Into<String>,
        status: u16,
        message: impl Into<String>,
        raw: serde_json::Value,
    ) -> Self {
        let retryable = matches!(status, 429 | 500 | 502 | 503 | 529);
        Self::Api {
            provider: provider.into(),
            status,
            message: message.into(),
            error_type: None,
            raw,
            retryable,
        }
    }
}
