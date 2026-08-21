//! Counter CRDT — signed integer with concurrent increment/decrement support.
//!
//! Implemented as a delta-based signed counter with version-vector idempotency.
//! Each `add(delta)` produces an op carrying the delta and a unique [`OpId`];
//! applying an already-seen op is a no-op, so concurrent and replayed deltas
//! all resolve to the same final sum.
//!
//! Internally also tracks a per-replica positive/negative running sum so the
//! state is exposed in PN-Counter form for inspection.
//!
//! ## Quick start
//!
//! ```
//! use abyo_crdt::Counter;
//!
//! let mut alice = Counter::new(1);
//! let mut bob = Counter::new(2);
//!
//! alice.add(5);
//! bob.add(-2);
//! bob.add(10);
//!
//! alice.merge(&bob);
//! bob.merge(&alice);
//!
//! assert_eq!(alice.value(), 13);
//! assert_eq!(bob.value(), 13);
//! ```

use crate::{
    error::Error,
    id::{OpId, ReplicaId},
    version::VersionVector,
};
use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Public op type
// ---------------------------------------------------------------------------

/// A single [`Counter`] CRDT operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CounterOp {
    /// Unique id of this op (drives idempotency).
    pub id: OpId,
    /// Signed delta to apply.
    pub delta: i64,
}

impl CounterOp {
    /// The id of this op.
    #[must_use]
    pub fn id(&self) -> OpId {
        self.id
    }
}

// ---------------------------------------------------------------------------
// Counter CRDT
// ---------------------------------------------------------------------------

/// PN-Counter CRDT (signed). See the module docs for semantics.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Counter {
    replica: ReplicaId,
    clock: u64,
    /// Per-replica positive cumulative sum (for inspection).
    p: HashMap<ReplicaId, u128>,
    /// Per-replica negative cumulative sum (for inspection).
    n: HashMap<ReplicaId, u128>,
    /// Cached signed sum, kept in sync with applied ops.
    value: i128,
    log: Vec<CounterOp>,
    version: VersionVector,
}

impl Counter {
    /// Create a counter at zero for the given replica.
    #[must_use]
    pub fn new(replica: ReplicaId) -> Self {
        Self {
            replica,
            clock: 0,
            p: HashMap::new(),
            n: HashMap::new(),
            value: 0,
            log: Vec::new(),
            version: VersionVector::new(),
        }
    }

    /// Create a new instance with a random [`ReplicaId`] from OS entropy.
    /// See [`crate::new_replica_id`].
    #[must_use]
    pub fn new_random() -> Self {
        Self::new(crate::id::new_replica_id())
    }

    /// This replica's id.
    #[must_use]
    pub fn replica_id(&self) -> ReplicaId {
        self.replica
    }

    /// Current signed value.
    #[must_use]
    pub fn value(&self) -> i128 {
        self.value
    }

    /// Sum of all positive contributions across all replicas.
    #[must_use]
    pub fn positive_total(&self) -> u128 {
        self.p.values().sum()
    }

    /// Sum of all negative contributions across all replicas.
    #[must_use]
    pub fn negative_total(&self) -> u128 {
        self.n.values().sum()
    }

    /// Apply a signed `delta` to this replica. Returns the generated op.
    pub fn add(&mut self, delta: i64) -> CounterOp {
        self.clock = self
            .clock
            .checked_add(1)
            .expect("Lamport clock overflow (>2^64 ops)");
        let id = OpId::new(self.clock, self.replica);
        let op = CounterOp { id, delta };
        self.apply_internal(id, delta);
        self.version.observe(id);
        self.log.push(op);
        op
    }

    /// Increment by an unsigned amount.
    pub fn increment(&mut self, by: u64) -> CounterOp {
        self.add(i64::try_from(by).expect("increment overflow"))
    }

    /// Decrement by an unsigned amount.
    pub fn decrement(&mut self, by: u64) -> CounterOp {
        self.add(-i64::try_from(by).expect("decrement overflow"))
    }

