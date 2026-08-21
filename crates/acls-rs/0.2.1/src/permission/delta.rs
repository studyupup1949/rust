//! Permission deltas: add/remove operations forming a monoid.

use super::atomic::AtomicPermission;
use super::composite::PermissionSet;
use crate::algebra::{Monoid, Semigroup};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A delta representing permission changes (additions and removals).
///
/// Forms a monoid where:
/// - Identity is empty add/remove sets
/// - Combine merges both add and remove sets
///
/// # Examples
///
/// ```
/// use acls_rs::permission::{PermissionDelta, AtomicPermission};
///
/// let delta = PermissionDelta::builder()
///     .grant_str("file", "read")
///     .grant_str("file", "write")
///     .remove_str("file", "delete")
///     .build();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PermissionDelta {
    /// Permissions to add.
    pub add: PermissionSet,
    /// Permissions to remove.
    pub remove: PermissionSet,
}

impl PermissionDelta {
    /// Create a new permission delta.
    pub fn new(add: PermissionSet, remove: PermissionSet) -> Self {
        Self { add, remove }
    }

    /// Create an empty delta (no changes).
    pub fn empty() -> Self {
        Self {
            add: PermissionSet::identity(),
            remove: PermissionSet::identity(),
        }
    }

    /// Apply this delta to a permission set.
    pub fn apply_to(&self, perms: PermissionSet) -> PermissionSet {
        perms.combine(self.add.clone()).difference(&self.remove)
    }

    /// Invert this delta (swap add and remove).
    pub fn invert(self) -> Self {
        Self {
            add: self.remove,
            remove: self.add,
        }
    }

    /// Create a builder for constructing deltas.
    pub fn builder() -> PermissionDeltaBuilder {
        PermissionDeltaBuilder::default()
    }
}

/// Builder for permission deltas.
#[derive(Default)]
pub struct PermissionDeltaBuilder {
    add: PermissionSet,
    remove: PermissionSet,
}

impl PermissionDeltaBuilder {
    /// Add a permission to the grants set.
    pub fn grant(mut self, perm: AtomicPermission) -> Self {
        self.add.extend([perm]);
        self
    }

    /// Add a permission from strings to the grants set.
    pub fn grant_str(self, namespace: &str, action: &str) -> Self {
        self.grant(AtomicPermission::new(namespace, action))
    }

    /// Add a permission to the remove set.
    pub fn remove(mut self, perm: AtomicPermission) -> Self {
        self.remove.extend([perm]);
        self
    }

    /// Add a permission from strings to remove set.
    pub fn remove_str(self, namespace: &str, action: &str) -> Self {
        self.remove(AtomicPermission::new(namespace, action))
    }

    /// Build the delta.
    pub fn build(self) -> PermissionDelta {
        PermissionDelta {
            add: self.add,
            remove: self.remove,
        }
    }
}

// Implement Semigroup: combine both add and remove sets
impl Semigroup for PermissionDelta {
    fn combine(self, other: Self) -> Self {
        Self {
            add: self.add.combine(other.add),
            remove: self.remove.combine(other.remove),
        }
    }
}

// Implement Monoid: identity is empty delta
impl Monoid for PermissionDelta {
    fn identity() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_to() {
        let perms = PermissionSet::from([AtomicPermission::new("file", "read")]);

        let delta = PermissionDelta::builder()
            .grant_str("file", "write")
            .remove_str("file", "read")
            .build();

        let result = delta.apply_to(perms);
        assert_eq!(result.len(), 1);
        assert!(result.contains(&AtomicPermission::new("file", "write")));
        assert!(!result.contains(&AtomicPermission::new("file", "read")));
    }

    #[test]
    fn test_monoid_identity() {
        let delta = PermissionDelta::builder().grant_str("file", "read").build();
        let identity = PermissionDelta::identity();

        assert_eq!(delta.clone().combine(identity), delta);
    }
}
