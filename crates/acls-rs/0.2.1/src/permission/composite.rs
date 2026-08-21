//! Composite permissions: sets of atomic permissions with algebraic structure.

use super::atomic::AtomicPermission;
use crate::algebra::{JoinSemilattice, MeetSemilattice, Monoid, Semigroup};
use std::collections::BTreeSet;
use std::fmt;
use std::iter::FromIterator;
use std::ops::{BitAnd, BitOr, Sub};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A set of atomic permissions forming a lattice structure.
///
/// `PermissionSet` is the primary building block for permission systems. It implements:
/// - [`Semigroup`] via union
/// - [`Monoid`] with empty set as identity
/// - [`MeetSemilattice`] with intersection as meet
/// - [`JoinSemilattice`] with union as join
/// - `Lattice` (via blanket impl)
///
/// The partial order is subset inclusion: `a ≤ b` iff `a ⊆ b` iff `a` is more restrictive than `b`.
///
/// # Examples
///
/// ```
/// use acls_rs::permission::{AtomicPermission, PermissionSet};
/// use acls_rs::algebra::{Monoid, Semigroup, MeetSemilattice, JoinSemilattice};
///
/// let perms1 = PermissionSet::from([
///     AtomicPermission::new("file", "read"),
/// ]);
/// let perms2 = PermissionSet::from([
///     AtomicPermission::new("file", "write"),
/// ]);
///
/// // Semigroup: union
/// let combined = perms1.clone().combine(perms2.clone());
/// assert_eq!(combined.len(), 2);
///
/// // Monoid: identity is empty
/// let empty = PermissionSet::identity();
/// assert!(empty.is_empty());
///
/// // Meet: intersection (most restrictive)
/// let intersection = perms1.clone().meet(perms2.clone());
/// assert!(intersection.is_empty());
///
/// // Join: union (least restrictive)
/// let union = perms1.join(perms2);
/// assert_eq!(union.len(), 2);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PermissionSet(pub(crate) BTreeSet<AtomicPermission>);

