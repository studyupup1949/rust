//! Map CRDT — Last-Write-Wins keyed dictionary.
//!
//! Each `(key, value)` pair carries a Lamport-style timestamp. On merge, the entry
//! with the highest timestamp wins per key. A delete is a tombstone with its
//! own timestamp; if its timestamp dominates the latest set, the key is
//! considered absent.
//!
//! ## Convergence
//!
//! Per-key resolution uses the `OpId` total order (counter, then replica),
//! which is a strictly monotonic Lamport clock. Concurrent set+set on the
//! same key resolves deterministically; concurrent set+remove resolves to
//! whichever has the higher `OpId`.
//!
//! ## Quick start
//!
//! ```
//! use abyo_crdt::Map;
//!
//! let mut alice: Map<String, i32> = Map::new(1);
//! let mut bob: Map<String, i32> = Map::new(2);
//!
//! alice.set("score".to_string(), 10);
//! bob.merge(&alice);
//!
//! // Concurrent edits to the same key.
//! alice.set("score".to_string(), 20);
//! bob.set("score".to_string(), 99);
//!
//! alice.merge(&bob);
//! bob.merge(&alice);
//!
//! // Both replicas converge — Lamport tiebreaker decides which value wins.
//! assert_eq!(alice.get(&"score".to_string()), bob.get(&"score".to_string()));
//! ```

use crate::{
    error::Error,
    id::{OpId, ReplicaId},
    version::VersionVector,
};
use std::collections::HashMap;
use std::hash::Hash;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Public op type
// ---------------------------------------------------------------------------

/// A single [`Map`] CRDT operation.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MapOp<K, V> {
    /// Set `key` to `value`.
    Set {
        /// Op id (also serves as the Lamport-style timestamp for conflict resolution).
        id: OpId,
        /// Key being set.
        key: K,
        /// Value being written.
        value: V,
    },
    /// Remove `key`.
    Remove {
        /// Op id.
        id: OpId,
        /// Key being removed.
        key: K,
    },
}

impl<K, V> MapOp<K, V> {
    /// The id of this op.
    #[must_use]
    pub fn id(&self) -> OpId {
        match self {
            MapOp::Set { id, .. } | MapOp::Remove { id, .. } => *id,
        }
    }

    /// The key this op affects.
    #[must_use]
    pub fn key(&self) -> &K {
        match self {
            MapOp::Set { key, .. } | MapOp::Remove { key, .. } => key,
        }
    }
}

// ---------------------------------------------------------------------------
// Internal entry
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct Entry<V> {
    /// Op id of the op that "wrote" this entry. The op with the highest
    /// `op_id` for a given key wins.
    op_id: OpId,
    /// `Some(v)` if the latest op was a set; `None` if it was a remove.
    value: Option<V>,
}

// ---------------------------------------------------------------------------
// Map CRDT
// ---------------------------------------------------------------------------

/// LWW-Map CRDT. See the module docs for semantics.
#[derive(Clone, Debug)]
pub struct Map<K: Eq + Hash + Clone, V: Clone> {
    replica: ReplicaId,
    clock: u64,
    entries: HashMap<K, Entry<V>>,
    log: Vec<MapOp<K, V>>,
    version: VersionVector,
}

impl<K: Eq + Hash + Clone, V: Clone> Map<K, V> {
    /// Create an empty map for the given replica.
    #[must_use]
    pub fn new(replica: ReplicaId) -> Self {
        Self {
            replica,
            clock: 0,
            entries: HashMap::new(),
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

    /// Number of visible (non-tombstoned) entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.values().filter(|e| e.value.is_some()).count()
    }

    /// Are there any visible entries?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Look up the value for `key`, or `None` if absent / tombstoned.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.entries.get(key).and_then(|e| e.value.as_ref())
    }

