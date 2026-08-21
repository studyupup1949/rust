use thiserror::Error;
use std::any::TypeId;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Service not found: {0:?}")]
    ServiceNotFound(TypeId),

    #[error("Missing dependency: {0:?}")]
    MissingDependency(TypeId),

    #[error("Initialization error: {0}")]
    InitializationError(String),

    #[error("Shutdown error: {0}")]
    ShutdownError(String),
}
