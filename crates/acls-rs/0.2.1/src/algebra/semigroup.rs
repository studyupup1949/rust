//! Semigroup algebraic structure.
//!
//! A semigroup is a set equipped with an associative binary operation.
//!
//! # Algebraic Laws
//!
//! For a type to implement `Semigroup`, it must satisfy:
//!
//! - **Associativity**: `(a ∘ b) ∘ c = a ∘ (b ∘ c)` for all `a, b, c`
//!
//! # Examples
//!
//! ```
//! use acls_rs::algebra::Semigroup;
//!
//! // PermissionSet implements Semigroup via union
//! # use acls_rs::permission::{AtomicPermission, PermissionSet};
//! let perms1 = PermissionSet::from([
//!     AtomicPermission::new("file", "read"),
//! ]);
//! let perms2 = PermissionSet::from([
//!     AtomicPermission::new("file", "write"),
//! ]);
//!
//! let combined = perms1.combine(perms2);
//! // combined contains both read and write
//! ```

use std::collections::BTreeSet;

/// A semigroup: a set with an associative binary operation.
///
/// # Laws
///
/// Implementations must satisfy:
/// - **Associativity**: `(a.combine(b)).combine(c) == a.combine(b.combine(c))`
pub trait Semigroup: Sized {
    /// Combine two elements associatively.
    ///
    /// This operation must be associative: `(a ∘ b) ∘ c = a ∘ (b ∘ c)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use acls_rs::algebra::Semigroup;
    /// # use acls_rs::permission::{AtomicPermission, PermissionSet};
    ///
    /// let a = PermissionSet::from([AtomicPermission::new("a", "read")]);
    /// let b = PermissionSet::from([AtomicPermission::new("b", "read")]);
    /// let c = PermissionSet::from([AtomicPermission::new("c", "read")]);
    ///
    /// // Associativity
    /// let left = a.clone().combine(b.clone()).combine(c.clone());
    /// let right = a.combine(b.combine(c));
    /// assert_eq!(left, right);
    /// ```
    fn combine(self, other: Self) -> Self;

    /// Combine all elements from an iterator.
    ///
    /// Returns `None` if the iterator is empty, otherwise returns `Some` containing
    /// the result of combining all elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use acls_rs::algebra::Semigroup;
    /// # use acls_rs::permission::{AtomicPermission, PermissionSet};
    ///
    /// let sets = vec![
    ///     PermissionSet::from([AtomicPermission::new("a", "read")]),
    ///     PermissionSet::from([AtomicPermission::new("b", "read")]),
    ///     PermissionSet::from([AtomicPermission::new("c", "read")]),
    /// ];
    ///
    /// let combined = PermissionSet::combine_all(sets).unwrap();
    /// assert_eq!(combined.len(), 3);
    /// ```
    fn combine_all<I: IntoIterator<Item = Self>>(iter: I) -> Option<Self> {
        iter.into_iter().reduce(|a, b| a.combine(b))
    }
}

// Blanket implementation for BTreeSet (used internally by PermissionSet)
impl<T: Ord + Clone> Semigroup for BTreeSet<T> {
    #[inline]
    fn combine(self, other: Self) -> Self {
        self.union(&other).cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_btreeset_semigroup_associativity() {
        let a: BTreeSet<i32> = [1, 2].iter().cloned().collect();
        let b: BTreeSet<i32> = [2, 3].iter().cloned().collect();
        let c: BTreeSet<i32> = [3, 4].iter().cloned().collect();

        let left = a.clone().combine(b.clone()).combine(c.clone());
        let right = a.combine(b.combine(c));

        assert_eq!(left, right);
    }

    #[test]
    fn test_combine_all() {
        let sets = vec![
            [1, 2].iter().cloned().collect::<BTreeSet<_>>(),
            [3, 4].iter().cloned().collect::<BTreeSet<_>>(),
            [5].iter().cloned().collect::<BTreeSet<_>>(),
        ];

        let combined = BTreeSet::combine_all(sets).unwrap();
        assert_eq!(combined, [1, 2, 3, 4, 5].iter().cloned().collect());
    }

    #[test]
    fn test_combine_all_empty() {
        let empty: Vec<BTreeSet<i32>> = vec![];
        assert_eq!(BTreeSet::combine_all(empty), None);
    }
}
