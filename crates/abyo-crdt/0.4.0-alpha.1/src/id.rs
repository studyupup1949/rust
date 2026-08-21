//! Operation identifiers.
//!
//! `abyo-crdt` uses Lamport-style identifiers: every op carries a
//! `(counter, replica)` pair where `counter` advances both on local op
//! generation and on receipt of a remote op (`max(local, remote) + 1`).
//!
//! This means `OpId`s have a meaningful total order that respects causality:
//! if op A causally precedes op B, then `A.counter < B.counter`. Concurrent
//! ops are tiebroken by `replica`.

use core::cmp::Ordering;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Identifier for a replica (collaborator).
///
/// Each replica must have a unique `u64`. Use [`new_replica_id`] to
/// generate one from OS entropy at first launch and persist it; reusing
/// a replica id across two participants breaks correctness (and is now
/// detected at op-apply time as [`crate::Error::ReplicaIdConflict`]).
pub type ReplicaId = u64;

/// Generate a random [`ReplicaId`] from OS entropy.
///
/// 64 random bits gives a collision probability of ~10⁻¹⁹ for any two
/// replicas in practice. Persist the returned value (e.g. in app
/// settings) and reuse it across sessions on the same device.
///
/// # Panics
///
/// Panics if the OS RNG fails — typically only on very unusual platforms
/// where `getrandom` returns an error.
#[must_use]
pub fn new_replica_id() -> ReplicaId {
    let mut bytes = [0u8; 8];
    getrandom::fill(&mut bytes).expect("OS entropy unavailable");
    u64::from_le_bytes(bytes)
}

/// Globally unique identifier for a single CRDT operation.
///
/// `OpId` is `(counter, replica)` lexicographically. Since `counter` is a
/// Lamport timestamp that monotonically advances on every local op AND on
/// every remote op received, `counter` collisions only occur for genuinely
/// concurrent ops, which are then tiebroken by `replica`.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OpId {
    /// Lamport-style logical counter. Strictly monotonic per replica;
    /// also strictly monotonic along any causal chain.
    pub counter: u64,
    /// Replica that authored this op.
    pub replica: ReplicaId,
}

impl OpId {
    /// Construct an `OpId`.
    #[inline]
    #[must_use]
    pub const fn new(counter: u64, replica: ReplicaId) -> Self {
        Self { counter, replica }
    }
}

impl Ord for OpId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.counter
            .cmp(&other.counter)
            .then_with(|| self.replica.cmp(&other.replica))
    }
}

impl PartialOrd for OpId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordering_is_lex_counter_first() {
        let a = OpId::new(1, 99);
        let b = OpId::new(2, 1);
        let c = OpId::new(2, 50);
        assert!(a < b);
        assert!(b < c);
        assert!(a < c);
    }

    #[test]
    fn equality_requires_both_components() {
        assert_ne!(OpId::new(1, 1), OpId::new(1, 2));
        assert_ne!(OpId::new(1, 1), OpId::new(2, 1));
        assert_eq!(OpId::new(1, 1), OpId::new(1, 1));
    }
}
