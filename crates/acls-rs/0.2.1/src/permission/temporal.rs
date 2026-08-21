//! Temporal permissions with time-based validity.

use super::atomic::AtomicPermission;
use super::composite::PermissionSet;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Unix timestamp in milliseconds.
pub type Timestamp = u64;

/// Get the current timestamp in milliseconds.
///
/// # Platform Support
///
/// On native platforms, uses `std::time::SystemTime::now()`.
/// On WASM, uses JavaScript's `Date.now()` via the `js-sys` crate.
#[cfg(not(target_arch = "wasm32"))]
pub fn current_timestamp_millis() -> Timestamp {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(target_arch = "wasm32")]
/// Return the current time as milliseconds since the Unix epoch (WASM variant).
pub fn current_timestamp_millis() -> Timestamp {
    js_sys::Date::now() as u64
}

/// A permission with time-based validity.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub struct TemporalPermission {
    /// The underlying permission.
    pub permission: AtomicPermission,
    /// Start of the validity window (inclusive), or `None` for unbounded.
    pub valid_from: Option<Timestamp>,
    /// End of the validity window (exclusive), or `None` for unbounded.
    pub valid_until: Option<Timestamp>,
}

impl TemporalPermission {
    /// Create a temporal permission with optional validity bounds.
    pub fn new(
        permission: AtomicPermission,
        valid_from: Option<Timestamp>,
        valid_until: Option<Timestamp>,
    ) -> Self {
        Self {
            permission,
            valid_from,
            valid_until,
        }
    }

    /// Returns `true` if this permission is valid at the given timestamp.
    pub fn is_valid_at(&self, timestamp: Timestamp) -> bool {
        let after_start = self.valid_from.is_none_or(|start| timestamp >= start);
        let before_end = self.valid_until.is_none_or(|end| timestamp < end);
        after_start && before_end
    }

    /// Returns `true` if this permission is valid right now.
    pub fn is_currently_valid(&self) -> bool {
        self.is_valid_at(current_timestamp_millis())
    }
}

/// A set of temporal permissions.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub struct TemporalPermissionSet {
    permissions: Vec<TemporalPermission>,
}

impl TemporalPermissionSet {
    /// Create an empty temporal permission set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a temporal permission to this set.
    pub fn add(&mut self, perm: TemporalPermission) {
        self.permissions.push(perm);
    }

    /// Returns the permissions that are valid at the given timestamp.
    pub fn effective_at(&self, timestamp: Timestamp) -> PermissionSet {
        self.permissions
            .iter()
            .filter(|p| p.is_valid_at(timestamp))
            .map(|p| p.permission.clone())
            .collect()
    }

    /// Returns the permissions that are valid right now.
    pub fn currently_effective(&self) -> PermissionSet {
        self.effective_at(current_timestamp_millis())
    }

    /// Returns the number of temporal permissions in this set.
    pub fn len(&self) -> usize {
        self.permissions.len()
    }

    /// Returns `true` if this set contains no temporal permissions.
    pub fn is_empty(&self) -> bool {
        self.permissions.is_empty()
    }

    /// Iterate over the temporal permissions in this set.
    pub fn iter(&self) -> std::slice::Iter<'_, TemporalPermission> {
        self.permissions.iter()
    }

    /// Remove all temporal entries for the given permission. Returns `true` if any were removed.
    pub fn remove(&mut self, permission: &AtomicPermission) -> bool {
        let initial_len = self.permissions.len();
        self.permissions.retain(|tp| &tp.permission != permission);
        self.permissions.len() != initial_len
    }
}
