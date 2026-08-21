use thiserror::Error;

/// Error type for Docker container operations.
#[derive(Error, Debug)]
pub enum DockerError {
    #[error("Failed to start container")]
    ContainerStartFailed(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Failed to stop container")]
    ContainerStopFailed(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Failed to get container host")]
    HostResolutionFailed,

    #[error("Failed to get container port {port}")]
    PortMappingFailed { port: u16 },

    #[error("Failed to construct service wrapper")]
    WrapperConstructionFailed(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Container operation failed: {0}")]
    OperationFailed(String),
}
