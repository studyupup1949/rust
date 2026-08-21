//! Types for values with ordering constraints.
//!
//! This module provides wrappers that enforce invariants at construction time
//! rather than at every call site. If you need a value that stays within a
//! range for its entire lifetime, [`Bounded`] removes that burden from the caller.

mod bounded;
pub use bounded::Bounded;
