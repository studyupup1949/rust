//! Order-statistic AVL tree.
//!
//! A balanced BST where the "key" is the **insertion-order rank** rather
//! than a value comparison. Each node is augmented with two subtree counts
//! — `total_count` (all nodes) and `visible_count` (nodes whose `visible`
//! flag is true) — yielding `O(log N)` answers to:
//!
//! - `at_visible(rank)` / `at_total(rank)` — find the i-th node.
//! - `visible_position_of(key)` / `total_position_of(key)` — find a node's
//!   rank without walking the whole tree.
//! - `insert_at_total(rank, key)` — insert before the i-th node.
//! - `set_visible(key, bool)` — flip a node's visibility.
//!
//! This is the index sitting under [`crate::List`]: the CRDT semantics
//! (parent / side / children) live in `List::items`, while this tree gives
//! the visible-position view in logarithmic time. Tombstones flip the
//! `visible` flag without restructuring the tree, so deletes are also
//! `O(log N)`.
//!
//! ## Implementation notes
//!
//! - **Safe Rust only**: nodes live in a `Vec<Option<Node>>` slab, indexed
//!   by stable [`NodeId`]s. No `unsafe`, no raw pointers.
//! - **Free list**: removed nodes (rare — used only on full GC paths) are
//!   recycled.
//! - **AVL invariant**: at every node, `|height(left) - height(right)| ≤ 1`.
//!   Maintained via single (left/right) and double (LR/RL) rotations after
//!   each insertion.
//! - **Augmentation**: every rotation and child-pointer swap calls
//!   `update_counts` to keep `visible_count` and `total_count` consistent.

#![allow(dead_code)] // Several methods are exposed for future tasks (cursors,
                     // GC, fuzz harness) and aren't called from List yet.

use std::collections::HashMap;
use std::hash::Hash;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Stable handle to a node in the [`OrderTree`] slab.
///
/// 32-bit because most realistic documents fit in 4 billion items;
/// shrinking the per-node payload halves the working set vs. a 64-bit handle.
pub(crate) type NodeId = u32;

const NIL: NodeId = u32::MAX;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct Node<T> {
    key: T,
    visible: bool,
    parent: NodeId,
    left: NodeId,
    right: NodeId,
    height: i8,
    visible_count: u32,
    total_count: u32,
}

/// Order-statistic AVL tree.
///
/// See the [module docs](self) for semantics and complexity.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) struct OrderTree<T: Eq + Hash + Clone> {
    /// Slab of nodes. `None` slots are recycled via `free`.
    nodes: Vec<Option<Node<T>>>,
    /// Indices of free slots.
    free: Vec<NodeId>,
    /// Current root, or `NIL` when the tree is empty.
    root: NodeId,
    /// `key → NodeId` reverse index for O(1) lookup of a node by its key.
    by_key: HashMap<T, NodeId>,
}

