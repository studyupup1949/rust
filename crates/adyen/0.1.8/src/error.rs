use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ApiError {
    Other {
        status: u16,
        error_code: String,
        message: String,
        error_type: String,
        psp_reference: String,
    },
}

impl From<(u16, String, String, String, String)> for ApiError {
    #[track_caller]
    fn from(err: (u16, String, String, String, String)) -> Self {
        let (status, error_code, message, error_type, psp_reference) = err;
        match error_code.as_str() {
            _ => ApiError::Other {
                status,
                error_code,
                message,
                error_type,
                psp_reference,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Error {
    Unspecified(String),
    // ParseError(String),
    SerializationError(String),
    NetworkError(String),
    ApiError(ApiError),
    // Throttling,
    // ChecksumValidationError,
    ConversionError(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            Error::Unspecified(g) => g,
            // Error::ParseError(g) => g,
            Error::SerializationError(g) => g,
            Error::NetworkError(g) => g,
            Error::ApiError(err) => match err {
                ApiError::Other { message, .. } => message,
            },
            // Error::Throttling => "throttling",
            // Error::ChecksumValidationError => "failed checksum validation",
            Error::ConversionError(g) => g,
        };
        write!(f, "{}", text)
    }
}
