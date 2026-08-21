//! List CRDT — the core data type of `abyo-crdt`.
//!
//! Implements the **Fugue-Maximal** positioning algorithm (Weidner 2023) over
//! an **Eg-walker**-style causal event log: every operation is recorded in
//! `log: Vec<ListOp<T>>` in the order it was observed, and the visible sequence
//! is computed by walking the tree formed by parent/side anchors.
//!
//! ## Algorithm summary
//!
//! Each item in the list is a node in an **ordered tree**:
//!
//! ```text
//! Item { id, parent: Option<OpId>, side: Left | Right, value, deletes }
//! ```
//!
//! - `parent = None` means a top-level item.
//! - Children of a node are split into **left children** (visited before the
//!   node in in-order traversal) and **right children** (visited after).
//! - Within a side, siblings are sorted by `OpId` ascending. This is a Lamport-
//!   based total order, so older concurrent inserts are visited first.
//!
//! When inserting a new item at visible position `pos`:
//!
//! 1. If `pos == 0` and the document is non-empty: parent is the current first
//!    visible item, side = `Left`.
//! 2. Else if the predecessor `visible[pos-1]` has any right child (visible or
//!    tombstoned) **and** `pos < visible.len()`: parent is `visible[pos]`,
//!    side = `Left`.
//! 3. Else: parent is `visible[pos-1]`, side = `Right`.
//!
//! This rule guarantees the **non-interleaving property**: contiguous bursts
//! of inserts at the same position produced by different replicas remain
//! contiguous after merge.
//!
//! ## Performance characteristics (v0.1)
//!
//! - `insert` / `delete`: `O(N + log K)` where `N` is total ops and `K` is the
//!   sibling count (binary search on a `Vec<OpId>`). The `O(N)` term is from
//!   recomputing the visible sequence on every call.
//! - `merge`: `O(M log M + (N + M) log K)` where `M` is the number of
//!   missing ops in the other replica's log.
//! - Memory: `O(N)` items + `O(N)` ops in the log.
//!
//! v0.2 will introduce a B-tree index over the visible sequence, dropping
//! `insert`/`delete` to `O(log N)`.

use crate::{
    error::Error,
    id::{OpId, ReplicaId},
    ost::OrderTree,
    version::VersionVector,
};
use smallvec::SmallVec;
use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Public op + side types (List-specific)
// ---------------------------------------------------------------------------

/// Side of a parent that an item is anchored to in the Fugue tree.
///
/// In the in-order traversal of the document tree, a node's `Left` children
/// are visited before the node itself, and `Right` children after.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Side {
    /// Anchored to the left of the parent (visited before parent).
    Left,
    /// Anchored to the right of the parent (visited after parent).
    Right,
}

/// A single [`List`] CRDT operation. Wire-format and event-log entry.
///
/// `ListOp<T>` is `Clone + Send + Sync` whenever `T` is. With the `serde`
/// feature (default), it is also `Serialize + Deserialize` whenever `T` is.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ListOp<T> {
    /// Insert `value` as a child of `parent` on the given `side`.
    ///
    /// `parent = None` means the new item is at the document root.
    Insert {
        /// Globally unique id of this insertion.
        id: OpId,
        /// Anchor parent. `None` ⇒ inserted at root level.
        parent: Option<OpId>,
        /// Which side of the parent.
        side: Side,
        /// Payload.
        value: T,
    },
    /// Mark a previously inserted item as deleted (tombstone).
    ///
    /// Deletes are idempotent: applying the same `Delete` twice is a no-op.
    Delete {
        /// Id of this delete operation (used for the event log).
        id: OpId,
        /// Id of the insertion being deleted.
        target: OpId,
    },
}

impl<T> ListOp<T> {
    /// The id of this op.
    #[must_use]
    pub fn id(&self) -> OpId {
        match self {
            ListOp::Insert { id, .. } | ListOp::Delete { id, .. } => *id,
        }
    }
}

// ---------------------------------------------------------------------------
// Internal item
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct Item<T> {
    id: OpId,
    parent: Option<OpId>,
    side: Side,
    value: T,
    /// Op ids of every delete targeting this item. Empty ⇒ visible.
    /// `SmallVec[1]` makes the common (zero-or-one delete) case allocation-free.
    deletes: SmallVec<[OpId; 1]>,
    /// Sorted ASC by `OpId`.
    left_children: Vec<OpId>,
    /// Sorted ASC by `OpId`.
    right_children: Vec<OpId>,
    /// Doc-order linked-list pointer to the in-order **tree** predecessor
    /// (regardless of visibility). `None` means this item is at the start
    /// of the document. Maintained on every insert; together with the
    /// AVL OST these give `O(log N)` lookup of any item's total rank.
    #[cfg_attr(feature = "serde", serde(default))]
    prev_doc: Option<OpId>,
    /// In-order tree successor.
    #[cfg_attr(feature = "serde", serde(default))]
    next_doc: Option<OpId>,
}

impl<T> Item<T> {
    #[inline]
    fn is_deleted(&self) -> bool {
        !self.deletes.is_empty()
    }
}

// ---------------------------------------------------------------------------
// List CRDT
// ---------------------------------------------------------------------------

