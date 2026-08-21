//! Error types for Actflow.
//!
//! All errors in Actflow are represented by the `ActflowError` enum,
//! which provides specific variants for different error categories.

use std::{io::ErrorKind, string::FromUtf8Error};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Unified error type for all Actflow operations.
///
/// Each variant represents a specific category of error that can occur
/// during workflow definition, execution, or storage operations.
#[derive(Deserialize, Serialize, Error, Debug, Clone, PartialEq)]
pub enum ActflowError {
    /// Engine-level errors (startup, shutdown, configuration).
    #[error("{0}")]
    Engine(String),

    /// Configuration parsing or validation errors.
    #[error("{0}")]
    Config(String),

    /// Data conversion errors (JSON, protobuf, etc.).
    #[error("{0}")]
    Convert(String),

    /// Script execution errors (JavaScript, Python).
    #[error("{0}")]
    Script(String),

    /// Structured exception with error code.
    #[error("ecode: {ecode}, message: {message}")]
    Exception {
        ecode: String,
        message: String,
    },

    /// Runtime execution errors.
    #[error("{0}")]
    Runtime(String),

    /// Storage operation errors.
    #[error("{0}")]
    Store(String),

    /// Process lifecycle errors.
    #[error("{0}")]
    Process(String),

    /// Workflow definition errors.
    #[error("{0}")]
    Workflow(String),

    /// Node definition or execution errors.
    #[error("{0}")]
    Node(String),

    /// Edge definition errors.
    #[error("{0}")]
    Edge(String),

    /// Action execution errors.
    #[error("{0}")]
    Action(String),

    /// I/O operation errors.
    #[error("{0}")]
    IoError(String),

    /// Message queue errors.
    #[error("{0}")]
    Queue(String),
}

impl From<ActflowError> for String {
    fn from(val: ActflowError) -> Self {
        val.to_string()
    }
}

impl From<std::io::Error> for ActflowError {
    fn from(error: std::io::Error) -> Self {
        ActflowError::IoError(error.to_string())
    }
}

impl From<ActflowError> for std::io::Error {
    fn from(val: ActflowError) -> Self {
        #[allow(clippy::io_other_error)]
        std::io::Error::new(ErrorKind::Other, val.to_string())
    }
}

impl From<FromUtf8Error> for ActflowError {
    fn from(_: FromUtf8Error) -> Self {
        ActflowError::Runtime("Error with utf-8 string convert".to_string())
    }
}

impl From<serde_json::Error> for ActflowError {
    fn from(error: serde_json::Error) -> Self {
        ActflowError::Convert(error.to_string())
    }
}

impl From<jsonschema::ValidationError<'_>> for ActflowError {
    fn from(error: jsonschema::ValidationError<'_>) -> Self {
        ActflowError::Runtime(error.to_string())
    }
}
