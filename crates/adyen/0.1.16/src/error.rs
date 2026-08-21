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

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        psp_reference: Option<String>,
    },
}

impl From<(u16, String, String, String, Option<String>)> for ApiError {
    #[track_caller]
    fn from(err: (u16, String, String, String, Option<String>)) -> Self {
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
    UnsupportedPaymentMethod,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let g;
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
            Error::UnsupportedPaymentMethod => {
                g = String::from("unsupported payment method");
                &g
            }
        };
        write!(f, "{}", text)
    }
}
