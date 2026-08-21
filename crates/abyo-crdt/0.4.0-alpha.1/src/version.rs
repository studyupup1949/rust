//! Version vectors — a summary of "what ops a replica has seen".
//!
//! Used by every CRDT in this crate to drive idempotent op application
//! and incremental sync via [`crate::List::ops_since`] et al.

use crate::id::{OpId, ReplicaId};
use std::collections::BTreeMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Summary of "what ops a replica has seen", indexed by replica.
///
/// `vv[replica]` is the highest `counter` ever observed from that replica.
/// Because `counter`s are Lamport timestamps that strictly increase along
/// any causal chain, "seen `(replica, counter)`" implies "seen everything
/// from that replica with counter ≤ that counter".
#[derive(Clone, Default, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VersionVector {
    /// `replica → highest seen counter`.
    inner: BTreeMap<ReplicaId, u64>,
}

impl VersionVector {
    /// Empty version vector.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Has this version vector seen op `id`?
    #[must_use]
    pub fn contains(&self, id: OpId) -> bool {
        self.inner
            .get(&id.replica)
            .is_some_and(|&v| v >= id.counter)
    }

    /// Number of distinct replicas seen.
    #[must_use]
    pub fn replica_count(&self) -> usize {
        self.inner.len()
    }

    /// Highest counter observed from `replica`, or 0 if none.
    #[must_use]
    pub fn get(&self, replica: ReplicaId) -> u64 {
        self.inner.get(&replica).copied().unwrap_or(0)
    }

    /// Record `id` as observed.
    pub fn observe(&mut self, id: OpId) {
        let entry = self.inner.entry(id.replica).or_insert(0);
        *entry = (*entry).max(id.counter);
    }

    /// Iterate `(replica, highest_counter)` pairs in replica-id order.
    pub fn iter_clocks(&self) -> impl Iterator<Item = (ReplicaId, u64)> + '_ {
        self.inner.iter().map(|(&r, &c)| (r, c))
    }
}
