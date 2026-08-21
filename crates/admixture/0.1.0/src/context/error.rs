use std::time::Duration;
use thiserror::Error;

/// Error type for context operations.
///
/// This is a generic container for errors that occur during context lifecycle.
/// Specific service errors are wrapped as opaque trait objects, allowing the
/// context layer to work with any service implementation without coupling to
/// specific error types.
#[derive(Error, Debug)]
pub enum ContextError {
    #[error("Service failed to start")]
    ServiceStartFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Health check timed out after {elapsed:?} ({attempts} attempts)")]
    HealthCheckTimeout {
        elapsed: Duration,
        attempts: usize,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Health check failed")]
    HealthCheckFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Failed to build context")]
    BuildFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Context shutdown failed")]
    ShutdownFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
