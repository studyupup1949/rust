//! # Active UUID Registry
//! 
//! A library for managing in-process, system-wide `UUID`s for tracking component liveness.
//! 
//! This library provides a thread-safe UUID pool that can be used to track the liveness of UUIDs.
//! 
//! UUIDs are organized in a two-level global registry (`namespace -> context -> UUID set`), making it straightforward to track running components across logical scopes in dynamic systems.
//! 
//! Strongly encouraged to use proper management and implementation practices for their particular process/use-case, as there is no garbage collection or pool monitoring on the part of this library.
//! 
//! Use at your own (possible but unlikely) risk. *You* are responsible for managing the pool and its contents.
//! 
//! # Feature Flags
//! 
//! - **Default / single-threaded**: Use a single-threaded map for the UUID pool.
//!   - Uses a `parking_lot::Mutex<HashMap<NamespaceKey, HashMap<ContextKey, HashSet<Uuid>>>>` for the UUID pool.
//! - **concurrent-map**: Use a concurrent map for the UUID pool.
//!   - Uses a `DashMap<NamespaceKey, DashMap<ContextKey, DashSet<Uuid>>>` for the UUID pool.
//! 


mod registry;

/// Interface API module for the Active UUID Registry.
pub mod interface;

mod raii;

/// A cloneable, opaque write-authorization capability for a namespace claimed via
/// [`interface::reserve_owned_namespace`].
pub use raii::OwnedNamespace;

#[doc(inline)]
pub use uuid as registry_uuid;  

/// Represents the name of a namespace within the registry pool.
pub type NamespaceString = String;

/// Represents the name of a context within the registry pool.
pub type ContextString = String;

/// Error type for UUID pool operations.
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
    #[error("Unauthorized write access to namespace '{0}'")]
    UnauthorizedNamespaceWriteAccessError(String),
    #[error("Failed to claim owned namespace: {0}")]
    FailedToClaimNamespaceError(String),
}
