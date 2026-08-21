//! Error handling.
use std::{error::Error, fmt};

// TODO: Add wrapper type around this so we can expose a generic client error instead of lifxi's.
use adm::lifxi::http::Error as LifxiClientError;

/// Represents an error encountered while using the `turn` subcommand.
#[derive(Debug)]
pub enum TurnError {
    /// No devices matched the given specifier.
    DeviceNotFound(String),
    /// The target state could not be parsed.
    ///
    /// To turn *on* a device, use a state of `on` or `1`; to turn *off* a device, use a state of
    /// `off` or `0`.
    UnrecognizedState(String),
    /// The input was parsed correctly, but the client encountered an error.
    LifxiClientError(LifxiClientError),
}

impl From<LifxiClientError> for TurnError {
    fn from(err: LifxiClientError) -> Self {
        TurnError::LifxiClientError(err)
    }
}

impl fmt::Display for TurnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::TurnError::*;
        match self {
            DeviceNotFound(device) => write!(f, "No devices found matching specifier {}", device),
            UnrecognizedState(state) => write!(f, "Unrecognized target state {}", state),
            LifxiClientError(err) => write!(f, "lifxi error: {}", err),
        }
    }
}

impl Error for TurnError {}
