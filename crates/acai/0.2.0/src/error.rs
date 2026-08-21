// Error mapping utilities for acai.
//
// This module provides utilities for mapping between different error types
// used in the acai codebase. These functions standardize error handling
// patterns and improve code consistency.

use crate::JsonRpcError;
use crate::client::Error as ClientError;
use crate::server::Error as ServerError;
use crate::server::push_notification::PushNotificationError;
use http::uri::InvalidUri;
use std::fmt;

/// Error type for task management operations
#[derive(Debug)]
pub struct TaskError {
    message: String,
}

impl TaskError {
    /// Create a new TaskError
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Task error: {}", self.message)
    }
}

impl std::error::Error for TaskError {}

// Implement From for different error conversions

/// Convert TaskError to JsonRpcError for server-side error handling
impl From<TaskError> for JsonRpcError {
    fn from(err: TaskError) -> Self {
        JsonRpcError::task(err.message)
    }
}

/// Convert TaskError to ClientError for client-side error handling
impl From<TaskError> for ClientError {
    fn from(err: TaskError) -> Self {
        ClientError::TaskError(err.message)
    }
}

/// Convert std::io::Error to JsonRpcError
impl From<std::io::Error> for JsonRpcError {
    fn from(err: std::io::Error) -> Self {
        JsonRpcError::internal_error(format!("IO error: {}", err))
    }
}

/// Convert hyper::Error to JsonRpcError
impl From<hyper::Error> for JsonRpcError {
    fn from(err: hyper::Error) -> Self {
        JsonRpcError::internal_error(format!("Hyper error: {}", err))
    }
}

/// Convert PushNotificationError to JsonRpcError
impl From<PushNotificationError> for JsonRpcError {
    fn from(err: PushNotificationError) -> Self {
        JsonRpcError::push_notification(err)
    }
}

/// Convert PushNotificationError to ClientError
impl From<PushNotificationError> for ClientError {
    fn from(err: PushNotificationError) -> Self {
        match err {
            PushNotificationError::HttpError(e) => ClientError::HttpError(e),
            PushNotificationError::SerializationError(e) => ClientError::SerializationError(e),
            _ => ClientError::InternalError(err.to_string()),
        }
    }
}

/// Convert InvalidUri to JsonRpcError
impl From<InvalidUri> for JsonRpcError {
    fn from(err: InvalidUri) -> Self {
        JsonRpcError::invalid_parameters(format!("Invalid URI: {}", err))
    }
}

/// Convert JsonRpcError to ServerError
impl From<JsonRpcError> for ServerError {
    fn from(err: JsonRpcError) -> Self {
        ServerError::JsonRpcError(err)
    }
}

/// Convert JsonRpcError to ClientError
impl From<JsonRpcError> for ClientError {
    fn from(err: JsonRpcError) -> Self {
        ClientError::JsonRpcError(err)
    }
}

/// Convert ClientError to JsonRpcError
impl From<ClientError> for JsonRpcError {
    fn from(err: ClientError) -> Self {
        match err {
            ClientError::JsonRpcError(e) => e,
            _ => JsonRpcError::internal_error(err.to_string()),
        }
    }
}

/// Convert TaskError to ServerError
impl From<TaskError> for ServerError {
    fn from(err: TaskError) -> Self {
        ServerError::Other(err.to_string())
    }
}

/// Convert PushNotificationError to ServerError
impl From<PushNotificationError> for ServerError {
    fn from(err: PushNotificationError) -> Self {
        match err {
            PushNotificationError::HttpError(e) => {
                ServerError::Other(format!("Push notification HTTP error: {}", e))
            }
            _ => ServerError::Other(err.to_string()),
        }
    }
}

/// Convert std::io::Error to ServerError
impl From<std::io::Error> for ServerError {
    fn from(err: std::io::Error) -> Self {
        ServerError::Other(format!("IO error: {}", err))
    }
}

/// Convert std::io::Error to ClientError
impl From<std::io::Error> for ClientError {
    fn from(err: std::io::Error) -> Self {
        ClientError::InternalError(format!("IO error: {}", err))
    }
}

/// Convert std::io::Error to TaskError
impl From<std::io::Error> for TaskError {
    fn from(err: std::io::Error) -> Self {
        TaskError::new(format!("IO error: {}", err))
    }
}

/// Generic error mapper for JsonRpcError
///
/// This function creates an internal error JsonRpcError with the provided error's
/// message. Use this for server-side error mapping when you can't use the ? operator.
///
/// # Example
/// ```
/// use acai::error::to_json_rpc_error;
///
/// let std_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
/// let json_rpc_error = to_json_rpc_error(std_error);
/// ```
pub fn to_json_rpc_error<E: fmt::Display>(err: E) -> JsonRpcError {
    JsonRpcError::internal_error(format!("{}", err))
}

/// Generic error mapper for ClientError
///
/// This function creates a ClientError with the provided error's
/// message. Use this for client-side error mapping when you can't use the ? operator.
///
/// # Example
/// ```
/// use acai::error::to_client_error;
///
/// let std_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
/// let client_error = to_client_error(std_error);
/// ```
pub fn to_client_error<E: fmt::Display>(err: E) -> ClientError {
    ClientError::InternalError(format!("{}", err))
}

/// Generic error mapper for ServerError
///
/// This function creates a ServerError with the provided error's
/// message. Use this for server-side error mapping when you can't use the ? operator.
///
/// # Example
/// ```
/// use acai::error::to_server_error;
///
/// let std_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
/// let server_error = to_server_error(std_error);
/// ```
pub fn to_server_error<E: fmt::Display>(err: E) -> ServerError {
    ServerError::Other(format!("{}", err))
}

/// Generic error mapper for TaskError
///
/// This function creates a TaskError with the provided error's
/// message. Use this for task management operations when you can't use the ? operator.
///
/// # Example
/// ```
/// use acai::error::to_task_error;
///
/// let std_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
/// let task_error = to_task_error(std_error);
/// ```
pub fn to_task_error<E: fmt::Display>(err: E) -> TaskError {
    TaskError::new(format!("{}", err))
}

/// Error type for URL validation
#[derive(Debug)]
pub struct UrlValidationError {
    message: String,
}

impl UrlValidationError {
    /// Create a new UrlValidationError
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for UrlValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "URL validation error: {}", self.message)
    }
}

impl std::error::Error for UrlValidationError {}

/// Convert UrlValidationError to JsonRpcError
impl From<UrlValidationError> for JsonRpcError {
    fn from(err: UrlValidationError) -> Self {
        JsonRpcError::invalid_parameters(err.message)
    }
}

/// Convert UrlValidationError to PushNotificationError
impl From<UrlValidationError> for PushNotificationError {
    fn from(err: UrlValidationError) -> Self {
        PushNotificationError::Other(err.to_string())
    }
}

/// Validate that a URL is properly formatted
pub fn validate_url(url: &str) -> Result<(), UrlValidationError> {
    // Check if URL starts with http:// or https://
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(UrlValidationError::new(
            "URL must start with http:// or https://",
        ));
    }

    // Check if URL can be parsed
    match url.parse::<http::Uri>() {
        Ok(_) => Ok(()),
        Err(e) => Err(UrlValidationError::new(format!("Invalid URL: {}", e))),
    }
}
