use std::{io::ErrorKind, string::FromUtf8Error};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Deserialize, Serialize, Error, Debug, Clone, PartialEq)]
pub enum ActflowError {
    #[error("{0}")]
    Engine(String),

    #[error("{0}")]
    Config(String),

    #[error("{0}")]
    Convert(String),

    #[error("{0}")]
    Script(String),

    #[error("ecode: {ecode}, message: {message}")]
    Exception {
        ecode: String,
        message: String,
    },

    #[error("{0}")]
    Runtime(String),

    #[error("{0}")]
    Store(String),

    #[error("{0}")]
    Process(String),

    #[error("{0}")]
    Workflow(String),

    #[error("{0}")]
    Node(String),

    #[error("{0}")]
    Edge(String),

    #[error("{0}")]
    Action(String),

    #[error("{0}")]
    IoError(String),

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
