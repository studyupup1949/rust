//! Monoid algebraic structure.
//!
//! A monoid is a semigroup with an identity element.
//!
//! # Algebraic Laws
//!
//! For a type to implement `Monoid`, it must satisfy:
//!
//! - **Associativity**: `(a ∘ b) ∘ c = a ∘ (b ∘ c)` (inherited from Semigroup)
//! - **Left Identity**: `e ∘ a = a` for all `a`
//! - **Right Identity**: `a ∘ e = a` for all `a`
//!
//! where `e` is the identity element.

use super::Semigroup;
use std::collections::BTreeSet;

/// A monoid: a semigroup with an identity element.
///
/// # Laws
///
/// Implementations must satisfy (in addition to semigroup laws):
/// - **Left Identity**: `Monoid::identity().combine(a) == a`
/// - **Right Identity**: `a.combine(Monoid::identity()) == a`
pub trait Monoid: Semigroup {
    /// Return the identity element.
    ///
    /// The identity element must satisfy:
    /// - `identity().combine(a) == a` (left identity)
    /// - `a.combine(identity()) == a` (right identity)
    ///
    /// # Examples
    ///
    /// ```
    /// use acls_rs::algebra::Monoid;
    /// use acls_rs::permission::PermissionSet;
    ///
    /// let empty = PermissionSet::identity();
    /// assert!(empty.is_empty());
    /// ```
    fn identity() -> Self;

    /// Combine all elements from an iterator, using identity for empty iterators.
    ///
    /// Unlike `Semigroup::combine_all`, this always returns a value, using the
    /// identity element when the iterator is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use acls_rs::algebra::Monoid;
    /// use acls_rs::permission::PermissionSet;
    ///
    /// let empty: Vec<PermissionSet> = vec![];
    /// let result = PermissionSet::combine_all_with_identity(empty);
    /// assert_eq!(result, PermissionSet::identity());
    /// ```
    fn combine_all_with_identity<I: IntoIterator<Item = Self>>(iter: I) -> Self {
        Self::combine_all(iter).unwrap_or_else(Self::identity)
    }
}

// Blanket implementation for BTreeSet
impl<T: Ord + Clone> Monoid for BTreeSet<T> {
    #[inline]
    fn identity() -> Self {
        BTreeSet::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algebra::Semigroup;

    #[test]
    fn test_btreeset_left_identity() {
        let a: BTreeSet<i32> = [1, 2, 3].iter().cloned().collect();
        let e = BTreeSet::identity();

        assert_eq!(e.combine(a.clone()), a);
    }

    #[test]
    fn test_btreeset_right_identity() {
        let a: BTreeSet<i32> = [1, 2, 3].iter().cloned().collect();
        let e = BTreeSet::identity();

        assert_eq!(a.clone().combine(e), a);
    }

    #[test]
    fn test_combine_all_with_identity_empty() {
        let empty: Vec<BTreeSet<i32>> = vec![];
        let result = BTreeSet::combine_all_with_identity(empty);

        assert_eq!(result, BTreeSet::identity());
    }

    #[test]
    fn test_combine_all_with_identity_nonempty() {
        let sets = vec![
            [1, 2].iter().cloned().collect::<BTreeSet<_>>(),
            [3, 4].iter().cloned().collect::<BTreeSet<_>>(),
        ];

        let result = BTreeSet::combine_all_with_identity(sets);
        assert_eq!(result, [1, 2, 3, 4].iter().cloned().collect());
    }
}
