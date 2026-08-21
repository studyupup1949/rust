use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Error {
    Unspecified(String),
    // ParseError(String),
    SerializationError(String),
    NetworkError(String),
    // ApiError(String, String, Option<String>, Option<String>, String),
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
            // Error::ApiError(_, _, _, _, g) => g,
            // Error::Throttling => "throttling",
            // Error::ChecksumValidationError => "failed checksum validation",
            Error::ConversionError(g) => g,
        };
        write!(f, "{}", text)
    }
}
