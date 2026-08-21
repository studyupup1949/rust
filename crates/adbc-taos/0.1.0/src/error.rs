//! Error types for ADBC-Taos driver.
//!
//! Provides error conversion between TDengine client errors and ADBC status codes.

use std::backtrace::Backtrace;
use std::fmt::{Display, Formatter};

/// Error type for ADBC-Taos operations.
///
/// Contains contextual information about the error including backtrace
/// for debugging in async contexts.
#[derive(Debug)]
pub struct TaosError {
    kind: ErrorKind,
    backtrace: Backtrace,
}

impl TaosError {
    /// Creates a new connection error.
    pub(crate) fn connection(msg: String) -> Self {
        Self {
            kind: ErrorKind::Connection(msg),
            backtrace: Backtrace::capture(),
        }
    }

    /// Creates a new query execution error.
    pub(crate) fn query(msg: String) -> Self {
        Self {
            kind: ErrorKind::Query(msg),
            backtrace: Backtrace::capture(),
        }
    }

    /// Creates a new type conversion error.
    pub(crate) fn conversion(msg: String) -> Self {
        Self {
            kind: ErrorKind::Conversion(msg),
            backtrace: Backtrace::capture(),
        }
    }

    /// Creates a new invalid option error.
    pub(crate) fn invalid_option(msg: String) -> Self {
        Self {
            kind: ErrorKind::InvalidOption(msg),
            backtrace: Backtrace::capture(),
        }
    }

    /// Returns the backtrace captured when this error was created.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    /// Returns true if this is a connection error.
    pub fn is_connection(&self) -> bool {
        matches!(self.kind, ErrorKind::Connection(_))
    }

    /// Returns true if this is a query error.
    pub fn is_query(&self) -> bool {
        matches!(self.kind, ErrorKind::Query(_))
    }

    /// Returns true if this is a conversion error.
    pub fn is_conversion(&self) -> bool {
        matches!(self.kind, ErrorKind::Conversion(_))
    }

    /// Returns the corresponding ADBC status code.
    pub fn adbc_status(&self) -> adbc_core::error::Status {
        match &self.kind {
            ErrorKind::Connection(_) => adbc_core::error::Status::Unknown,
            ErrorKind::Query(_) => adbc_core::error::Status::Unknown,
            ErrorKind::Conversion(_) => adbc_core::error::Status::InvalidArguments,
            ErrorKind::InvalidOption(_) => adbc_core::error::Status::InvalidArguments,
        }
    }
}

impl Display for TaosError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ErrorKind::Connection(msg) => write!(f, "Connection error: {}", msg),
            ErrorKind::Query(msg) => write!(f, "Query error: {}", msg),
            ErrorKind::Conversion(msg) => write!(f, "Type conversion error: {}", msg),
            ErrorKind::InvalidOption(msg) => write!(f, "Invalid option: {}", msg),
        }
    }
}

impl std::error::Error for TaosError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[derive(Debug)]
enum ErrorKind {
    Connection(String),
    Query(String),
    Conversion(String),
    InvalidOption(String),
}

/// Converts `TaosError` to ADBC `Error`.
impl From<TaosError> for adbc_core::error::Error {
    fn from(err: TaosError) -> Self {
        adbc_core::error::Error::with_message_and_status(err.to_string(), err.adbc_status())
    }
}

/// Converts `taos_client::Error` to `TaosError`.
impl From<taos_client::Error> for TaosError {
    fn from(err: taos_client::Error) -> Self {
        Self::query(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_error() {
        let err = TaosError::connection("test connection error".to_string());
        assert!(err.is_connection());
        assert!(!err.is_query());
        assert!(!err.is_conversion());
        assert_eq!(err.adbc_status(), adbc_core::error::Status::Unknown);
        assert!(err.to_string().contains("Connection error"));
    }

    #[test]
    fn test_query_error() {
        let err = TaosError::query("test query error".to_string());
        assert!(err.is_query());
        assert!(!err.is_connection());
        assert_eq!(err.adbc_status(), adbc_core::error::Status::Unknown);
        assert!(err.to_string().contains("Query error"));
    }

    #[test]
    fn test_conversion_error() {
        let err = TaosError::conversion("test conversion error".to_string());
        assert!(err.is_conversion());
        assert_eq!(err.adbc_status(), adbc_core::error::Status::InvalidArguments);
        assert!(err.to_string().contains("Type conversion error"));
    }

    #[test]
    fn test_invalid_option_error() {
        let err = TaosError::invalid_option("test invalid option".to_string());
        assert_eq!(err.adbc_status(), adbc_core::error::Status::InvalidArguments);
        assert!(err.to_string().contains("Invalid option"));
    }

    #[test]
    fn test_error_to_adbc() {
        let err = TaosError::connection("test".to_string());
        let adbc_err: adbc_core::error::Error = err.into();
        assert_eq!(adbc_err.status, adbc_core::error::Status::Unknown);
    }
}
