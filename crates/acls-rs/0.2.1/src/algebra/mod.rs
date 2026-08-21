//! Algebraic structures and traits.
//!
//! This module provides the algebraic foundation for the permissions system:
//!
//! - [`Semigroup`]: Associative binary operation
//! - [`Monoid`]: Semigroup with identity element
//! - [`MeetSemilattice`]: Partial order with greatest lower bound (meet/∧)
//! - [`JoinSemilattice`]: Partial order with least upper bound (join/∨)
//! - [`Lattice`]: Both meet and join semilattices
//! - [`MonoidAction`]: Monoid elements acting on other types
//!
//! All algebraic laws are documented in the respective trait definitions and
//! verified via property-based tests.
//!
//! # Why Algebraic Structures?
//!
//! Using algebraic structures provides:
//!
//! - **Predictability**: Operations follow mathematical laws
//! - **Composability**: Small pieces combine in well-defined ways
//! - **Correctness**: Properties can be formally verified
//! - **Genericity**: Algorithms work on any type satisfying the laws
//!
//! # Examples
//!
//! ```
//! use acls_rs::algebra::{Monoid, Semigroup, MeetSemilattice, JoinSemilattice};
//! use acls_rs::permission::{AtomicPermission, PermissionSet};
//!
//! // Create permission sets
//! let perms1 = PermissionSet::from([
//!     AtomicPermission::new("file", "read"),
//! ]);
//! let perms2 = PermissionSet::from([
//!     AtomicPermission::new("file", "write"),
//! ]);
//!
//! // Semigroup: combine via union
//! let combined = perms1.clone().combine(perms2.clone());
//!
//! // Monoid: identity is empty set
//! let empty = PermissionSet::identity();
//! assert_eq!(perms1.clone().combine(empty), perms1);
//!
//! // Join semilattice: union (least restrictive)
//! let union = perms1.clone().join(perms2.clone());
//!
//! // Meet semilattice: intersection (most restrictive)
//! let intersection = perms1.meet(perms2);
//! ```

mod action;
mod monoid;
mod semigroup;
mod semilattice;

pub use action::MonoidAction;
pub use monoid::Monoid;
pub use semigroup::Semigroup;
pub use semilattice::{
    BoundedJoinSemilattice, BoundedMeetSemilattice, JoinSemilattice, Lattice, MeetSemilattice,
};
