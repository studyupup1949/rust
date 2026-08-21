//! Configurable bitmask-to-permission mapping.
//!
//! A [`PermissionMapping`] defines how bitmask values translate to
//! [`AtomicPermission`] entries in a [`PermissionSet`]. Different access
//! control models assign different semantics to the same bit positions —
//! for example, bit 0x0010 is `FILE_WRITE_EA` in Windows file contexts
//! but `READ_PROP` in Active Directory. A `PermissionMapping` lets each
//! model define its own interpretation.
//!
//! # Examples
//!
//! ```
//! use acls_rs::permission::{AtomicPermission, PermissionMapping, PermissionSet};
//!
//! let mapping = PermissionMapping::new("posix")
//!     .add(0b100, "read")
//!     .add(0b010, "write")
//!     .add(0b001, "execute");
//!
//! let set = mapping.to_permission_set(0b101);
//! assert!(set.contains(&AtomicPermission::new("posix", "read")));
//! assert!(set.contains(&AtomicPermission::new("posix", "execute")));
//! assert!(!set.contains(&AtomicPermission::new("posix", "write")));
//!
//! assert_eq!(mapping.from_permission_set(&set), 0b101);
//! ```

use super::{AtomicPermission, PermissionSet};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A mapping between bitmask values and named permissions.
///
/// Each entry maps a single bit (or bit pattern) to an action name in a
/// fixed namespace. The mapping is bidirectional: bits can be expanded
/// into a [`PermissionSet`], and a `PermissionSet` can be collapsed back
/// into a bitmask.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PermissionMapping {
    namespace: String,
    entries: Vec<(u32, String)>,
}

impl PermissionMapping {
    /// Create a new mapping with the given namespace.
    pub fn new(namespace: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            entries: Vec::new(),
        }
    }

    /// Add a bit-to-action entry (builder pattern).
    pub fn add(mut self, bit: u32, action: impl Into<String>) -> Self {
        self.entries.push((bit, action.into()));
        self
    }

    /// Return the namespace for this mapping.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Return the entries as `(bit, action)` pairs.
    pub fn entries(&self) -> &[(u32, String)] {
        &self.entries
    }

    /// Expand a bitmask into a [`PermissionSet`].
    ///
    /// Each entry whose bit pattern is present in `bits` produces an
    /// `AtomicPermission` in the result set.
    pub fn to_permission_set(&self, bits: u32) -> PermissionSet {
        let mut set = PermissionSet::new();
        for (mask, action) in &self.entries {
            if bits & mask == *mask {
                set.insert(AtomicPermission::new(&self.namespace, action));
            }
        }
        set
    }

    /// Collapse a [`PermissionSet`] back into a bitmask.
    ///
    /// Only permissions matching this mapping's namespace are considered.
    pub fn from_permission_set(&self, set: &PermissionSet) -> u32 {
        let mut bits = 0u32;
        for perm in set.iter() {
            if perm.namespace() == self.namespace {
                for (mask, action) in &self.entries {
                    if perm.action() == action {
                        bits |= mask;
                        break;
                    }
                }
            }
        }
        bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_roundtrip() {
        let m = PermissionMapping::new("test")
            .add(0x01, "read")
            .add(0x02, "write")
            .add(0x04, "execute");

        let set = m.to_permission_set(0x05);
        assert!(set.contains(&AtomicPermission::new("test", "read")));
        assert!(!set.contains(&AtomicPermission::new("test", "write")));
        assert!(set.contains(&AtomicPermission::new("test", "execute")));

        assert_eq!(m.from_permission_set(&set), 0x05);
    }

    #[test]
    fn empty_bits() {
        let m = PermissionMapping::new("test").add(0x01, "read");
        let set = m.to_permission_set(0);
        assert!(set.is_empty());
        assert_eq!(m.from_permission_set(&set), 0);
    }

    #[test]
    fn ignores_other_namespace() {
        let m = PermissionMapping::new("test").add(0x01, "read");
        let mut set = PermissionSet::new();
        set.insert(AtomicPermission::new("other", "read"));
        assert_eq!(m.from_permission_set(&set), 0);
    }
}
