//! Error types for the Accumulate Rust SDK

#![allow(missing_docs)]

use thiserror::Error;

/// Main error type for the Accumulate SDK
#[derive(Error, Debug)]
pub enum Error {
    #[error("Signature error: {0}")]
    Signature(#[from] SignatureError),

    #[error("Encoding error: {0}")]
    Encoding(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("RPC error: code={code}, message={message}")]
    Rpc { code: i32, message: String },

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("General error: {0}")]
    General(String),
}

/// Validation-specific errors for transaction bodies and headers
#[derive(Error, Debug, Clone)]
pub enum ValidationError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Required field missing: {0}")]
    RequiredFieldMissing(String),

    #[error("Invalid field value: {field} - {reason}")]
    InvalidFieldValue { field: String, reason: String },

    #[error("Amount must be positive: {0}")]
    InvalidAmount(String),

    #[error("Empty collection not allowed: {0}")]
    EmptyCollection(String),

    #[error("Invalid hash: expected {expected} bytes, got {actual}")]
    InvalidHash { expected: usize, actual: usize },

    #[error("Value out of range: {field} must be between {min} and {max}")]
    OutOfRange { field: String, min: String, max: String },

    #[error("Invalid token symbol: {0}")]
    InvalidTokenSymbol(String),

    #[error("Invalid precision: must be between 0 and 18, got {0}")]
    InvalidPrecision(u64),
}

/// Signature-specific errors
#[derive(Error, Debug, Clone)]
pub enum SignatureError {
    #[error("Invalid signature format")]
    InvalidFormat,

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("Unsupported signature type: {0}")]
    UnsupportedType(String),

    #[error("Invalid public key")]
    InvalidPublicKey,

    #[error("Invalid signature bytes")]
    InvalidSignature,

    #[error("Cryptographic error: {0}")]
    Crypto(String),
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::General(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::General(s.to_string())
    }
}

impl Error {
    /// Create an RPC error with code and message
    pub fn rpc(code: i32, message: String) -> Self {
        Error::Rpc { code, message }
    }
}