impl PermissionSet {
    /// Create a new empty permission set.
    ///
    /// # Examples
    ///
    /// ```
    /// use acls_rs::permission::PermissionSet;
    ///
    /// let perms = PermissionSet::new();
    /// assert!(perms.is_empty());
    /// ```
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }

    /// Check if the permission set is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use acls_rs::permission::PermissionSet;
    ///
    /// let perms = PermissionSet::new();
    /// assert!(perms.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of permissions in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use acls_rs::permission::{AtomicPermission, PermissionSet};
    ///
    /// let perms = PermissionSet::from([
    ///     AtomicPermission::new("file", "read"),
    ///     AtomicPermission::new("file", "write"),
    /// ]);
    /// assert_eq!(perms.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the set contains a specific permission.
    ///
    /// # Examples
    ///
    /// ```
    /// use acls_rs::permission::{AtomicPermission, PermissionSet};
    ///
    /// let perms = PermissionSet::from([
    ///     AtomicPermission::new("file", "read"),
    /// ]);
    ///
    /// assert!(perms.contains(&AtomicPermission::new("file", "read")));
    /// assert!(!perms.contains(&AtomicPermission::new("file", "write")));
    /// ```
    pub fn contains(&self, perm: &AtomicPermission) -> bool {
        self.0.contains(perm)
    }

    /// Check if this set is a superset of another.
    ///
    /// In permission terms: this set grants at least all permissions in `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use acls_rs::permission::{AtomicPermission, PermissionSet};
    ///
    /// let all = PermissionSet::from([
    ///     AtomicPermission::new("file", "read"),
    ///     AtomicPermission::new("file", "write"),
    /// ]);
    /// let some = PermissionSet::from([
    ///     AtomicPermission::new("file", "read"),
    /// ]);
    ///
    /// assert!(all.is_superset_of(&some));
    /// assert!(!some.is_superset_of(&all));
    /// ```
    pub fn is_superset_of(&self, other: &Self) -> bool {
        self.0.is_superset(&other.0)
    }

    /// Check if this set is a subset of another.
    ///
    /// In permission terms: `other` grants at least all permissions in this set.
    ///
    /// # Examples
    ///
    /// ```
    /// use acls_rs::permission::{AtomicPermission, PermissionSet};
    ///
    /// let some = PermissionSet::from([
    ///     AtomicPermission::new("file", "read"),
    /// ]);
    /// let all = PermissionSet::from([
    ///     AtomicPermission::new("file", "read"),
    ///     AtomicPermission::new("file", "write"),
    /// ]);
    ///
    /// assert!(some.is_subset_of(&all));
    /// assert!(!all.is_subset_of(&some));
    /// ```
    pub fn is_subset_of(&self, other: &Self) -> bool {
        self.0.is_subset(&other.0)
    }

    /// Check if this set has no permissions in common with another.
    ///
    /// # Examples
    ///
    /// ```
    /// use acls_rs::permission::{AtomicPermission, PermissionSet};
    ///
    /// let read = PermissionSet::from([AtomicPermission::new("file", "read")]);
    /// let write = PermissionSet::from([AtomicPermission::new("file", "write")]);
    ///
    /// assert!(read.is_disjoint(&write));
    /// ```
    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.0.is_disjoint(&other.0)
    }

    /// Insert a permission into the set. Returns `true` if the permission was not already present.
    ///
    /// # Examples
    ///
    /// ```
    /// use acls_rs::permission::{AtomicPermission, PermissionSet};
    ///
    /// let mut perms = PermissionSet::new();
    /// assert!(perms.insert(AtomicPermission::new("file", "read")));
    /// assert!(!perms.insert(AtomicPermission::new("file", "read"))); // Already present
    /// assert_eq!(perms.len(), 1);
    /// ```
    pub fn insert(&mut self, perm: AtomicPermission) -> bool {
        self.0.insert(perm)
    }

    /// Remove a permission from the set. Returns `true` if the permission was present.
    ///
    /// # Examples
    ///
    /// ```
    /// use acls_rs::permission::{AtomicPermission, PermissionSet};
    ///
    /// let mut perms = PermissionSet::from([AtomicPermission::new("file", "read")]);
    /// assert!(perms.remove(&AtomicPermission::new("file", "read")));
    /// assert!(!perms.remove(&AtomicPermission::new("file", "write"))); // Not present
    /// assert!(perms.is_empty());
    /// ```
    pub fn remove(&mut self, perm: &AtomicPermission) -> bool {
        self.0.remove(perm)
    }

    /// Iterate over the permissions in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use acls_rs::permission::{AtomicPermission, PermissionSet};
    ///
    /// let perms = PermissionSet::from([
    ///     AtomicPermission::new("file", "read"),
    /// ]);
    ///
    /// for perm in perms.iter() {
    ///     println!("{}", perm);
    /// }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &AtomicPermission> {
        self.0.iter()
    }

    /// Compute the set difference (permissions in self but not in other).
    ///
    /// # Examples
    ///
    /// ```
    /// use acls_rs::permission::{AtomicPermission, PermissionSet};
    ///
    /// let all = PermissionSet::from([
    ///     AtomicPermission::new("file", "read"),
    ///     AtomicPermission::new("file", "write"),
    /// ]);
    /// let read_only = PermissionSet::from([
    ///     AtomicPermission::new("file", "read"),
    /// ]);
    ///
    /// let diff = all.difference(&read_only);
    /// assert_eq!(diff.len(), 1);
    /// assert!(diff.contains(&AtomicPermission::new("file", "write")));
    /// ```
    pub fn difference(&self, other: &Self) -> Self {
        Self(self.0.difference(&other.0).cloned().collect())
    }

    /// Compute the symmetric difference (permissions in either set but not both).
    ///
    /// # Examples
    ///
    /// ```
    /// use acls_rs::permission::{AtomicPermission, PermissionSet};
    ///
    /// let perms1 = PermissionSet::from([AtomicPermission::new("file", "read")]);
    /// let perms2 = PermissionSet::from([AtomicPermission::new("file", "write")]);
    ///
    /// let sym_diff = perms1.symmetric_difference(&perms2);
    /// assert_eq!(sym_diff.len(), 2);
    /// ```
    pub fn symmetric_difference(&self, other: &Self) -> Self {
        Self(self.0.symmetric_difference(&other.0).cloned().collect())
    }

    /// Create a builder for constructing permission sets.
    ///
    /// # Examples
    ///
    /// ```
    /// use acls_rs::permission::{AtomicPermission, PermissionSet};
    ///
    /// let perms = PermissionSet::builder()
    ///     .insert(AtomicPermission::new("file", "read"))
    ///     .insert(AtomicPermission::new("file", "write"))
    ///     .build();
    ///
    /// assert_eq!(perms.len(), 2);
    /// ```
    pub fn builder() -> PermissionSetBuilder {
        PermissionSetBuilder::new()
    }
}

/// Builder for constructing permission sets.
#[derive(Default)]
pub struct PermissionSetBuilder {
    permissions: BTreeSet<AtomicPermission>,
}

impl PermissionSetBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a permission into the set.
    pub fn insert(mut self, perm: AtomicPermission) -> Self {
        self.permissions.insert(perm);
        self
    }

    /// Build the permission set.
    pub fn build(self) -> PermissionSet {
        PermissionSet(self.permissions)
    }
}

// Implement Default
impl Default for PermissionSet {
    fn default() -> Self {
        Self::new()
    }
}

// Implement Semigroup via union
impl Semigroup for PermissionSet {
    #[inline]
    fn combine(self, other: Self) -> Self {
        Self(self.0.union(&other.0).cloned().collect())
    }
}

// Implement Monoid with empty set as identity
impl Monoid for PermissionSet {
    #[inline]
    fn identity() -> Self {
        Self::new()
    }
}