/// List CRDT with **Fugue-Maximal** positioning over an **Eg-walker** event log.
///
/// See the [crate docs](crate) for usage examples. The algorithm is documented
/// in detail in the source of this module.
#[derive(Clone, Debug)]
pub struct List<T: Clone> {
    /// This replica's id.
    replica: ReplicaId,
    /// Lamport clock — advances on every local op AND every applied remote op.
    clock: u64,
    /// All items, both visible and tombstoned.
    items: HashMap<OpId, Item<T>>,
    /// Top-level (parent = None) items on the left of the virtual root.
    root_left_children: Vec<OpId>,
    /// Top-level items on the right of the virtual root.
    root_right_children: Vec<OpId>,
    /// What ops this replica has seen.
    version: VersionVector,
    /// All ops, in the order they were observed (causal-respecting since we
    /// always observe parents before children).
    log: Vec<ListOp<T>>,
    /// Order-statistic AVL tree over the visible+tombstoned items, indexed
    /// by document-order rank. Provides `O(log N)` for **all** position
    /// queries — including `position_of(opid)` via parent-pointer walk-up,
    /// which `Vec`/`imbl::Vector` can't do.
    ///
    /// We *also* maintain the parent/side tree in `items` — that's where
    /// the CRDT semantics live. This index is the fast position view.
    index: OrderTree<OpId>,
}

// ---------------------------------------------------------------------------
// Serde
// ---------------------------------------------------------------------------
//
// `HashMap<OpId, Item<T>>` cannot use struct keys with text-format serializers
// like JSON. We serialize via a `Vec<Item<T>>` snapshot — Items already embed
// their own id, so we rebuild the HashMap on deserialize.

#[cfg(feature = "serde")]
#[derive(Serialize, Deserialize)]
struct ListSnapshot<T: Clone> {
    replica: ReplicaId,
    clock: u64,
    items: Vec<Item<T>>,
    root_left_children: Vec<OpId>,
    root_right_children: Vec<OpId>,
    version: VersionVector,
    log: Vec<ListOp<T>>,
}

#[cfg(feature = "serde")]
impl<T: Clone + Serialize> Serialize for List<T> {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        // Sort items by OpId for deterministic output.
        let mut items: Vec<Item<T>> = self.items.values().cloned().collect();
        items.sort_by_key(|i| i.id);
        let snap = ListSnapshot {
            replica: self.replica,
            clock: self.clock,
            items,
            root_left_children: self.root_left_children.clone(),
            root_right_children: self.root_right_children.clone(),
            version: self.version.clone(),
            log: self.log.clone(),
        };
        snap.serialize(ser)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Clone + Deserialize<'de>> Deserialize<'de> for List<T> {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let snap = ListSnapshot::<T>::deserialize(de)?;
        let mut list = List {
            replica: snap.replica,
            clock: snap.clock,
            items: snap.items.into_iter().map(|i| (i.id, i)).collect(),
            root_left_children: snap.root_left_children,
            root_right_children: snap.root_right_children,
            version: snap.version,
            log: snap.log,
            index: OrderTree::new(),
        };
        // Rebuild the OST by walking the CRDT tree in document order.
        list.rebuild_index();
        Ok(list)
    }
}