impl<T: Eq + Hash + Clone> Default for OrderTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Eq + Hash + Clone> OrderTree<T> {
    /// Create an empty tree.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            nodes: Vec::new(),
            free: Vec::new(),
            root: NIL,
            by_key: HashMap::new(),
        }
    }

    /// Number of **visible** items in the tree.
    #[must_use]
    pub(crate) fn len(&self) -> usize {
        if self.root == NIL {
            0
        } else {
            self.node(self.root).visible_count as usize
        }
    }

    /// Number of items in the tree, visible *or* hidden.
    #[must_use]
    pub(crate) fn total_len(&self) -> usize {
        if self.root == NIL {
            0
        } else {
            self.node(self.root).total_count as usize
        }
    }

    /// Are there any visible items?
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Does the tree contain `key` (visible or hidden)?
    pub(crate) fn contains(&self, key: &T) -> bool {
        self.by_key.contains_key(key)
    }

    /// Get the visible item at `rank`, or `None` if out of bounds.
    pub(crate) fn at_visible(&self, rank: usize) -> Option<&T> {
        let id = self.find_at(rank, true)?;
        Some(&self.node(id).key)
    }

    /// Get the item at total `rank` (counting tombstoned), or `None`.
    pub(crate) fn at_total(&self, rank: usize) -> Option<&T> {
        let id = self.find_at(rank, false)?;
        Some(&self.node(id).key)
    }

    /// Visible-rank of `key`, or `None` if absent or hidden.
    pub(crate) fn visible_position_of(&self, key: &T) -> Option<usize> {
        let id = *self.by_key.get(key)?;
        if !self.node(id).visible {
            return None;
        }
        Some(self.position_walk_up(id, true))
    }

    /// Total-rank of `key` (tombstoned items get the rank of the next visible),
    /// or `None` if absent.
    pub(crate) fn total_position_of(&self, key: &T) -> Option<usize> {
        let id = *self.by_key.get(key)?;
        Some(self.position_walk_up(id, false))
    }

    /// Phantom visible-rank: position in the visible sequence the `key`
    /// would occupy. For a visible node this is its `visible_position_of`;
    /// for a tombstoned node it's the visible rank of the next visible
    /// item in document order (i.e. how many visible items precede it).
    ///
    /// Used by [`crate::Text`] to resolve anchors that point at tombstoned
    /// characters.
    pub(crate) fn phantom_visible_position_of(&self, key: &T) -> Option<usize> {
        let id = *self.by_key.get(key)?;
        Some(self.position_walk_up(id, true))
    }

    /// Insert `key` so it appears at **visible rank** `rank` in the
    /// visible-only sequence. The new node is visible by default.
    ///
    /// Equivalent to translating `rank` to its corresponding total rank
    /// (the total rank of the existing visible item at that rank, or the
    /// end of the tree if `rank == self.len()`) and calling
    /// [`Self::insert_at_total`].
    ///
    /// This is the API [`crate::List`] uses for both local and remote
    /// inserts — you pass the visible position you want, and the OST
    /// figures out where it lands among any intervening tombstones.
    ///
    /// # Panics
    ///
    /// Panics if `rank > self.len()`, or if `key` is already in the tree.
    pub(crate) fn insert_at_visible(&mut self, rank: usize, key: T) {
        let total_rank = if rank == self.len() {
            self.total_len()
        } else {
            let id = self
                .find_at(rank, true)
                .expect("visible rank in bounds (post len check)");
            self.position_walk_up(id, false)
        };
        self.insert_at_total(total_rank, key, true);
    }

    /// Insert `key` at total-rank `rank`. `visible` controls whether the
    /// new node counts toward `len()`.
    ///
    /// # Panics
    ///
    /// Panics if `rank > self.total_len()`, or if `key` is already in the tree.
    pub(crate) fn insert_at_total(&mut self, rank: usize, key: T, visible: bool) {
        // `total_len()` is still pre-attach here — alloc creates a detached
        // node that doesn't yet count toward the root's `total_count`.
        let old_total = self.total_len();
        assert!(
            rank <= old_total,
            "rank {rank} out of bounds (len {old_total})"
        );
        assert!(!self.by_key.contains_key(&key), "duplicate key");
        let new_id = self.alloc(key, visible);
        if self.root == NIL {
            self.root = new_id;
            return;
        }
        // Find the insertion site: we want `new_id` to occupy total-rank
        // `rank` after insertion. Either:
        //   - rank == old_total → append (rightmost descendant gets a right child).
        //   - rank < old_total → place as left child of node currently at
        //     `rank`, or as right child of that node's in-order predecessor
        //     if its left slot is occupied.
        if rank == old_total {
            let rightmost = self.rightmost_of(self.root);
            self.attach_right(rightmost, new_id);
        } else {
            let succ = self.find_at(rank, false).expect("rank in bounds");
            if self.node(succ).left == NIL {
                self.attach_left(succ, new_id);
            } else {
                let target = self.rightmost_of(self.node(succ).left);
                self.attach_right(target, new_id);
            }
        }
        let parent = self.node(new_id).parent;
        self.rebalance_up(parent);
    }

    /// Flip a node's visibility. Returns `true` if the visibility actually
    /// changed.
    pub(crate) fn set_visible(&mut self, key: &T, visible: bool) -> bool {
        let Some(&id) = self.by_key.get(key) else {
            return false;
        };
        if self.node(id).visible == visible {
            return false;
        }
        self.node_mut(id).visible = visible;
        self.update_counts_up(id);
        true
    }

    /// Iterate visible keys in in-order.
    pub(crate) fn iter_visible(&self) -> impl Iterator<Item = &T> + '_ {
        InOrderIter::new(self).filter_map(move |id| {
            let n = self.node(id);
            if n.visible {
                Some(&n.key)
            } else {
                None
            }
        })
    }

    /// Iterate every key (visible or hidden) in in-order, paired with
    /// its visibility flag.
    pub(crate) fn iter_total(&self) -> impl Iterator<Item = (&T, bool)> + '_ {
        InOrderIter::new(self).map(move |id| {
            let n = self.node(id);
            (&n.key, n.visible)
        })
    }

    /// Remove a node entirely (used by tombstone GC). Returns the removed key.
    ///
    /// Note: in normal use, tombstoning is done via [`Self::set_visible`];
    /// `remove` permanently deletes the node, breaking any external anchors
    /// that referenced it. Callers must be sure no anchors reference `key`.
    pub(crate) fn remove(&mut self, key: &T) -> Option<T> {
        let id = self.by_key.remove(key)?;
        let removed_key = self.node(id).key.clone();
        self.detach(id);
        // Recycle slot.
        self.nodes[id as usize] = None;
        self.free.push(id);
        Some(removed_key)
    }

    /// Number of nodes (slots in use) — useful for memory accounting.
    #[must_use]
    #[allow(dead_code)] // exposed for future memory benchmarks
    pub(crate) fn node_count(&self) -> usize {
        self.nodes.len() - self.free.len()
    }

    // -----------------------------------------------------------------------
    // Slab management
    // -----------------------------------------------------------------------

    fn alloc(&mut self, key: T, visible: bool) -> NodeId {
        let id = if let Some(reused) = self.free.pop() {
            reused
        } else {
            let id = self.nodes.len() as NodeId;
            self.nodes.push(None);
            id
        };
        self.nodes[id as usize] = Some(Node {
            key: key.clone(),
            visible,
            parent: NIL,
            left: NIL,
            right: NIL,
            height: 1,
            visible_count: u32::from(visible),
            total_count: 1,
        });
        self.by_key.insert(key, id);
        id
    }

    #[inline]
    fn node(&self, id: NodeId) -> &Node<T> {
        debug_assert!(id != NIL);
        self.nodes[id as usize]
            .as_ref()
            .expect("OST: dangling NodeId")
    }

    #[inline]
    fn node_mut(&mut self, id: NodeId) -> &mut Node<T> {
        debug_assert!(id != NIL);
        self.nodes[id as usize]
            .as_mut()
            .expect("OST: dangling NodeId")
    }

    fn height_of(&self, id: NodeId) -> i8 {
        if id == NIL {
            0
        } else {
            self.node(id).height
        }
    }

    fn visible_count_of(&self, id: NodeId) -> u32 {
        if id == NIL {
            0
        } else {
            self.node(id).visible_count
        }
    }

    fn total_count_of(&self, id: NodeId) -> u32 {
        if id == NIL {
            0
        } else {
            self.node(id).total_count
        }
    }

    // -----------------------------------------------------------------------
    // Tree navigation
    // -----------------------------------------------------------------------

    fn rightmost_of(&self, mut id: NodeId) -> NodeId {
        debug_assert!(id != NIL);
        loop {
            let r = self.node(id).right;
            if r == NIL {
                return id;
            }
            id = r;
        }
    }

    /// Find the node at `rank` (0-based) when counting visible-only or all.
    fn find_at(&self, mut rank: usize, visible_only: bool) -> Option<NodeId> {
        let mut cur = self.root;
        if cur == NIL {
            return None;
        }
        let total = if visible_only {
            self.visible_count_of(cur) as usize
        } else {
            self.total_count_of(cur) as usize
        };
        if rank >= total {
            return None;
        }
        loop {
            let n = self.node(cur);
            let left_count = if visible_only {
                self.visible_count_of(n.left) as usize
            } else {
                self.total_count_of(n.left) as usize
            };
            if rank < left_count {
                cur = n.left;
                continue;
            }
            rank -= left_count;
            let self_count = if visible_only {
                u32::from(n.visible) as usize
            } else {
                1
            };
            if rank < self_count {
                return Some(cur);
            }
            rank -= self_count;
            cur = n.right;
        }
    }

    /// Walk up from a node to root, accumulating the count of items
    /// strictly before it in in-order traversal.
    fn position_walk_up(&self, start: NodeId, visible_only: bool) -> usize {
        let n = self.node(start);
        let mut rank = if visible_only {
            self.visible_count_of(n.left) as usize
        } else {
            self.total_count_of(n.left) as usize
        };
        let mut cur = start;
        while self.node(cur).parent != NIL {
            let parent_id = self.node(cur).parent;
            let parent = self.node(parent_id);
            if parent.right == cur {
                let parent_left_count = if visible_only {
                    self.visible_count_of(parent.left) as usize
                } else {
                    self.total_count_of(parent.left) as usize
                };
                let parent_self = if visible_only {
                    u32::from(parent.visible) as usize
                } else {
                    1
                };
                rank += parent_left_count + parent_self;
            }
            cur = parent_id;
        }
        rank
    }

    // -----------------------------------------------------------------------
    // Mutators (attach / detach / rotate / rebalance)
    // -----------------------------------------------------------------------

    /// Attach `child` as the left child of `parent`. `parent`'s existing
    /// left child must be `NIL`.
    fn attach_left(&mut self, parent: NodeId, child: NodeId) {
        debug_assert_eq!(self.node(parent).left, NIL);
        self.node_mut(parent).left = child;
        self.node_mut(child).parent = parent;
        self.update_counts_up(child);
    }

    /// Attach `child` as the right child of `parent`. `parent`'s existing
    /// right child must be `NIL`.
    fn attach_right(&mut self, parent: NodeId, child: NodeId) {
        debug_assert_eq!(self.node(parent).right, NIL);
        self.node_mut(parent).right = child;
        self.node_mut(child).parent = parent;
        self.update_counts_up(child);
    }

    /// Recompute `height`, `visible_count`, and `total_count` for `id`
    /// from its children.
    fn update_counts(&mut self, id: NodeId) {
        let l = self.node(id).left;
        let r = self.node(id).right;
        let lh = self.height_of(l);
        let rh = self.height_of(r);
        let lv = self.visible_count_of(l);
        let rv = self.visible_count_of(r);
        let lt = self.total_count_of(l);
        let rt = self.total_count_of(r);
        let n = self.node_mut(id);
        n.height = 1 + lh.max(rh);
        n.visible_count = lv + u32::from(n.visible) + rv;
        n.total_count = lt + 1 + rt;
    }

    /// Walk from `id` upward to root, calling `update_counts` at each node.
    /// No rotations.
    fn update_counts_up(&mut self, mut id: NodeId) {
        while id != NIL {
            self.update_counts(id);
            id = self.node(id).parent;
        }
    }

    fn balance_factor(&self, id: NodeId) -> i8 {
        if id == NIL {
            0
        } else {
            self.height_of(self.node(id).left) - self.height_of(self.node(id).right)
        }
    }

    /// Walk up from `start` rebalancing as needed. Updates counts at every
    /// node visited.
    fn rebalance_up(&mut self, mut id: NodeId) {
        while id != NIL {
            self.update_counts(id);
            let bf = self.balance_factor(id);
            if bf > 1 {
                // Left-heavy.
                let l = self.node(id).left;
                if self.balance_factor(l) < 0 {
                    // LR case: rotate left at child first.
                    self.rotate_left(l);
                }
                id = self.rotate_right(id);
            } else if bf < -1 {
                // Right-heavy.
                let r = self.node(id).right;
                if self.balance_factor(r) > 0 {
                    // RL case: rotate right at child first.
                    self.rotate_right(r);
                }
                id = self.rotate_left(id);
            }
            id = self.node(id).parent;
        }
    }

    /// Right rotation at `x`. Returns the new top of the rotated subtree.
    /// Counts and parent pointers are fully maintained.
    fn rotate_right(&mut self, x: NodeId) -> NodeId {
        // Before:        After:
        //     x            y
        //    / \          / \
        //   y   c   →    a   x
        //  / \              / \
        // a   b            b   c
        let y = self.node(x).left;
        debug_assert!(y != NIL, "rotate_right requires left child");
        let b = self.node(y).right;
        let x_parent = self.node(x).parent;

        // Re-wire x's left to b.
        self.node_mut(x).left = b;
        if b != NIL {
            self.node_mut(b).parent = x;
        }
        // Re-wire y's right to x.
        self.node_mut(y).right = x;
        self.node_mut(x).parent = y;
        // Re-wire y's parent to x's old parent.
        self.node_mut(y).parent = x_parent;
        if x_parent == NIL {
            self.root = y;
        } else {
            let p = self.node_mut(x_parent);
            if p.left == x {
                p.left = y;
            } else {
                p.right = y;
            }
        }
        // Update counts: x first (its children changed), then y.
        self.update_counts(x);
        self.update_counts(y);
        y
    }

    /// Left rotation at `x`. Returns the new top.
    fn rotate_left(&mut self, x: NodeId) -> NodeId {
        // Before:        After:
        //   x              y
        //  / \            / \
        // a   y     →    x   c
        //    / \        / \
        //   b   c      a   b
        let y = self.node(x).right;
        debug_assert!(y != NIL, "rotate_left requires right child");
        let b = self.node(y).left;
        let x_parent = self.node(x).parent;

        self.node_mut(x).right = b;
        if b != NIL {
            self.node_mut(b).parent = x;
        }
        self.node_mut(y).left = x;
        self.node_mut(x).parent = y;
        self.node_mut(y).parent = x_parent;
        if x_parent == NIL {
            self.root = y;
        } else {
            let p = self.node_mut(x_parent);
            if p.left == x {
                p.left = y;
            } else {
                p.right = y;
            }
        }
        self.update_counts(x);
        self.update_counts(y);
        y
    }

    /// Detach `id` from the tree. Used by [`Self::remove`].
    /// Standard BST deletion: 0/1/2-child cases.
    fn detach(&mut self, id: NodeId) {
        let left = self.node(id).left;
        let right = self.node(id).right;
        let parent = self.node(id).parent;

        let replacement = if left == NIL && right == NIL {
            // Leaf: just unlink.
            NIL
        } else if left == NIL {
            right
        } else if right == NIL {
            left
        } else {
            // Two children: splice in the in-order successor's subtree.
            // Successor = leftmost of right subtree.
            let mut succ = right;
            while self.node(succ).left != NIL {
                succ = self.node(succ).left;
            }
            // Detach succ first (it has at most a right child).
            let succ_right = self.node(succ).right;
            let succ_parent = self.node(succ).parent;
            if succ_parent != id {
                // succ is somewhere down in the right subtree; replace succ
                // with its right child in succ_parent.
                self.replace_child(succ_parent, succ, succ_right);
                if succ_right != NIL {
                    self.node_mut(succ_right).parent = succ_parent;
                }
                // succ takes id's right subtree.
                self.node_mut(succ).right = right;
                self.node_mut(right).parent = succ;
            }
            // succ takes id's left subtree.
            self.node_mut(succ).left = left;
            if left != NIL {
                self.node_mut(left).parent = succ;
            }
            // succ moves into id's slot — its parent pointer becomes id's
            // original parent (NOT NIL — that was a real bug caught by
            // adversarial_inserts_then_remove_root_repeatedly).
            self.node_mut(succ).parent = parent;
            // Update the original parent's child pointer to succ.
            self.replace_in_parent(id, succ);
            // Rebalance from the deepest affected node up. If succ was a
            // direct child of id, succ itself is the deepest affected (it
            // gained id's left subtree); otherwise succ_parent is (it
            // lost succ as a child).
            let rebalance_from = if succ_parent == id { succ } else { succ_parent };
            self.update_counts_up(rebalance_from);
            self.rebalance_up(rebalance_from);
            return;
        };

        // 0- or 1-child case.
        self.replace_in_parent(id, replacement);
        if replacement != NIL {
            self.node_mut(replacement).parent = parent;
        }
        if parent != NIL {
            self.update_counts_up(parent);
            self.rebalance_up(parent);
        }
    }

    /// Replace `id` in its parent's child slot with `with`. Doesn't update
    /// counts or `with.parent`.
    fn replace_in_parent(&mut self, id: NodeId, with: NodeId) {
        let parent = self.node(id).parent;
        if parent == NIL {
            self.root = with;
        } else {
            let p = self.node_mut(parent);
            if p.left == id {
                p.left = with;
            } else {
                debug_assert_eq!(p.right, id);
                p.right = with;
            }
        }
    }

    /// Replace `child` in `parent`'s slot (left or right) with `with`.
    fn replace_child(&mut self, parent: NodeId, child: NodeId, with: NodeId) {
        let p = self.node_mut(parent);
        if p.left == child {
            p.left = with;
        } else {
            debug_assert_eq!(p.right, child);
            p.right = with;
        }
    }

    /// Walk every node and assert structural + augmentation invariants.
    /// Test-only.
    #[cfg(test)]
    pub(crate) fn check_invariants(&self) {
        if self.root == NIL {
            return;
        }
        self.check_subtree(self.root, NIL);
    }

    #[cfg(test)]
    fn check_subtree(&self, id: NodeId, expected_parent: NodeId) -> (i8, u32, u32) {
        let n = self.node(id);
        assert_eq!(n.parent, expected_parent, "node {id} has wrong parent");
        let (lh, lv, lt) = if n.left == NIL {
            (0, 0, 0)
        } else {
            self.check_subtree(n.left, id)
        };
        let (rh, rv, rt) = if n.right == NIL {
            (0, 0, 0)
        } else {
            self.check_subtree(n.right, id)
        };
        let bf = lh - rh;
        assert!(
            bf.abs() <= 1,
            "node {id} balance factor {bf} (height {})",
            n.height
        );
        let h = 1 + lh.max(rh);
        let v = lv + u32::from(n.visible) + rv;
        let t = lt + 1 + rt;
        assert_eq!(n.height, h, "node {id} stored height wrong");
        assert_eq!(n.visible_count, v, "node {id} stored visible_count wrong");
        assert_eq!(n.total_count, t, "node {id} stored total_count wrong");
        (h, v, t)
    }
}

