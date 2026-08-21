pub mod interface;

mod registry;

pub use uuid as registry_uuid;

pub type NamespaceString = String;
pub type ContextString = String;

/// UUID generation code with thread-safe pool management.
///
/// This module provides functions for generating unique UUIDs and tracking them in a thread-safe pool.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq, Hash)]
pub enum UuidPoolError {
    #[error("Failed to generate unique UUID: {0}")]
    FailedToGenerateUniqueUuidError(String),
    #[error("Failed to find UUID in pool: {0}")]
    FailedToFindUuidInPoolError(String),
    #[error("Failed to set UUID in pool: {0}")]
    FailedToSetUuidInPoolError(String),
    #[error("Failed to add UUID to pool: {0}")]
    FailedToAddUuidToPoolError(String),
    #[error("Failed to remove UUID from pool: {0}")]
    FailedToRemoveUuidFromPoolError(String),
    #[error("Failed to replace UUID in pool: {0}")]
    FailedToReplaceUuidInPoolError(String),
}