impl<T: Clone> List<T> {
    /// Create a new empty list for the given replica.
    #[must_use]
    pub fn new(replica: ReplicaId) -> Self {
        Self {
            replica,
            clock: 0,
            items: HashMap::new(),
            root_left_children: Vec::new(),
            root_right_children: Vec::new(),
            version: VersionVector::new(),
            log: Vec::new(),
            index: OrderTree::new(),
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

    /// Visible length of the document. **O(1).**
    #[must_use]
    pub fn len(&self) -> usize {
        self.index.len()
    }

    /// Is the document empty (no visible items)?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// Iterate over visible values in document order. **O(N) total**, with
    /// amortized `O(1)` per `next()`.
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.index
            .iter_visible()
            .map(move |id| &self.items[id].value)
    }

    /// Materialize visible values as a `Vec<T>`.
    pub fn to_vec(&self) -> Vec<T> {
        self.iter().cloned().collect()
    }

    /// Visible value at `pos`, or `None` if out of bounds. **O(log N).**
    pub fn get(&self, pos: usize) -> Option<&T> {
        self.index.at_visible(pos).map(|id| &self.items[id].value)
    }

    /// Get the [`OpId`] of the visible item at `pos`, or `None` if out of
    /// bounds. **O(log N).**
    ///
    /// This is what callers building higher-level CRDTs (e.g. rich
    /// text with format spans anchored to character ids) need.
    #[must_use]
    pub fn id_at(&self, pos: usize) -> Option<OpId> {
        self.index.at_visible(pos).copied()
    }

    /// All visible item ids in document order. **O(N).**
    #[must_use]
    pub fn op_ids(&self) -> Vec<OpId> {
        self.index.iter_visible().copied().collect()
    }

    /// Visible position of the item with this [`OpId`], or `None` if the
    /// item is unknown or tombstoned. **O(log N).**
    #[must_use]
    pub fn position_of(&self, id: OpId) -> Option<usize> {
        self.index.visible_position_of(&id)
    }

    /// Phantom visible position: where this `OpId` *would* sit in the
    /// visible sequence even if it's tombstoned. For visible items this
    /// is the same as [`Self::position_of`]; for tombstoned items it is
    /// the position the next visible item occupies. `None` if `id` is
    /// unknown to this replica. **O(log N).**
    #[must_use]
    pub fn phantom_position_of(&self, id: OpId) -> Option<usize> {
        self.index.phantom_visible_position_of(&id)
    }

    /// Garbage-collect tombstoned items that no replica still needs.
    ///
    /// An item is eligible for GC if **all** of:
    /// 1. It is tombstoned (`deletes` is non-empty).
    /// 2. It has no left or right children — removing it can't orphan
    ///    other items.
    /// 3. Its insertion op is in `frontier` (i.e., observed by every
    ///    replica we care about).
    /// 4. All of its delete ops are in `frontier`.
    ///
    /// Returns the number of items removed. Run repeatedly to cascade —
    /// once a tombstoned leaf is removed, its parent may become a
    /// tombstoned leaf and become eligible on the next call.
    ///
    /// The caller must construct `frontier` to be the **intersection** of
    /// every collaborating replica's version vector. If `frontier` is too
    /// permissive, future ops referencing GC'd items will fail with
    /// [`Error::MissingParent`].
    pub fn gc(&mut self, frontier: &VersionVector) -> usize {
        let eligible: Vec<OpId> = self
            .items
            .iter()
            .filter(|(_, item)| {
                item.is_deleted()
                    && item.left_children.is_empty()
                    && item.right_children.is_empty()
                    && frontier.contains(item.id)
                    && item.deletes.iter().all(|d| frontier.contains(*d))
            })
            .map(|(id, _)| *id)
            .collect();
        let removed = eligible.len();
        for id in eligible {
            self.remove_item(id);
        }
        removed
    }

    /// Remove a single tombstoned, child-less item. Used by [`Self::gc`].
    fn remove_item(&mut self, id: OpId) {
        let (parent, side, prev, next) = {
            let item = self.items.get(&id).expect("remove_item: missing");
            (item.parent, item.side, item.prev_doc, item.next_doc)
        };

        // Detach from parent's children list (or root_*).
        let target_list = match (parent, side) {
            (None, Side::Left) => &mut self.root_left_children,
            (None, Side::Right) => &mut self.root_right_children,
            (Some(p), Side::Left) => {
                &mut self
                    .items
                    .get_mut(&p)
                    .expect("remove_item: parent missing")
                    .left_children
            }
            (Some(p), Side::Right) => {
                &mut self
                    .items
                    .get_mut(&p)
                    .expect("remove_item: parent missing")
                    .right_children
            }
        };
        target_list.retain(|&x| x != id);

        // Remove from OST.
        self.index.remove(&id);

        // Patch the doc-order linked list.
        if let Some(p) = prev {
            self.items
                .get_mut(&p)
                .expect("remove_item: prev missing")
                .next_doc = next;
        }
        if let Some(n) = next {
            self.items
                .get_mut(&n)
                .expect("remove_item: next missing")
                .prev_doc = prev;
        }

        // Drop the item itself.
        self.items.remove(&id);
    }

    /// Drop log entries whose ops are entirely covered by `frontier`. Useful
    /// alongside [`Self::gc`] to reclaim event-log memory after long sessions.
    ///
    /// Returns the number of ops removed.
    pub fn compact_log(&mut self, frontier: &VersionVector) -> usize {
        let before = self.log.len();
        self.log.retain(|op| !frontier.contains(op.id()));
        before - self.log.len()
    }

    /// Apply the inverse of `op` as a *new* local op, returning that
    /// new op so the caller can push it onto a redo stack.
    ///
    /// - Inverse of `Insert` is a `Delete` targeting that insertion.
    /// - Inverse of `Delete` is a fresh `Insert` of the original value
    ///   at the original anchor (a NEW [`OpId`]) — the original
    ///   tombstoned item stays tombstoned; we conjure a duplicate.
    ///   This matches how Yjs and Automerge implement undo of deletion.
    ///
    /// Returns `None` if the op's target item is unknown (e.g. GC'd).
    pub fn apply_inverse(&mut self, op: &ListOp<T>) -> Option<ListOp<T>> {
        match op {
            ListOp::Insert { id, .. } => {
                if !self.items.contains_key(id) {
                    return None;
                }
                self.clock = self
                    .clock
                    .checked_add(1)
                    .expect("Lamport clock overflow (>2^64 ops)");
                let inverse_id = OpId::new(self.clock, self.replica);
                let inv = ListOp::Delete {
                    id: inverse_id,
                    target: *id,
                };
                // Apply locally.
                let target_item = self.items.get_mut(id).expect("checked above");
                let was_visible = target_item.deletes.is_empty();
                target_item.deletes.push(inverse_id);
                if was_visible {
                    self.index.set_visible(id, false);
                }
                self.version.observe(inverse_id);
                self.log.push(inv.clone());
                Some(inv)
            }
            ListOp::Delete { target, .. } => {
                // To restore the deleted item's *visible* position we conjure
                // a new item anchored to the tombstone itself: parent =
                // target, side = Left. In in-order traversal that puts the
                // new item BEFORE target (and BEFORE any tombstoned right
                // subtree of target), exactly where target used to be.
                let value = self.items.get(target)?.value.clone();
                self.clock = self
                    .clock
                    .checked_add(1)
                    .expect("Lamport clock overflow (>2^64 ops)");
                let new_id = OpId::new(self.clock, self.replica);
                let new_op = ListOp::Insert {
                    id: new_id,
                    parent: Some(*target),
                    side: Side::Left,
                    value: value.clone(),
                };
                let item = Item {
                    id: new_id,
                    parent: Some(*target),
                    side: Side::Left,
                    value,
                    deletes: SmallVec::new(),
                    left_children: Vec::new(),
                    right_children: Vec::new(),
                    prev_doc: None,
                    next_doc: None,
                };
                self.apply_insert_internal(item);
                let total_pos = self.compute_total_position_for(new_id);
                self.index.insert_at_total(total_pos, new_id, true);
                self.version.observe(new_id);
                self.log.push(new_op.clone());
                Some(new_op)
            }
        }
    }

    /// Walk the underlying tree (visible + tombstoned items) and produce a
    /// map `OpId → "phantom position"` — the visible-index that item would
    /// have if visible. Tombstoned items share the phantom position of the
    /// next visible item in document order. Visible items get their actual
    /// visible index. Used by higher-level CRDTs that need stable anchor
    /// positions even for deleted characters.
    #[must_use]
    pub fn phantom_positions(&self) -> HashMap<OpId, usize> {
        let mut map = HashMap::with_capacity(self.items.len());
        let mut visible_count = 0usize;
        let mut stack: Vec<TraverseFrame> = Vec::with_capacity(64);
        for &child in self.root_right_children.iter().rev() {
            stack.push(TraverseFrame::EnterNode(child));
        }
        for &child in self.root_left_children.iter().rev() {
            stack.push(TraverseFrame::EnterNode(child));
        }
        while let Some(frame) = stack.pop() {
            match frame {
                TraverseFrame::EnterNode(id) => {
                    let item = &self.items[&id];
                    stack.push(TraverseFrame::EmitAndRight(id));
                    for &child in item.left_children.iter().rev() {
                        stack.push(TraverseFrame::EnterNode(child));
                    }
                }
                TraverseFrame::EmitAndRight(id) => {
                    let item = &self.items[&id];
                    map.insert(id, visible_count);
                    if !item.is_deleted() {
                        visible_count += 1;
                    }
                    for &child in item.right_children.iter().rev() {
                        stack.push(TraverseFrame::EnterNode(child));
                    }
                }
            }
        }
        map
    }

    /// Advance this replica's Lamport clock by 1 and return a fresh [`OpId`].
    ///
    /// This exists for higher-level CRDTs (e.g. [`crate::Text`]) that
    /// embed a `List` and need to share its Lamport clock. Callers should
    /// pair this with [`Self::observe_external`] when applying remote ops
    /// from the parent CRDT that didn't go through `apply`.
    pub fn next_op_id(&mut self) -> OpId {
        self.clock = self
            .clock
            .checked_add(1)
            .expect("Lamport clock overflow (>2^64 ops)");
        OpId::new(self.clock, self.replica)
    }

    /// Advance this replica's Lamport clock to be at least `id.counter`
    /// without applying an op. Used by higher-level CRDTs to keep clocks
    /// in sync with their shared event stream.
    pub fn observe_external(&mut self, id: OpId) {
        self.clock = self.clock.max(id.counter);
    }

    /// Was this item ever observed (visible or tombstoned)?
    #[must_use]
    pub fn contains_id(&self, id: OpId) -> bool {
        self.items.contains_key(&id)
    }

    /// Is the item with the given [`OpId`] currently visible (not tombstoned)?
    /// `None` if the item is unknown.
    #[must_use]
    pub fn is_visible(&self, id: OpId) -> Option<bool> {
        self.items.get(&id).map(|i| !i.is_deleted())
    }

    /// Insert `value` at visible position `pos`.
    ///
    /// Returns the generated `ListOp<T>` so the caller can broadcast it to peers.
    ///
    /// # Panics
    ///
    /// Panics if `pos > self.len()`. Use [`Self::try_insert`] for a checked
    /// variant.
    pub fn insert(&mut self, pos: usize, value: T) -> ListOp<T> {
        self.try_insert(pos, value)
            .unwrap_or_else(|e| panic!("List::insert: {e}"))
    }

    /// Checked variant of [`Self::insert`].
    pub fn try_insert(&mut self, pos: usize, value: T) -> Result<ListOp<T>, Error> {
        let len = self.index.len();
        if pos > len {
            return Err(Error::OutOfBounds { pos, len });
        }
        let (parent, side) = self.determine_anchor_at(pos);
        self.clock = self
            .clock
            .checked_add(1)
            .expect("Lamport clock overflow (>2^64 ops)");
        let id = OpId::new(self.clock, self.replica);
        let op = ListOp::Insert {
            id,
            parent,
            side,
            value: value.clone(),
        };
        let item = Item {
            id,
            parent,
            side,
            value,
            deletes: SmallVec::new(),
            left_children: Vec::new(),
            right_children: Vec::new(),
            prev_doc: None,
            next_doc: None,
        };
        self.apply_insert_internal(item);
        // Local AND remote inserts compute the total rank from the CRDT tree:
        // `pos` only describes the visible position the user wants, but the
        // new item may be in-order *before* tombstones that occupy total
        // ranks past `pos`. Using insert_at_visible(pos) would put the item
        // at total_len when pos == len, which is wrong if those trailing
        // tombstones belong to a right subtree we don't actually trail.
        let _ = pos;
        let total_pos = self.compute_total_position_for(id);
        self.index.insert_at_total(total_pos, id, true);
        self.version.observe(id);
        self.log.push(op.clone());
        Ok(op)
    }

    /// Delete the item at visible position `pos`.
    ///
    /// # Panics
    ///
    /// Panics if `pos >= self.len()`. Use [`Self::try_delete`] for a checked
    /// variant.
    pub fn delete(&mut self, pos: usize) -> ListOp<T> {
        self.try_delete(pos)
            .unwrap_or_else(|e| panic!("List::delete: {e}"))
    }

    /// Checked variant of [`Self::delete`].
    pub fn try_delete(&mut self, pos: usize) -> Result<ListOp<T>, Error> {
        let len = self.index.len();
        if pos >= len {
            return Err(Error::OutOfBounds { pos, len });
        }
        let target = *self.index.at_visible(pos).expect("pos < len");
        self.clock = self
            .clock
            .checked_add(1)
            .expect("Lamport clock overflow (>2^64 ops)");
        let id = OpId::new(self.clock, self.replica);
        let item = self
            .items
            .get_mut(&target)
            .expect("visible id missing from items");
        item.deletes.push(id);
        // Flip visibility in the OST — O(log N), no restructuring.
        self.index.set_visible(&target, false);
        let op = ListOp::Delete { id, target };
        self.version.observe(id);
        self.log.push(op.clone());
        Ok(op)
    }

    /// Apply a single remote operation.
    ///
    /// Idempotent: if the op has already been observed, returns `Ok(())`
    /// without modification.
    ///
    /// # Errors
    ///
    /// Returns [`Error::MissingParent`] if an `Insert` op references a parent
    /// not yet observed. Callers must respect causal delivery order.
    pub fn apply(&mut self, op: ListOp<T>) -> Result<(), Error> {
        let op_id = op.id();
        // Own ops (already in our version vector) are an idempotent no-op
        // even when we're explicitly re-applying our own log.
        if self.version.contains(op_id) {
            return Ok(());
        }
        // After the version check: a remote op claiming our replica id is a
        // genuine collision, not an echo of our own work.
        match &op {
            ListOp::Insert {
                id,
                parent,
                side,
                value,
            } => {
                if let Some(p) = parent {
                    if !self.items.contains_key(p) {
                        return Err(Error::MissingParent {
                            op: *id,
                            missing: *p,
                        });
                    }
                }
                let item = Item {
                    id: *id,
                    parent: *parent,
                    side: *side,
                    value: value.clone(),
                    deletes: SmallVec::new(),
                    left_children: Vec::new(),
                    right_children: Vec::new(),
                    prev_doc: None,
                    next_doc: None,
                };
                self.apply_insert_internal(item);
                // Compute the total-rank position for this remote insert
                // (independent of visibility) and splice it into the OST.
                let total_pos = self.compute_total_position_for(*id);
                self.index.insert_at_total(total_pos, *id, true);
            }
            ListOp::Delete { id: _, target } => {
                let Some(item) = self.items.get_mut(target) else {
                    return Err(Error::UnknownTarget {
                        op: op_id,
                        target: *target,
                    });
                };
                let was_visible = item.deletes.is_empty();
                if !item.deletes.contains(&op_id) {
                    item.deletes.push(op_id);
                }
                // The first tombstone flips the OST visibility. Concurrent
                // additional deletes on the same target are no-ops here.
                if was_visible {
                    self.index.set_visible(target, false);
                }
            }
        }
        self.version.observe(op_id);
        self.clock = self.clock.max(op_id.counter);
        self.log.push(op);
        Ok(())
    }

    /// Merge all of `other`'s state into `self`.
    ///
    /// Equivalent to applying every op in `other.log` that `self.version`
    /// hasn't seen, in causal order.
    pub fn merge(&mut self, other: &Self) {
        // OpIds are Lamport — sorting by id ASC ⇒ topological order, so parents
        // are always applied before children.
        let mut to_apply: Vec<&ListOp<T>> = other
            .log
            .iter()
            .filter(|op| !self.version.contains(op.id()))
            .collect();
        to_apply.sort_by_key(|op| op.id());
        for op in to_apply {
            // We trust other's log to be self-consistent. If apply fails here
            // it indicates corrupt input.
            self.apply(op.clone())
                .expect("corrupt op log in merge source");
        }
    }

    /// All ops in this list's event log, in observed (causal-respecting) order.
    #[must_use]
    pub fn ops(&self) -> &[ListOp<T>] {
        &self.log
    }

    /// Iterate over ops not yet seen by `since`. Useful for incremental sync:
    /// the peer sends its `version()`, you respond with `ops_since(their_version)`.
    pub fn ops_since<'a>(
        &'a self,
        since: &'a VersionVector,
    ) -> impl Iterator<Item = &'a ListOp<T>> + 'a {
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

    /// Compute `(parent, side)` for an insert at visible position `pos`.
    /// Reads the index in O(log N).
    ///
    /// Implements the Fugue-Maximal anchor rule:
    /// - empty doc → `(None, Right)` (top-level)
    /// - `pos == 0` → left child of visible[0]
    /// - visible[pos-1] has any right child & `pos < len` → left child of visible[pos]
    /// - otherwise → right child of visible[pos-1]
    fn determine_anchor_at(&self, pos: usize) -> (Option<OpId>, Side) {
        let len = self.index.len();
        if len == 0 {
            return (None, Side::Right);
        }
        if pos == 0 {
            return (Some(*self.index.at_visible(0).unwrap()), Side::Left);
        }
        let pred = *self.index.at_visible(pos - 1).unwrap();
        let pred_has_right = !self.items[&pred].right_children.is_empty();
        if pred_has_right && pos < len {
            (Some(*self.index.at_visible(pos).unwrap()), Side::Left)
        } else {
            (Some(pred), Side::Right)
        }
    }

    /// Total-rank position the just-inserted `id` occupies in the OST.
    /// O(log N) via the cached `prev_doc` pointer + OST `total_position_of`.
    fn compute_total_position_for(&self, id: OpId) -> usize {
        match self.items[&id].prev_doc {
            Some(pred) => {
                self.index
                    .total_position_of(&pred)
                    .expect("prev_doc target must be in OST")
                    + 1
            }
            None => 0,
        }
    }

    /// Old recursive helper, retained to satisfy the old test stubs that
    /// referenced it. Returns the rightmost visible item or `None`.
    #[allow(dead_code)]
    fn rightmost_visible_in_subtree(&self, root: OpId) -> Option<OpId> {
        enum Action {
            EnterSubtree(OpId),
            EmitSelf(OpId),
        }
        let mut stack = vec![Action::EnterSubtree(root)];
        while let Some(action) = stack.pop() {
            match action {
                Action::EnterSubtree(id) => {
                    let item = &self.items[&id];
                    for &child in &item.left_children {
                        stack.push(Action::EnterSubtree(child));
                    }
                    stack.push(Action::EmitSelf(id));
                    for &child in &item.right_children {
                        stack.push(Action::EnterSubtree(child));
                    }
                }
                Action::EmitSelf(id) => {
                    if !self.items[&id].is_deleted() {
                        return Some(id);
                    }
                }
            }
        }
        None
    }

    /// Insert an `Item` into the tree, maintaining the sorted-children
    /// invariant AND the doc-order linked list (`prev_doc`/`next_doc`).
    ///
    /// Pre-condition: the item's `parent` (if any) already exists in `self.items`.
    fn apply_insert_internal(&mut self, item: Item<T>) {
        let id = item.id;
        let parent = item.parent;
        let side = item.side;
        // Insert the item first; subsequent &mut borrow for children list update
        // can't conflict with self.items.insert.
        self.items.insert(id, item);
        let target_list = match (parent, side) {
            (None, Side::Left) => &mut self.root_left_children,
            (None, Side::Right) => &mut self.root_right_children,
            (Some(parent_id), Side::Left) => {
                &mut self
                    .items
                    .get_mut(&parent_id)
                    .expect("apply_insert_internal: parent missing")
                    .left_children
            }
            (Some(parent_id), Side::Right) => {
                &mut self
                    .items
                    .get_mut(&parent_id)
                    .expect("apply_insert_internal: parent missing")
                    .right_children
            }
        };
        // Maintain ASC-by-OpId order via binary search.
        let pos = target_list.binary_search(&id).unwrap_or_else(|e| e);
        target_list.insert(pos, id);
        // Link the new item into the doc-order chain.
        self.link_doc_pointers(id);
    }

    /// Compute and install `prev_doc` / `next_doc` pointers for a freshly
    /// inserted item, AND update the affected neighbors' pointers. Uses the
    /// existing pointers of siblings/parent as O(1) shortcuts where possible;
    /// only the leftmost/rightmost descendant walks (`subtree_first`/
    /// `subtree_last`) cost more, and only for inserts that land between
    /// existing siblings with deep subtrees.
    fn link_doc_pointers(&mut self, c_id: OpId) {
        let (prev, next) = self.find_doc_neighbors(c_id);
        let c_item = self
            .items
            .get_mut(&c_id)
            .expect("link_doc_pointers: c missing");
        c_item.prev_doc = prev;
        c_item.next_doc = next;
        if let Some(p) = prev {
            self.items
                .get_mut(&p)
                .expect("prev_doc target missing")
                .next_doc = Some(c_id);
        }
        if let Some(n) = next {
            self.items
                .get_mut(&n)
                .expect("next_doc target missing")
                .prev_doc = Some(c_id);
        }
    }

    /// Read-only sibling slice for `(parent, side)`.
    fn children_slice(&self, parent: Option<OpId>, side: Side) -> &[OpId] {
        match (parent, side) {
            (None, Side::Left) => &self.root_left_children,
            (None, Side::Right) => &self.root_right_children,
            (Some(p), Side::Left) => &self.items[&p].left_children,
            (Some(p), Side::Right) => &self.items[&p].right_children,
        }
    }

    /// In-order **first** node in the subtree rooted at `id` — the leftmost
    /// descendant. Iterative; bounded by the depth of the left chain.
    fn subtree_first(&self, mut id: OpId) -> OpId {
        loop {
            match self.items[&id].left_children.first() {
                Some(&first) => id = first,
                None => return id,
            }
        }
    }

    /// In-order **last** node in the subtree rooted at `id` — the rightmost
    /// descendant. Iterative; bounded by the depth of the right chain.
    fn subtree_last(&self, mut id: OpId) -> OpId {
        loop {
            match self.items[&id].right_children.last() {
                Some(&last) => id = last,
                None => return id,
            }
        }
    }

    /// Compute `(prev_doc, next_doc)` for an item that has just been spliced
    /// into the children list at its CRDT-determined rank.
    ///
    /// Important property: when a next sibling exists, that sibling's
    /// `prev_doc` is currently the value we want for `c.prev_doc` — we
    /// haven't updated it yet, so it still points at the in-order
    /// predecessor of the position c now occupies.
    fn find_doc_neighbors(&self, c_id: OpId) -> (Option<OpId>, Option<OpId>) {
        let item = &self.items[&c_id];
        let parent = item.parent;
        let side = item.side;
        let siblings = self.children_slice(parent, side);
        let k = siblings
            .iter()
            .position(|&x| x == c_id)
            .expect("c just inserted into siblings");
        let n = siblings.len();

        // ---- predecessor ----
        let prev = if k + 1 < n {
            // The in-order successor of c is subtree_first(next_sibling), and
            // c's predecessor is whatever its successor's prev_doc currently
            // is (the chain hasn't been rewritten yet).
            let succ_first = self.subtree_first(siblings[k + 1]);
            self.items[&succ_first].prev_doc
        } else if k > 0 {
            // No next sibling. Predecessor = subtree_last of prev sibling.
            Some(self.subtree_last(siblings[k - 1]))
        } else {
            // c is the only sibling on this side.
            match side {
                Side::Right => match parent {
                    Some(p) => Some(p),
                    None => self
                        .root_left_children
                        .last()
                        .map(|&id| self.subtree_last(id)),
                },
                Side::Left => match parent {
                    Some(p) => self.items[&p].prev_doc,
                    None => None,
                },
            }
        };

        // ---- successor ----
        let next = if k + 1 < n {
            // Successor = subtree_first of next sibling.
            Some(self.subtree_first(siblings[k + 1]))
        } else if k > 0 {
            // No next sibling, but prev sibling exists. Use the OLD
            // subtree_last(prev_sib).next_doc which we haven't rewritten yet.
            let old_last = self.subtree_last(siblings[k - 1]);
            self.items[&old_last].next_doc
        } else {
            // c is the only sibling on this side.
            match side {
                Side::Left => match parent {
                    Some(p) => Some(p),
                    None => self
                        .root_right_children
                        .first()
                        .map(|&id| self.subtree_first(id)),
                },
                Side::Right => match parent {
                    Some(p) => self.items[&p].next_doc,
                    None => None,
                },
            }
        };

        (prev, next)
    }

    /// Rebuild the OST index by walking the CRDT tree in document order.
    /// Used after deserialization. O(N).
    fn rebuild_index(&mut self) {
        self.index = OrderTree::new();
        let mut stack: Vec<TraverseFrame> = Vec::with_capacity(64);
        for &child in self.root_right_children.iter().rev() {
            stack.push(TraverseFrame::EnterNode(child));
        }
        for &child in self.root_left_children.iter().rev() {
            stack.push(TraverseFrame::EnterNode(child));
        }
        let mut total_rank = 0usize;
        while let Some(frame) = stack.pop() {
            match frame {
                TraverseFrame::EnterNode(id) => {
                    let item = &self.items[&id];
                    stack.push(TraverseFrame::EmitAndRight(id));
                    for &child in item.left_children.iter().rev() {
                        stack.push(TraverseFrame::EnterNode(child));
                    }
                }
                TraverseFrame::EmitAndRight(id) => {
                    let visible = !self.items[&id].is_deleted();
                    self.index.insert_at_total(total_rank, id, visible);
                    total_rank += 1;
                    let item = &self.items[&id];
                    for &child in item.right_children.iter().rev() {
                        stack.push(TraverseFrame::EnterNode(child));
                    }
                }
            }
        }
    }

    /// Iteratively walk the tree to compute the visible sequence.
    ///
    /// Used by `phantom_positions()` for compatibility. Mutation paths
    /// maintain the OST index incrementally and don't call this.
    ///
    /// Iterative (rather than recursive) traversal avoids stack overflow on
    /// long right-chains (e.g. typing a 100K-character document at the end).
    #[allow(dead_code)]
    fn compute_visible_ids(&self) -> Vec<OpId> {
        // Conservative initial capacity: most items in a "young" doc are visible.
        let mut out = Vec::with_capacity(self.items.len());
        let mut stack: Vec<TraverseFrame> = Vec::with_capacity(64);

        // Seed: root's right children, then root's left children (in reverse so
        // popping yields left-children first).
        for &child in self.root_right_children.iter().rev() {
            stack.push(TraverseFrame::EnterNode(child));
        }
        for &child in self.root_left_children.iter().rev() {
            stack.push(TraverseFrame::EnterNode(child));
        }

        while let Some(frame) = stack.pop() {
            match frame {
                TraverseFrame::EnterNode(id) => {
                    let item = &self.items[&id];
                    // After processing left subtrees, we want to come back and emit
                    // self + process right subtrees. Push EmitAndRight first so it
                    // runs *after* all left children.
                    stack.push(TraverseFrame::EmitAndRight(id));
                    for &child in item.left_children.iter().rev() {
                        stack.push(TraverseFrame::EnterNode(child));
                    }
                }
                TraverseFrame::EmitAndRight(id) => {
                    let item = &self.items[&id];
                    if !item.is_deleted() {
                        out.push(id);
                    }
                    for &child in item.right_children.iter().rev() {
                        stack.push(TraverseFrame::EnterNode(child));
                    }
                }
            }
        }

        out
    }

    /// Internal invariant check, used by tests. Verifies that
    /// every children-list is sorted, every parent reference points to an
    /// existing item, and the version vector matches the observed log.
    #[cfg(test)]
    pub(crate) fn check_invariants(&self) {
        for item in self.items.values() {
            if let Some(parent) = item.parent {
                assert!(
                    self.items.contains_key(&parent),
                    "item {:?} references missing parent {:?}",
                    item.id,
                    parent
                );
            }
            for window in item.left_children.windows(2) {
                assert!(window[0] < window[1], "left_children not sorted");
            }
            for window in item.right_children.windows(2) {
                assert!(window[0] < window[1], "right_children not sorted");
            }
        }
        for window in self.root_left_children.windows(2) {
            assert!(window[0] < window[1], "root_left_children not sorted");
        }
        for window in self.root_right_children.windows(2) {
            assert!(window[0] < window[1], "root_right_children not sorted");
        }
        for op in &self.log {
            assert!(
                self.version.contains(op.id()),
                "version vector missing logged op {:?}",
                op.id()
            );
        }
    }
}

impl<T: Clone> Default for List<T> {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Stack frame used by the iterative tree traversal in [`List::visible_ids`].
enum TraverseFrame {
    EnterNode(OpId),
    EmitAndRight(OpId),
}

// ---------------------------------------------------------------------------
// String convenience: `List<char>` formats as the underlying text.
// ---------------------------------------------------------------------------

impl std::fmt::Display for List<char> {
    /// Write the visible characters with no separator. `format!("{list}")`
    /// gives the document text.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for c in self.iter() {
            f.write_str(c.encode_utf8(&mut [0u8; 4]))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let list = List::<char>::new(1);
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert_eq!(list.to_vec(), Vec::<char>::new());
    }

