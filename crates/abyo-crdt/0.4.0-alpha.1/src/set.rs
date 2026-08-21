//! Set CRDT — Observed-Remove Set (OR-Set).
//!
//! Each `add` creates a unique tag (the op's `OpId`). An element is "in"
//! the set if it has at least one tag that has not been observed-removed.
//! `remove` drops every tag *currently observed* for the value — concurrent
//! adds (whose tags weren't observed) survive.
//!
//! ## Add-wins semantics
//!
//! Concurrent `add(x)` and `remove(x)` resolve to "x is in the set." This is
//! the standard "add wins" behavior preferred for most use cases (e.g. in
//! collaborative tag editing).
//!
//! ## Quick start
//!
//! ```
//! use abyo_crdt::Set;
//!
//! let mut alice: Set<&str> = Set::new(1);
//! let mut bob: Set<&str> = Set::new(2);
//!
//! alice.add("apple");
//! alice.add("pear");
//! bob.merge(&alice);
//!
//! // Concurrent: alice removes "apple", bob re-adds "apple".
//! alice.remove(&"apple");
//! bob.add("apple");
//!
//! alice.merge(&bob);
//! bob.merge(&alice);
//!
//! // Add wins: "apple" is still in the set.
//! assert!(alice.contains(&"apple"));
//! assert!(bob.contains(&"apple"));
//! ```

use crate::{
    error::Error,
    id::{OpId, ReplicaId},
    version::VersionVector,
};
use smallvec::SmallVec;
use std::collections::HashMap;
use std::hash::Hash;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Public op type
// ---------------------------------------------------------------------------

/// A single [`Set`] CRDT operation.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SetOp<T> {
    /// Add `value` with a unique tag (this op's id).
    Add {
        /// Op id (also serves as the unique tag for this addition).
        id: OpId,
        /// Value being added.
        value: T,
    },
    /// Remove `value`, dropping the listed tags.
    ///
    /// `tags` is exactly the set of add-tags this replica observed for `value`
    /// at the moment of removal. Concurrent adds (whose tags aren't in this
    /// list) survive — that's the "add wins" rule.
    Remove {
        /// Op id of this remove.
        id: OpId,
        /// Value being removed.
        value: T,
        /// Tags being dropped.
        tags: Vec<OpId>,
    },
}

impl<T> SetOp<T> {
    /// The id of this op.
    #[must_use]
    pub fn id(&self) -> OpId {
        match self {
            SetOp::Add { id, .. } | SetOp::Remove { id, .. } => *id,
        }
    }
}

// ---------------------------------------------------------------------------
// Set CRDT
// ---------------------------------------------------------------------------

/// OR-Set CRDT. See the module docs for semantics.
#[derive(Clone, Debug)]
pub struct Set<T: Eq + Hash + Clone> {
    replica: ReplicaId,
    clock: u64,
    /// `value → set of currently-live tags`. Empty `SmallVec` means the value
    /// is not in the set (could be never-added or fully removed).
    tags: HashMap<T, SmallVec<[OpId; 2]>>,
    log: Vec<SetOp<T>>,
    version: VersionVector,
}

impl<T: Eq + Hash + Clone> Set<T> {
    /// Create an empty set.
    #[must_use]
    pub fn new(replica: ReplicaId) -> Self {
        Self {
            replica,
            clock: 0,
            tags: HashMap::new(),
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

    /// Number of distinct values currently in the set.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tags.values().filter(|t| !t.is_empty()).count()
    }

    /// Is the set empty?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Does the set contain `value`?
    pub fn contains(&self, value: &T) -> bool {
        self.tags.get(value).is_some_and(|t| !t.is_empty())
    }

    /// Iterate over values currently in the set.
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.tags
            .iter()
            .filter_map(|(v, t)| if t.is_empty() { None } else { Some(v) })
    }

    /// Add `value`. If it's already present, an additional tag is created;
    /// removing it then requires a remove op that observes both tags
    /// (which always happens when removes carry the tag list at remove-time).
    pub fn add(&mut self, value: T) -> SetOp<T> {
        self.clock = self
            .clock
            .checked_add(1)
            .expect("Lamport clock overflow (>2^64 ops)");
        let id = OpId::new(self.clock, self.replica);
        let op = SetOp::Add {
            id,
            value: value.clone(),
        };
        self.tags.entry(value).or_default().push(id);
        self.version.observe(id);
        self.log.push(op.clone());
        op
    }

    /// Remove `value`. No-op (returns `None`) if the value is not present.
    pub fn remove(&mut self, value: &T) -> Option<SetOp<T>> {
        let observed: Vec<OpId> = match self.tags.get(value) {
            Some(t) if !t.is_empty() => t.iter().copied().collect(),
            _ => return None,
        };
        self.clock = self
            .clock
            .checked_add(1)
            .expect("Lamport clock overflow (>2^64 ops)");
        let id = OpId::new(self.clock, self.replica);
        let op = SetOp::Remove {
            id,
            value: value.clone(),
            tags: observed.clone(),
        };
        // Drop the observed tags. Any concurrent-add tags not in `observed` survive.
        if let Some(slot) = self.tags.get_mut(value) {
            slot.retain(|t| !observed.contains(t));
        }
        self.version.observe(id);
        self.log.push(op.clone());
        Some(op)
    }