// Implement PartialOrd via subset relation
impl PartialOrd for PermissionSet {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;

        if self == other {
            Some(Ordering::Equal)
        } else if self.is_subset_of(other) {
            Some(Ordering::Less)
        } else if self.is_superset_of(other) {
            Some(Ordering::Greater)
        } else {
            None // Incomparable (neither subset nor superset)
        }
    }
}

// Implement MeetSemilattice with intersection as meet
impl MeetSemilattice for PermissionSet {
    #[inline]
    fn meet(self, other: Self) -> Self {
        Self(self.0.intersection(&other.0).cloned().collect())
    }
}

// Implement JoinSemilattice with union as join
impl JoinSemilattice for PermissionSet {
    #[inline]
    fn join(self, other: Self) -> Self {
        self.combine(other) // Reuse semigroup combine
    }
}

// Operator overloading: & for meet
impl BitAnd for PermissionSet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.meet(rhs)
    }
}

// Operator overloading: | for join
impl BitOr for PermissionSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.join(rhs)
    }
}

// Operator overloading: - for difference
impl Sub for PermissionSet {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(&rhs)
    }
}

// Implement From conversions
impl From<BTreeSet<AtomicPermission>> for PermissionSet {
    fn from(set: BTreeSet<AtomicPermission>) -> Self {
        Self(set)
    }
}

impl From<Vec<AtomicPermission>> for PermissionSet {
    fn from(vec: Vec<AtomicPermission>) -> Self {
        Self(vec.into_iter().collect())
    }
}

impl<const N: usize> From<[AtomicPermission; N]> for PermissionSet {
    fn from(arr: [AtomicPermission; N]) -> Self {
        Self(arr.into_iter().collect())
    }
}

// Implement FromIterator
impl FromIterator<AtomicPermission> for PermissionSet {
    fn from_iter<T: IntoIterator<Item = AtomicPermission>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

// Implement IntoIterator
impl IntoIterator for PermissionSet {
    type Item = AtomicPermission;
    type IntoIter = std::collections::btree_set::IntoIter<AtomicPermission>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// Implement Extend
impl Extend<AtomicPermission> for PermissionSet {
    fn extend<T: IntoIterator<Item = AtomicPermission>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

// Implement Display
impl fmt::Display for PermissionSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        for (i, perm) in self.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", perm)?;
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn perm(ns: &str, action: &str) -> AtomicPermission {
        AtomicPermission::new(ns, action)
    }

    #[test]
    fn test_new() {
        let perms = PermissionSet::new();
        assert!(perms.is_empty());
        assert_eq!(perms.len(), 0);
    }

    #[test]
    fn test_from_array() {
        let perms = PermissionSet::from([perm("file", "read"), perm("file", "write")]);

        assert_eq!(perms.len(), 2);
        assert!(perms.contains(&perm("file", "read")));
    }

    #[test]
    fn test_semigroup_combine() {
        let p1 = PermissionSet::from([perm("file", "read")]);
        let p2 = PermissionSet::from([perm("file", "write")]);

        let combined = p1.combine(p2);
        assert_eq!(combined.len(), 2);
    }

    #[test]
    fn test_monoid_identity() {
        let p = PermissionSet::from([perm("file", "read")]);
        let e = PermissionSet::identity();

        assert_eq!(p.clone().combine(e.clone()), p);
        assert_eq!(e.combine(p.clone()), p);
    }

    #[test]
    fn test_meet() {
        let p1 = PermissionSet::from([perm("file", "read"), perm("file", "write")]);
        let p2 = PermissionSet::from([perm("file", "read")]);

        let meet = p1.meet(p2);
        assert_eq!(meet.len(), 1);
        assert!(meet.contains(&perm("file", "read")));
    }

    #[test]
    fn test_join() {
        let p1 = PermissionSet::from([perm("file", "read")]);
        let p2 = PermissionSet::from([perm("file", "write")]);

        let join = p1.join(p2);
        assert_eq!(join.len(), 2);
    }

    #[test]
    fn test_partial_ord() {
        let subset = PermissionSet::from([perm("file", "read")]);
        let superset = PermissionSet::from([perm("file", "read"), perm("file", "write")]);

        assert!(subset < superset);
        assert!(superset > subset);
    }

    #[test]
    fn test_operators() {
        let p1 = PermissionSet::from([perm("file", "read"), perm("file", "write")]);
        let p2 = PermissionSet::from([perm("file", "read")]);

        // Meet (intersection)
        let meet = p1.clone() & p2.clone();
        assert_eq!(meet.len(), 1);

        // Join (union)
        let join = p1.clone() | p2.clone();
        assert_eq!(join.len(), 2);

        // Difference
        let diff = p1 - p2;
        assert_eq!(diff.len(), 1);
        assert!(diff.contains(&perm("file", "write")));
    }

    #[test]
    fn test_display() {
        let perms = PermissionSet::from([perm("file", "read")]);
        let s = perms.to_string();
        assert!(s.contains("file:read"));
    }
}