    #[test]
    fn single_insert() {
        let mut list = List::<char>::new(1);
        list.insert(0, 'a');
        assert_eq!(list.to_vec(), vec!['a']);
        list.check_invariants();
    }

    #[test]
    fn append() {
        let mut list = List::<char>::new(1);
        for (i, c) in "Hello".chars().enumerate() {
            list.insert(i, c);
        }
        assert_eq!(list.to_string(), "Hello");
        list.check_invariants();
    }

    #[test]
    fn insert_at_beginning() {
        let mut list = List::<char>::new(1);
        list.insert(0, 'b');
        list.insert(0, 'a');
        assert_eq!(list.to_string(), "ab");
        list.check_invariants();
    }

    #[test]
    fn insert_in_middle() {
        let mut list = List::<char>::new(1);
        list.insert(0, 'a');
        list.insert(1, 'c');
        list.insert(1, 'b');
        assert_eq!(list.to_string(), "abc");
        list.check_invariants();
    }

    #[test]
    fn delete_works() {
        let mut list = List::<char>::new(1);
        for (i, c) in "Hello".chars().enumerate() {
            list.insert(i, c);
        }
        list.delete(0);
        assert_eq!(list.to_string(), "ello");
        list.delete(3);
        assert_eq!(list.to_string(), "ell");
        list.check_invariants();
    }

    #[test]
    fn insert_after_delete_at_end() {
        let mut list = List::<char>::new(1);
        list.insert(0, 'a');
        list.insert(1, 'b');
        list.insert(2, 'c');
        list.delete(2);
        assert_eq!(list.to_string(), "ab");
        // Insert at end (position 2). Predecessor 'b' has tombstoned right child 'c'.
        list.insert(2, 'X');
        assert_eq!(list.to_string(), "abX");
        list.check_invariants();
    }
}