    /// Apply a remote op. Idempotent.
    pub fn apply(&mut self, op: SetOp<T>) -> Result<(), Error> {
        let op_id = op.id();
        if self.version.contains(op_id) {
            return Ok(());
        }
        match &op {
            SetOp::Add { id, value } => {
                self.tags.entry(value.clone()).or_default().push(*id);
            }
            SetOp::Remove { id: _, value, tags } => {
                if let Some(slot) = self.tags.get_mut(value) {
                    slot.retain(|t| !tags.contains(t));
                }
            }
        }
        self.version.observe(op_id);
        self.clock = self.clock.max(op_id.counter);
        self.log.push(op);
        Ok(())
    }

    /// Merge all of `other`'s state into `self`.
    pub fn merge(&mut self, other: &Self) {
        let mut to_apply: Vec<&SetOp<T>> = other
            .log
            .iter()
            .filter(|op| !self.version.contains(op.id()))
            .collect();
        to_apply.sort_by_key(|op| op.id());
        for op in to_apply {
            self.apply(op.clone()).expect("set apply cannot fail");
        }
    }

    /// All ops in this set's log.
    #[must_use]
    pub fn ops(&self) -> &[SetOp<T>] {
        &self.log
    }

    /// Iterate over ops not yet seen by `since`.
    pub fn ops_since<'a>(
        &'a self,
        since: &'a VersionVector,
    ) -> impl Iterator<Item = &'a SetOp<T>> + 'a {
        self.log.iter().filter(move |op| !since.contains(op.id()))
    }

    /// This replica's current version vector.
    #[must_use]
    pub fn version(&self) -> &VersionVector {
        &self.version
    }
}

impl<T: Eq + Hash + Clone> Default for Set<T> {
    fn default() -> Self {
        Self::new(0)
    }
}

// ---------------------------------------------------------------------------
// Serde
// ---------------------------------------------------------------------------

#[cfg(feature = "serde")]
#[derive(Serialize, Deserialize)]
struct SetSnapshot<T> {
    replica: ReplicaId,
    clock: u64,
    tags: Vec<(T, SmallVec<[OpId; 2]>)>,
    version: VersionVector,
    log: Vec<SetOp<T>>,
}

#[cfg(feature = "serde")]
impl<T> Serialize for Set<T>
where
    T: Eq + Hash + Clone + Serialize,
{
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let snap = SetSnapshot {
            replica: self.replica,
            clock: self.clock,
            tags: self
                .tags
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            version: self.version.clone(),
            log: self.log.clone(),
        };
        snap.serialize(ser)
    }
}

#[cfg(feature = "serde")]
impl<'de, T> Deserialize<'de> for Set<T>
where
    T: Eq + Hash + Clone + Deserialize<'de>,
{
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let snap = SetSnapshot::<T>::deserialize(de)?;
        Ok(Set {
            replica: snap.replica,
            clock: snap.clock,
            tags: snap.tags.into_iter().collect(),
            version: snap.version,
            log: snap.log,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_set() {
        let s: Set<&str> = Set::new(1);
        assert!(s.is_empty());
        assert!(!s.contains(&"x"));
    }

    #[test]
    fn add_and_contains() {
        let mut s: Set<&str> = Set::new(1);
        s.add("a");
        s.add("b");
        assert!(s.contains(&"a"));
        assert!(s.contains(&"b"));
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn remove_drops_value() {
        let mut s: Set<&str> = Set::new(1);
        s.add("a");
        let op = s.remove(&"a");
        assert!(op.is_some());
        assert!(!s.contains(&"a"));
    }

    #[test]
    fn remove_absent_returns_none() {
        let mut s: Set<&str> = Set::new(1);
        assert!(s.remove(&"x").is_none());
    }

    #[test]
    fn add_wins_over_concurrent_remove() {
        let mut a: Set<&str> = Set::new(1);
        let mut b: Set<&str> = Set::new(2);
        a.add("x");
        b.merge(&a);

        a.remove(&"x"); // a removes the tag it knows about
        b.add("x"); // b adds a new tag concurrently

        let mut a2 = a.clone();
        a2.merge(&b);
        let mut b2 = b.clone();
        b2.merge(&a);

        // Both replicas: x present (b's add tag survives a's remove).
        assert!(a2.contains(&"x"));
        assert!(b2.contains(&"x"));
    }

    #[test]
    fn double_add_then_single_remove_keeps_value() {
        // a adds "x" twice (two different tags). a removes "x" — drops both
        // tags it observed. After remove, "x" is gone.
        let mut a: Set<&str> = Set::new(1);
        a.add("x");
        a.add("x");
        a.remove(&"x");
        assert!(!a.contains(&"x"));
    }

    #[test]
    fn idempotent_apply() {
        let mut a: Set<&str> = Set::new(1);
        let op1 = a.add("x");
        let op2 = a.add("y");

        let mut b: Set<&str> = Set::new(2);
        b.apply(op1.clone()).unwrap();
        b.apply(op2.clone()).unwrap();
        b.apply(op1).unwrap();
        b.apply(op2).unwrap();
        assert!(b.contains(&"x"));
        assert!(b.contains(&"y"));
    }

    #[test]
    fn merge_is_commutative() {
        let mut a1: Set<&str> = Set::new(1);
        let mut a2: Set<&str> = Set::new(1);
        let mut b1: Set<&str> = Set::new(2);
        let mut b2: Set<&str> = Set::new(2);
        a1.add("x");
        a2.add("x");
        b1.add("y");
        b2.add("y");
        a1.merge(&b1);
        b2.merge(&a2);
        assert_eq!(a1.len(), b2.len());
        assert!(a1.contains(&"x") && a1.contains(&"y"));
        assert!(b2.contains(&"x") && b2.contains(&"y"));
    }
}