// ---------------------------------------------------------------------------
// In-order iterator
// ---------------------------------------------------------------------------

struct InOrderIter<'a, T: Eq + Hash + Clone> {
    tree: &'a OrderTree<T>,
    next: NodeId,
}

impl<'a, T: Eq + Hash + Clone> InOrderIter<'a, T> {
    fn new(tree: &'a OrderTree<T>) -> Self {
        let mut next = tree.root;
        if next != NIL {
            while tree.node(next).left != NIL {
                next = tree.node(next).left;
            }
        }
        Self { tree, next }
    }
}

impl<T: Eq + Hash + Clone> Iterator for InOrderIter<'_, T> {
    type Item = NodeId;
    fn next(&mut self) -> Option<NodeId> {
        if self.next == NIL {
            return None;
        }
        let yielded = self.next;
        // Advance: in-order successor.
        let n = self.tree.node(yielded);
        if n.right == NIL {
            // Walk up until we come from the left.
            let mut cur = yielded;
            loop {
                let parent = self.tree.node(cur).parent;
                if parent == NIL {
                    self.next = NIL;
                    break;
                }
                if self.tree.node(parent).left == cur {
                    self.next = parent;
                    break;
                }
                cur = parent;
            }
        } else {
            // Leftmost of right subtree.
            let mut cur = n.right;
            while self.tree.node(cur).left != NIL {
                cur = self.tree.node(cur).left;
            }
            self.next = cur;
        }
        Some(yielded)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let t: OrderTree<u32> = OrderTree::new();
        assert!(t.is_empty());
        assert_eq!(t.len(), 0);
        assert_eq!(t.total_len(), 0);
        assert_eq!(t.at_visible(0), None);
        t.check_invariants();
    }

    #[test]
    fn single_insert() {
        let mut t = OrderTree::<u32>::new();
        t.insert_at_total(0, 42, true);
        assert_eq!(t.len(), 1);
        assert_eq!(t.at_visible(0), Some(&42));
        assert_eq!(t.visible_position_of(&42), Some(0));
        t.check_invariants();
    }

    #[test]
    fn append_many() {
        let mut t = OrderTree::<u32>::new();
        for i in 0..100 {
            t.insert_at_total(i as usize, i, true);
            t.check_invariants();
        }
        assert_eq!(t.len(), 100);
        for i in 0..100 {
            assert_eq!(t.at_visible(i as usize), Some(&i));
            assert_eq!(t.visible_position_of(&i), Some(i as usize));
        }
    }

    #[test]
    fn prepend_many() {
        let mut t = OrderTree::<u32>::new();
        for i in 0..100 {
            t.insert_at_total(0, i, true);
            t.check_invariants();
        }
        // After 100 prepends, the values are in reverse-insertion order.
        for i in 0..100 {
            assert_eq!(t.at_visible(i as usize), Some(&(99 - i)));
        }
    }

    #[test]
    fn random_inserts() {
        use rand::{Rng, SeedableRng};
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(42);
        let mut t = OrderTree::<u32>::new();
        let mut reference: Vec<u32> = Vec::new();
        for i in 0..200 {
            let pos = rng.gen_range(0..=reference.len());
            t.insert_at_total(pos, i, true);
            reference.insert(pos, i);
            t.check_invariants();
        }
        assert_eq!(t.len(), reference.len());
        for (i, expected) in reference.iter().enumerate() {
            assert_eq!(t.at_visible(i), Some(expected));
            assert_eq!(t.visible_position_of(expected), Some(i));
        }
    }

    #[test]
    fn set_visible_toggles_count() {
        let mut t = OrderTree::<u32>::new();
        for i in 0..10 {
            t.insert_at_total(i as usize, i, true);
        }
        assert_eq!(t.len(), 10);
        t.set_visible(&5, false);
        t.check_invariants();
        assert_eq!(t.len(), 9);
        // Position of 6 (was at rank 6) is now 5 in visible-only, 6 in total.
        assert_eq!(t.visible_position_of(&6), Some(5));
        assert_eq!(t.total_position_of(&6), Some(6));
        // 5 is hidden — visible_position_of returns None.
        assert_eq!(t.visible_position_of(&5), None);
        // Phantom position: 5's phantom rank is 5 (number of visible before its
        // total position 5).
        assert_eq!(t.phantom_visible_position_of(&5), Some(5));
        t.set_visible(&5, true);
        assert_eq!(t.len(), 10);
        assert_eq!(t.visible_position_of(&5), Some(5));
    }

    #[test]
    fn iter_in_order() {
        let mut t = OrderTree::<u32>::new();
        for i in 0..20 {
            t.insert_at_total(i as usize, i, true);
        }
        let collected: Vec<u32> = t.iter_visible().copied().collect();
        assert_eq!(collected, (0..20).collect::<Vec<_>>());
    }

    #[test]
    fn iter_visible_skips_tombstoned() {
        let mut t = OrderTree::<u32>::new();
        for i in 0..10 {
            t.insert_at_total(i as usize, i, true);
        }
        t.set_visible(&3, false);
        t.set_visible(&7, false);
        let collected: Vec<u32> = t.iter_visible().copied().collect();
        assert_eq!(collected, vec![0, 1, 2, 4, 5, 6, 8, 9]);
        let total: Vec<(u32, bool)> = t.iter_total().map(|(v, vis)| (*v, vis)).collect();
        assert_eq!(
            total,
            vec![
                (0, true),
                (1, true),
                (2, true),
                (3, false),
                (4, true),
                (5, true),
                (6, true),
                (7, false),
                (8, true),
                (9, true),
            ]
        );
    }

    #[test]
    fn remove_works() {
        let mut t = OrderTree::<u32>::new();
        for i in 0..15 {
            t.insert_at_total(i as usize, i, true);
        }
        assert_eq!(t.remove(&7), Some(7));
        t.check_invariants();
        assert!(!t.contains(&7));
        assert_eq!(t.len(), 14);
        // Items beyond the removed one shift down by 1.
        for i in 0..14 {
            let expected = if (i as u32) < 7 {
                i as u32
            } else {
                (i + 1) as u32
            };
            assert_eq!(t.at_visible(i), Some(&expected));
        }
    }

    #[test]
    fn position_of_walks_up_correctly() {
        // Build a tall tree by always-prepending.
        let mut t = OrderTree::<u32>::new();
        for i in 0..200u32 {
            t.insert_at_total(0, i, true);
            t.check_invariants();
        }
        // Visible order is 199, 198, ..., 0.
        for i in 0..200u32 {
            let expected_rank = (199 - i) as usize;
            assert_eq!(t.visible_position_of(&i), Some(expected_rank));
        }
    }

    #[test]
    fn adversarial_alternating_inserts() {
        // Alternate between inserting at the front and at the back. This
        // pattern stress-tests the AVL rebalancer because the tree
        // alternately leans left and right.
        let mut t = OrderTree::<u32>::new();
        for i in 0..500u32 {
            if i % 2 == 0 {
                t.insert_at_total(0, i, true);
            } else {
                t.insert_at_total(t.total_len(), i, true);
            }
            t.check_invariants();
        }
        assert_eq!(t.len(), 500);
    }

    #[test]
    fn adversarial_zigzag_inserts() {
        // Insert at near-middle positions alternating directions.
        let mut t = OrderTree::<u32>::new();
        for i in 0..500u32 {
            let mid = t.total_len() / 2;
            let pos = if i % 2 == 0 {
                mid
            } else {
                (mid + 1).min(t.total_len())
            };
            t.insert_at_total(pos, i, true);
            t.check_invariants();
        }
        assert_eq!(t.len(), 500);
    }

    #[test]
    fn adversarial_inserts_then_remove_root_repeatedly() {
        // Build up, then keep removing the root (in-order middle). Tests
        // remove + AVL rebalance under stress.
        let mut t = OrderTree::<u32>::new();
        for i in 0..200u32 {
            t.insert_at_total(t.total_len(), i, true);
        }
        t.check_invariants();
        // Remove the median repeatedly.
        for _ in 0..150 {
            let mid = t.total_len() / 2;
            let val = *t.at_total(mid).unwrap();
            t.remove(&val);
            t.check_invariants();
        }
        assert_eq!(t.len(), 50);
    }

    #[test]
    fn adversarial_set_visible_thrashing() {
        let mut t = OrderTree::<u32>::new();
        for i in 0..200u32 {
            t.insert_at_total(t.total_len(), i, true);
        }
        for round in 0..10 {
            for i in 0..200u32 {
                t.set_visible(&i, (i + round) % 2 == 0);
            }
            t.check_invariants();
            // Visible count must equal expected.
            let expected = (0..200u32).filter(|i| (i + round) % 2 == 0).count();
            assert_eq!(t.len(), expected);
        }
    }

    #[test]
    fn random_mixed_workload() {
        use rand::{Rng, SeedableRng};
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(2026);
        let mut t = OrderTree::<u32>::new();
        let mut reference: Vec<u32> = Vec::new();
        let mut next_key = 0u32;
        for _ in 0..1000 {
            match rng.gen_range(0..3) {
                0 | 1 => {
                    // Insert at a visible position.
                    let pos = rng.gen_range(0..=reference.len());
                    t.insert_at_visible(pos, next_key);
                    reference.insert(pos, next_key);
                    next_key += 1;
                }
                _ if !reference.is_empty() => {
                    // Hide a random visible item.
                    let idx = rng.gen_range(0..reference.len());
                    let key = reference[idx];
                    t.set_visible(&key, false);
                    reference.remove(idx);
                }
                _ => {}
            }
            t.check_invariants();
        }
        assert_eq!(t.len(), reference.len());
        for (i, expected) in reference.iter().enumerate() {
            assert_eq!(t.at_visible(i), Some(expected));
        }
    }

    #[test]
    fn insert_at_visible_threads_around_tombstones() {
        let mut t = OrderTree::<u32>::new();
        // Build [10, 20, 30, 40, 50] at total ranks 0..5, all visible.
        for (i, k) in [10, 20, 30, 40, 50].iter().enumerate() {
            t.insert_at_visible(i, *k);
        }
        // Hide 20 and 40 — visible is now [10, 30, 50].
        t.set_visible(&20, false);
        t.set_visible(&40, false);
        assert_eq!(t.len(), 3);
        assert_eq!(t.at_visible(0), Some(&10));
        assert_eq!(t.at_visible(1), Some(&30));
        assert_eq!(t.at_visible(2), Some(&50));
        // Insert 999 at visible rank 1 — should be just before 30 in visible
        // order, but AFTER the tombstoned 20.
        t.insert_at_visible(1, 999);
        t.check_invariants();
        assert_eq!(t.at_visible(0), Some(&10));
        assert_eq!(t.at_visible(1), Some(&999));
        assert_eq!(t.at_visible(2), Some(&30));
        assert_eq!(t.at_visible(3), Some(&50));
        // Total order: [10, 20(t), 999, 30, 40(t), 50].
        assert_eq!(t.at_total(0), Some(&10));
        assert_eq!(t.at_total(1), Some(&20));
        assert_eq!(t.at_total(2), Some(&999));
        assert_eq!(t.at_total(3), Some(&30));
        assert_eq!(t.at_total(4), Some(&40));
        assert_eq!(t.at_total(5), Some(&50));
    }
}
