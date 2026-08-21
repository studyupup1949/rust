use thiserror::Error;

#[derive(Debug, Error)]
pub enum AbpilotError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("Authentication failed: {0}")]
    AuthError(String),
    
    #[error("Invalid signature")]
    SignatureError,
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Insufficient balance")]
    InsufficientBalance,
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, AbpilotError>;
