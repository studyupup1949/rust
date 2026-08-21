mod registry;

pub mod interface;

/// UUID generation code with thread-safe pool management.
///
/// This module provides functions for generating unique UUIDs and tracking them in a thread-safe pool.
#[derive(Debug, thiserror::Error)]
pub enum UuidPoolError {
    #[error("Failed to generate unique UUID: {0}")]
    FailedToGenerateUniqueUuidError(String),
    #[error("Failed to find UUID in pool: {0}")]
    FailedToFindUuidInPoolError(String),
    #[error("Failed to set UUID in pool: {0}")]
    FailedToSetUuidInPoolError(String),
}