    /// Iterate visible `(key, value)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> + '_ {
        self.entries
            .iter()
            .filter_map(|(k, e)| e.value.as_ref().map(|v| (k, v)))
    }

    /// Iterate visible keys.
    pub fn keys(&self) -> impl Iterator<Item = &K> + '_ {
        self.iter().map(|(k, _)| k)
    }

    /// Iterate visible values.
    pub fn values(&self) -> impl Iterator<Item = &V> + '_ {
        self.iter().map(|(_, v)| v)
    }

    /// Does the map contain `key`?
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Set `key` to `value`. Returns the generated [`MapOp`].
    pub fn set(&mut self, key: K, value: V) -> MapOp<K, V> {
        self.clock = self
            .clock
            .checked_add(1)
            .expect("Lamport clock overflow (>2^64 ops)");
        let id = OpId::new(self.clock, self.replica);
        let op = MapOp::Set {
            id,
            key: key.clone(),
            value: value.clone(),
        };
        self.upsert(id, key, Some(value));
        self.version.observe(id);
        self.log.push(op.clone());
        op
    }

    /// Remove `key`. Always emits an op (even if the key was absent), so the
    /// remove can be replicated.
    pub fn remove(&mut self, key: K) -> MapOp<K, V> {
        self.clock = self
            .clock
            .checked_add(1)
            .expect("Lamport clock overflow (>2^64 ops)");
        let id = OpId::new(self.clock, self.replica);
        let op = MapOp::Remove {
            id,
            key: key.clone(),
        };
        self.upsert(id, key, None);
        self.version.observe(id);
        self.log.push(op.clone());
        op
    }

    /// Apply a remote operation. Idempotent.
    pub fn apply(&mut self, op: MapOp<K, V>) -> Result<(), Error> {
        let op_id = op.id();
        if self.version.contains(op_id) {
            return Ok(());
        }
        match &op {
            MapOp::Set { id, key, value } => {
                self.upsert(*id, key.clone(), Some(value.clone()));
            }
            MapOp::Remove { id, key } => {
                self.upsert(*id, key.clone(), None);
            }
        }
        self.version.observe(op_id);
        self.clock = self.clock.max(op_id.counter);
        self.log.push(op);
        Ok(())
    }

    /// Merge all of `other`'s state into `self`. Equivalent to applying
    /// every op in `other.log` that we haven't seen, in `OpId` order.
    pub fn merge(&mut self, other: &Self) {
        let mut to_apply: Vec<&MapOp<K, V>> = other
            .log
            .iter()
            .filter(|op| !self.version.contains(op.id()))
            .collect();
        to_apply.sort_by_key(|op| op.id());
        for op in to_apply {
            // Apply can never fail in a valid log replay.
            self.apply(op.clone())
                .expect("corrupt op log in merge source");
        }
    }

    /// All ops observed by this replica, in observation order.
    #[must_use]
    pub fn ops(&self) -> &[MapOp<K, V>] {
        &self.log
    }

    /// Iterate over ops not yet seen by `since`.
    pub fn ops_since<'a>(
        &'a self,
        since: &'a VersionVector,
    ) -> impl Iterator<Item = &'a MapOp<K, V>> + 'a {
        self.log.iter().filter(move |op| !since.contains(op.id()))
    }

    /// This replica's current version vector.
    #[must_use]
    pub fn version(&self) -> &VersionVector {
        &self.version
    }

    // -----------------------------------------------------------------------
    // Internals
    // -----------------------------------------------------------------------

    /// LWW write: only updates the entry if `id` dominates the existing one.
    fn upsert(&mut self, id: OpId, key: K, value: Option<V>) {
        match self.entries.get_mut(&key) {
            Some(entry) if id <= entry.op_id => {
                // Existing entry has a higher (or equal) op_id — keep it.
                // Equality only happens when the same op id arrives twice,
                // which `apply` already guards via the version-vector check;
                // included here for defense in depth.
            }
            Some(entry) => {
                entry.op_id = id;
                entry.value = value;
            }
            None => {
                self.entries.insert(key, Entry { op_id: id, value });
            }
        }
    }
}

impl<K: Eq + Hash + Clone, V: Clone> Default for Map<K, V> {
    fn default() -> Self {
        Self::new(0)
    }
}

// ---------------------------------------------------------------------------
// Serde
// ---------------------------------------------------------------------------
//
// HashMap<K, _> with a non-string K (e.g. `(u64, u64)`) won't survive a JSON
// round-trip. We serialize via a Vec<(K, Entry<V>)> snapshot.

#[cfg(feature = "serde")]
#[derive(Serialize, Deserialize)]
struct MapSnapshot<K, V> {
    replica: ReplicaId,
    clock: u64,
    entries: Vec<(K, Entry<V>)>,
    version: VersionVector,
    log: Vec<MapOp<K, V>>,
}

