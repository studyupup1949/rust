//! Semilattice and lattice algebraic structures.
//!
//! Semilattices extend monoids with meet (∧) and join (∨) operations that form
//! partial orders. Lattices combine both meet and join semilattices.

use super::Monoid;

/// A meet-semilattice: a partially ordered set with a greatest lower bound operation.
///
/// # Algebraic Laws
///
/// For a type to implement `MeetSemilattice`, it must satisfy:
///
/// - **Associativity**: `(a ∧ b) ∧ c = a ∧ (b ∧ c)`
/// - **Commutativity**: `a ∧ b = b ∧ a`
/// - **Idempotence**: `a ∧ a = a`
/// - **Partial Order**: `a ≤ b` iff `a ∧ b = a`
///
/// # Examples
///
/// ```
/// use acls_rs::algebra::MeetSemilattice;
/// use acls_rs::permission::{AtomicPermission, PermissionSet};
///
/// let perms1 = PermissionSet::from([
///     AtomicPermission::new("file", "read"),
///     AtomicPermission::new("file", "write"),
/// ]);
/// let perms2 = PermissionSet::from([
///     AtomicPermission::new("file", "read"),
/// ]);
///
/// // Meet is intersection (most restrictive)
/// let intersection = perms1.meet(perms2);
/// assert_eq!(intersection.len(), 1);
/// ```
pub trait MeetSemilattice: Monoid + PartialOrd {
    /// Compute the greatest lower bound (meet) of two elements.
    ///
    /// The meet operation `a ∧ b` returns the greatest element that is less than
    /// or equal to both `a` and `b`.
    ///
    /// For permission sets, meet is intersection (most restrictive combination).
    ///
    /// # Laws
    ///
    /// - **Associativity**: `(a ∧ b) ∧ c = a ∧ (b ∧ c)`
    /// - **Commutativity**: `a ∧ b = b ∧ a`
    /// - **Idempotence**: `a ∧ a = a`
    fn meet(self, other: Self) -> Self;
}

/// A join-semilattice: a partially ordered set with a least upper bound operation.
///
/// # Algebraic Laws
///
/// For a type to implement `JoinSemilattice`, it must satisfy:
///
/// - **Associativity**: `(a ∨ b) ∨ c = a ∨ (b ∨ c)`
/// - **Commutativity**: `a ∨ b = b ∨ a`
/// - **Idempotence**: `a ∨ a = a`
/// - **Partial Order**: `a ≤ b` iff `a ∨ b = b`
///
/// # Examples
///
/// ```
/// use acls_rs::algebra::JoinSemilattice;
/// use acls_rs::permission::{AtomicPermission, PermissionSet};
///
/// let perms1 = PermissionSet::from([
///     AtomicPermission::new("file", "read"),
/// ]);
/// let perms2 = PermissionSet::from([
///     AtomicPermission::new("file", "write"),
/// ]);
///
/// // Join is union (least restrictive)
/// let union = perms1.join(perms2);
/// assert_eq!(union.len(), 2);
/// ```
pub trait JoinSemilattice: Monoid + PartialOrd {
    /// Compute the least upper bound (join) of two elements.
    ///
    /// The join operation `a ∨ b` returns the smallest element that is greater than
    /// or equal to both `a` and `b`.
    ///
    /// For permission sets, join is union (least restrictive combination).
    ///
    /// # Laws
    ///
    /// - **Associativity**: `(a ∨ b) ∨ c = a ∨ (b ∨ c)`
    /// - **Commutativity**: `a ∨ b = b ∨ a`
    /// - **Idempotence**: `a ∨ a = a`
    fn join(self, other: Self) -> Self;
}

/// A bounded meet-semilattice with a top element.
///
/// The top element `⊤` is the greatest element in the partial order:
/// `a ∧ ⊤ = a` for all `a`.
pub trait BoundedMeetSemilattice: MeetSemilattice {
    /// Return the top element (greatest element).
    ///
    /// The top element must satisfy: `a.meet(Self::top()) == a` for all `a`.
    fn top() -> Self;
}

/// A bounded join-semilattice with a bottom element.
///
/// The bottom element `⊥` is the least element in the partial order:
/// `a ∨ ⊥ = a` for all `a`.
pub trait BoundedJoinSemilattice: JoinSemilattice {
    /// Return the bottom element (least element).
    ///
    /// The bottom element must satisfy: `a.join(Self::bottom()) == a` for all `a`.
    fn bottom() -> Self;
}

/// A lattice: a partially ordered set with both meet and join operations.
///
/// # Algebraic Laws
///
/// In addition to meet and join semilattice laws, a lattice must satisfy:
///
/// - **Absorption**: `a ∧ (a ∨ b) = a` and `a ∨ (a ∧ b) = a`
///
/// # Examples
///
/// ```
/// use acls_rs::algebra::Lattice;
/// use acls_rs::permission::{AtomicPermission, PermissionSet};
///
/// // PermissionSet is a Lattice
/// let perms: PermissionSet = PermissionSet::from([
///     AtomicPermission::new("file", "read"),
/// ]);
///
/// // Lattice operations are available via MeetSemilattice and JoinSemilattice traits
/// ```
pub trait Lattice: MeetSemilattice + JoinSemilattice {}

// Blanket implementation: any type with both meet and join is a lattice
impl<T: MeetSemilattice + JoinSemilattice> Lattice for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algebra::Semigroup;
    use std::collections::BTreeSet;

    // Implement semilattices for BTreeSet for testing
    impl<T: Ord + Clone> MeetSemilattice for BTreeSet<T> {
        fn meet(self, other: Self) -> Self {
            self.intersection(&other).cloned().collect()
        }
    }

    impl<T: Ord + Clone> JoinSemilattice for BTreeSet<T> {
        fn join(self, other: Self) -> Self {
            self.combine(other) // Reuse semigroup union
        }
    }

    #[test]
    fn test_meet_commutativity() {
        let a: BTreeSet<i32> = [1, 2, 3].iter().cloned().collect();
        let b: BTreeSet<i32> = [2, 3, 4].iter().cloned().collect();

        assert_eq!(a.clone().meet(b.clone()), b.meet(a));
    }

    #[test]
    fn test_meet_associativity() {
        let a: BTreeSet<i32> = [1, 2, 3].iter().cloned().collect();
        let b: BTreeSet<i32> = [2, 3, 4].iter().cloned().collect();
        let c: BTreeSet<i32> = [3, 4, 5].iter().cloned().collect();

        let left = a.clone().meet(b.clone()).meet(c.clone());
        let right = a.meet(b.meet(c));

        assert_eq!(left, right);
    }

    #[test]
    fn test_meet_idempotence() {
        let a: BTreeSet<i32> = [1, 2, 3].iter().cloned().collect();

        assert_eq!(a.clone().meet(a.clone()), a);
    }

    #[test]
    fn test_join_commutativity() {
        let a: BTreeSet<i32> = [1, 2].iter().cloned().collect();
        let b: BTreeSet<i32> = [3, 4].iter().cloned().collect();

        assert_eq!(a.clone().join(b.clone()), b.join(a));
    }

    #[test]
    fn test_join_idempotence() {
        let a: BTreeSet<i32> = [1, 2, 3].iter().cloned().collect();

        assert_eq!(a.clone().join(a.clone()), a);
    }
}
