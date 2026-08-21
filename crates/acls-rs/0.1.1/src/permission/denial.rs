//! Denial sets and grant/denial pairs.

use super::composite::PermissionSet;
use crate::algebra::{JoinSemilattice, MeetSemilattice, Monoid, Semigroup};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A set of explicit denials.
///
/// Semantically distinct from permissions, but uses the same underlying structure.
pub type DenialSet = PermissionSet;

/// A pair of grants and denials forming a dual-lattice structure.
///
/// The effective permissions are computed as `grants - denials`, where denials
/// override grants.
///
/// # Algebraic Structure
///
/// - **Meet** (most restrictive): intersect grants, union denials
/// - **Join** (least restrictive): union grants, intersect denials
///
/// # Examples
///
/// ```
/// use acls_rs::permission::{AtomicPermission, PermissionSet, GrantDenialPair};
///
/// let grants = PermissionSet::from([
///     AtomicPermission::new("file", "read"),
///     AtomicPermission::new("file", "write"),
/// ]);
/// let denials = PermissionSet::from([
///     AtomicPermission::new("file", "delete"),
/// ]);
///
/// let gd = GrantDenialPair::new(grants, denials);
/// let effective = gd.effective_permissions();
/// // effective contains read and write, but not delete
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default)]

pub struct GrantDenialPair {
    pub grants: PermissionSet,
    pub denials: DenialSet,
}

impl GrantDenialPair {
    /// Create a new grant/denial pair.
    pub fn new(grants: PermissionSet, denials: DenialSet) -> Self {
        Self { grants, denials }
    }

    /// Compute the effective permissions (grants minus denials).
    ///
    /// Denials override grants.
    pub fn effective_permissions(&self) -> PermissionSet {
        self.grants.difference(&self.denials)
    }

    /// Check if a specific permission is effectively granted.
    pub fn has_permission(&self, perm: &super::atomic::AtomicPermission) -> bool {
        self.grants.contains(perm) && !self.denials.contains(perm)
    }

    /// Create an empty grant/denial pair.
    pub fn empty() -> Self {
        Self {
            grants: PermissionSet::identity(),
            denials: DenialSet::identity(),
        }
    }
}

// Implement Semigroup: combine both grants and denials
impl Semigroup for GrantDenialPair {
    fn combine(self, other: Self) -> Self {
        Self {
            grants: self.grants.combine(other.grants),
            denials: self.denials.combine(other.denials),
        }
    }
}

// Implement Monoid: identity is empty grants and denials
impl Monoid for GrantDenialPair {
    fn identity() -> Self {
        Self::empty()
    }
}

// Implement PartialOrd based on effective permissions
impl PartialOrd for GrantDenialPair {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.effective_permissions()
            .partial_cmp(&other.effective_permissions())
    }
}

// Implement MeetSemilattice: intersect grants, union denials (most restrictive)
impl MeetSemilattice for GrantDenialPair {
    fn meet(self, other: Self) -> Self {
        Self {
            grants: self.grants.meet(other.grants),
            denials: self.denials.join(other.denials), // Union denials
        }
    }
}

// Implement JoinSemilattice: union grants, intersect denials (least restrictive)
impl JoinSemilattice for GrantDenialPair {
    fn join(self, other: Self) -> Self {
        Self {
            grants: self.grants.join(other.grants),
            denials: self.denials.meet(other.denials), // Intersect denials
        }
    }
}