#[cfg(feature = "serde")]
impl<K, V> Serialize for Map<K, V>
where
    K: Eq + Hash + Clone + Serialize,
    V: Clone + Serialize,
{
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let entries: Vec<(K, Entry<V>)> = self
            .entries
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let snap = MapSnapshot {
            replica: self.replica,
            clock: self.clock,
            entries,
            version: self.version.clone(),
            log: self.log.clone(),
        };
        snap.serialize(ser)
    }
}

#[cfg(feature = "serde")]
impl<'de, K, V> Deserialize<'de> for Map<K, V>
where
    K: Eq + Hash + Clone + Deserialize<'de>,
    V: Clone + Deserialize<'de>,
{
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let snap = MapSnapshot::<K, V>::deserialize(de)?;
        Ok(Map {
            replica: snap.replica,
            clock: snap.clock,
            entries: snap.entries.into_iter().collect(),
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
    fn empty_map() {
        let m: Map<String, i32> = Map::new(1);
        assert!(m.is_empty());
        assert_eq!(m.len(), 0);
        assert_eq!(m.get(&"k".to_string()), None);
    }

    #[test]
    fn set_and_get() {
        let mut m: Map<String, i32> = Map::new(1);
        m.set("a".into(), 1);
        m.set("b".into(), 2);
        assert_eq!(m.get(&"a".into()), Some(&1));
        assert_eq!(m.get(&"b".into()), Some(&2));
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn overwrite_in_one_replica() {
        let mut m: Map<&'static str, i32> = Map::new(1);
        m.set("a", 1);
        m.set("a", 2);
        m.set("a", 3);
        assert_eq!(m.get(&"a"), Some(&3));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn remove_drops_value() {
        let mut m: Map<&'static str, i32> = Map::new(1);
        m.set("a", 1);
        m.remove("a");
        assert!(!m.contains_key(&"a"));
        assert_eq!(m.len(), 0);
    }

    #[test]
    fn concurrent_set_lww_resolution() {
        let mut a: Map<&'static str, i32> = Map::new(1);
        let mut b: Map<&'static str, i32> = Map::new(2);

        a.set("k", 100);
        b.set("k", 200);

        let mut a2 = a.clone();
        a2.merge(&b);
        let mut b2 = b.clone();
        b2.merge(&a);

        // Both converge.
        assert_eq!(a2.get(&"k"), b2.get(&"k"));
        // The one with higher OpId wins.
        // a's op had counter=1, replica=1. b's op had counter=1, replica=2.
        // Counter ties → replica tiebreaker → b wins.
        assert_eq!(a2.get(&"k"), Some(&200));
    }

    #[test]
    fn set_beats_concurrent_remove_with_higher_id() {
        let mut a: Map<&'static str, i32> = Map::new(1);
        let mut b: Map<&'static str, i32> = Map::new(2);
        a.set("k", 1);
        b.merge(&a);

        a.remove("k"); // counter=2, replica=1
        b.set("k", 99); // counter=2, replica=2

        let mut a2 = a.clone();
        a2.merge(&b);
        let mut b2 = b.clone();
        b2.merge(&a);

        assert_eq!(a2.get(&"k"), b2.get(&"k"));
        // b's set has higher OpId (replica=2 > replica=1) → wins.
        assert_eq!(a2.get(&"k"), Some(&99));
    }

    #[test]
    fn idempotent_apply() {
        let mut a: Map<&'static str, i32> = Map::new(1);
        let op1 = a.set("k", 1);
        let op2 = a.set("j", 2);

        let mut b: Map<&'static str, i32> = Map::new(2);
        b.apply(op1.clone()).unwrap();
        b.apply(op2.clone()).unwrap();
        b.apply(op1).unwrap();
        b.apply(op2).unwrap();

        assert_eq!(b.len(), 2);
        assert_eq!(b.get(&"k"), Some(&1));
        assert_eq!(b.get(&"j"), Some(&2));
    }

    #[test]
    fn ops_since_returns_only_unseen() {
        let mut a: Map<&'static str, i32> = Map::new(1);
        a.set("k", 1);
        let v1 = a.version().clone();
        a.set("j", 2);

        let new: Vec<&MapOp<&'static str, i32>> = a.ops_since(&v1).collect();
        assert_eq!(new.len(), 1);
    }
}
