// adminx-core/src/error.rs
use std::fmt;

/// Framework-neutral error. Adapters map this onto their own response type.
#[derive(Debug, Clone)]
pub enum CoreError {
    NotFound,
    BadRequest(String),
    Unauthorized,
    Forbidden,
    Internal(String),
}

impl CoreError {
    /// HTTP status code this error maps to.
    pub fn status(&self) -> u16 {
        match self {
            CoreError::NotFound => 404,
            CoreError::BadRequest(_) => 400,
            CoreError::Unauthorized => 401,
            CoreError::Forbidden => 403,
            CoreError::Internal(_) => 500,
        }
    }

    pub fn message(&self) -> String {
        match self {
            CoreError::NotFound => "Not Found".to_string(),
            CoreError::BadRequest(m) => format!("Bad Request: {m}"),
            CoreError::Unauthorized => "Unauthorized".to_string(),
            CoreError::Forbidden => "Forbidden".to_string(),
            CoreError::Internal(m) => format!("Internal Server Error: {m}"),
        }
    }
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message())
    }
}

impl std::error::Error for CoreError {}
