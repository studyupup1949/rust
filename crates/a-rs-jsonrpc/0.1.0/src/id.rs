//! # JSON-RPC Identifiers
//!
//! This module defines the [`Id`] enum, which represents the `id` field in JSON-RPC
//! requests and responses. It supports both numeric and string identifiers as
//! per the JSON-RPC 2.0 specification.

use serde::{Deserialize, Serialize};

static ATOMIC_U64_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// A JSON-RPC identifier that can be either a number or a string.
///
/// This enum uses `#[serde(untagged)]` to ensure it serializes directly to the
/// underlying value (e.g., `1` or `"id-1"`) rather than a tagged object,
/// maintaining strict compatibility with the JSON-RPC specification.
///
/// ### Usage
/// ```rust
/// use a_rs_jsonrpc::Id;
///
/// // Create from literal
/// let id = Id::from(42);
///
/// // Generate automatic IDs
/// let next = Id::next_number();
/// ```

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Id {
    /// A numeric identifier (integer).
    Number(u64),
    /// A string identifier.
    String(String),
}

impl From<u64> for Id {
    fn from(value: u64) -> Self {
        Id::Number(value)
    }
}

impl From<String> for Id {
    fn from(value: String) -> Self {
        Id::String(value)
    }
}

impl From<&str> for Id {
    fn from(value: &str) -> Self {
        Id::String(value.to_string())
    }
}

impl Id {
    /// Generates a unique numeric ID by incrementing a global atomic counter.
    ///
    /// This is the preferred method for generating IDs for new client requests
    /// to ensure thread-safe uniqueness.
    pub fn next_number() -> Self {
        let id = ATOMIC_U64_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Id::Number(id)
    }

    /// Generates a unique string ID in the format `"id-{N}"`.
    ///
    /// Useful for systems that require or prefer string-based identifiers.
    pub fn next_string() -> Self {
        let id = ATOMIC_U64_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Id::String(format!("id-{}", id))
    }
}