    /// Apply a remote op. Idempotent.
    pub fn apply(&mut self, op: CounterOp) -> Result<(), Error> {
        if self.version.contains(op.id) {
            return Ok(());
        }
        self.apply_internal(op.id, op.delta);
        self.version.observe(op.id);
        self.clock = self.clock.max(op.id.counter);
        self.log.push(op);
        Ok(())
    }

    /// Merge all ops from `other` we haven't seen.
    pub fn merge(&mut self, other: &Self) {
        let mut to_apply: Vec<&CounterOp> = other
            .log
            .iter()
            .filter(|op| !self.version.contains(op.id))
            .collect();
        to_apply.sort_by_key(|op| op.id);
        for op in to_apply {
            self.apply(*op).expect("counter apply cannot fail");
        }
    }

    /// All ops in this counter's log.
    #[must_use]
    pub fn ops(&self) -> &[CounterOp] {
        &self.log
    }

    /// Iterate over ops not yet seen by `since`.
    pub fn ops_since<'a>(
        &'a self,
        since: &'a VersionVector,
    ) -> impl Iterator<Item = &'a CounterOp> + 'a {
        self.log.iter().filter(move |op| !since.contains(op.id))
    }

    /// This replica's current version vector.
    #[must_use]
    pub fn version(&self) -> &VersionVector {
        &self.version
    }

    fn apply_internal(&mut self, id: OpId, delta: i64) {
        self.value += i128::from(delta);
        if delta >= 0 {
            // delta is non-negative; cast to u64 is sign-safe.
            #[allow(clippy::cast_sign_loss)]
            let abs = delta as u64;
            *self.p.entry(id.replica).or_insert(0) += u128::from(abs);
        } else {
            *self.n.entry(id.replica).or_insert(0) += u128::from(delta.unsigned_abs());
        }
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_counter_is_zero() {
        let c = Counter::new(1);
        assert_eq!(c.value(), 0);
    }

    #[test]
    fn add_and_value() {
        let mut c = Counter::new(1);
        c.add(5);
        c.add(-2);
        c.add(10);
        assert_eq!(c.value(), 13);
    }

    #[test]
    fn increment_decrement_helpers() {
        let mut c = Counter::new(1);
        c.increment(10);
        c.decrement(3);
        assert_eq!(c.value(), 7);
    }

    #[test]
    fn merge_sums_concurrent_deltas() {
        let mut a = Counter::new(1);
        let mut b = Counter::new(2);
        a.add(10);
        b.add(20);
        let mut a2 = a.clone();
        a2.merge(&b);
        let mut b2 = b.clone();
        b2.merge(&a);
        assert_eq!(a2.value(), 30);
        assert_eq!(b2.value(), 30);
    }

    #[test]
    fn idempotent_apply() {
        let mut a = Counter::new(1);
        let op1 = a.add(5);
        let op2 = a.add(7);
        let mut b = Counter::new(2);
        b.apply(op1).unwrap();
        b.apply(op1).unwrap(); // duplicate
        b.apply(op2).unwrap();
        b.apply(op2).unwrap();
        assert_eq!(b.value(), 12);
    }

    #[test]
    fn pn_breakdown() {
        let mut a = Counter::new(1);
        let mut b = Counter::new(2);
        a.add(10);
        a.add(-3);
        b.add(20);
        b.add(-5);
        a.merge(&b);
        assert_eq!(a.value(), 22);
        assert_eq!(a.positive_total(), 30);
        assert_eq!(a.negative_total(), 8);
    }

    #[test]
    fn ops_since_returns_only_unseen() {
        let mut a = Counter::new(1);
        a.add(1);
        let v1 = a.version().clone();
        a.add(2);
        a.add(3);
        let new: Vec<&CounterOp> = a.ops_since(&v1).collect();
        assert_eq!(new.len(), 2);
    }
}
