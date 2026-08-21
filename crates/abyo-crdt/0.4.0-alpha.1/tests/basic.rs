//! Hand-written tests for the public List API.
//!
//! These tests exercise concrete scenarios — the convergence test suite in
//! `convergence.rs` covers randomized exhaustive checking.

use abyo_crdt::{List, ListOp, Side};

fn s(list: &List<char>) -> String {
    list.iter().collect()
}

// ---------------------------------------------------------------------------
// Single-replica behavior
// ---------------------------------------------------------------------------

#[test]
fn build_string_via_appends() {
    let mut list = List::<char>::new(1);
    for (i, c) in "abcdef".chars().enumerate() {
        list.insert(i, c);
    }
    assert_eq!(s(&list), "abcdef");
    assert_eq!(list.len(), 6);
}

#[test]
fn build_string_via_prepends() {
    let mut list = List::<char>::new(1);
    for c in "abcdef".chars().rev() {
        list.insert(0, c);
    }
    assert_eq!(s(&list), "abcdef");
}

#[test]
fn alternating_inserts_and_deletes() {
    let mut list = List::<char>::new(1);
    list.insert(0, 'a');
    list.insert(1, 'b');
    list.insert(2, 'c');
    list.delete(1); // remove 'b'
    list.insert(1, 'X');
    assert_eq!(s(&list), "aXc");
}

#[test]
fn deleting_already_deleted_position_panics() {
    // Deleting the same *visible position* after a delete is fine —
    // it now refers to a different item. This test just exercises the path.
    let mut list = List::<char>::new(1);
    list.insert(0, 'a');
    list.insert(1, 'b');
    list.delete(0);
    list.delete(0); // now deletes 'b'
    assert!(list.is_empty());
}

#[test]
fn try_insert_oob_returns_error() {
    let mut list = List::<char>::new(1);
    let err = list.try_insert(1, 'x').unwrap_err();
    assert!(matches!(
        err,
        abyo_crdt::Error::OutOfBounds { pos: 1, len: 0 }
    ));
}

#[test]
fn try_delete_oob_returns_error() {
    let mut list = List::<char>::new(1);
    let err = list.try_delete(0).unwrap_err();
    assert!(matches!(
        err,
        abyo_crdt::Error::OutOfBounds { pos: 0, len: 0 }
    ));
}

// ---------------------------------------------------------------------------
// Two-replica merge behavior
// ---------------------------------------------------------------------------

#[test]
fn empty_merge() {
    let mut a = List::<char>::new(1);
    let b = List::<char>::new(2);
    a.merge(&b);
    assert_eq!(s(&a), "");
}

#[test]
fn merge_after_disjoint_history() {
    let mut a = List::<char>::new(1);
    let mut b = List::<char>::new(2);
    a.insert(0, 'a');
    a.insert(1, 'b');
    b.merge(&a);
    b.insert(2, 'c');
    a.merge(&b);
    assert_eq!(s(&a), "abc");
    assert_eq!(s(&b), "abc");
}

#[test]
fn concurrent_inserts_at_same_pos_converge() {
    // Two replicas, both start from "ab", both insert one char at pos 1.
    let mut a = List::<char>::new(1);
    let mut b = List::<char>::new(2);
    a.insert(0, 'a');
    a.insert(1, 'b');
    b.merge(&a);

    a.insert(1, 'X');
    b.insert(1, 'Y');

    // Cross-merge.
    let mut a2 = a.clone();
    a2.merge(&b);
    let mut b2 = b.clone();
    b2.merge(&a);

    assert_eq!(s(&a2), s(&b2));
    let merged = s(&a2);
    // Either "aXYb" or "aYXb" depending on Lamport order; never interleaved.
    assert!(merged == "aXYb" || merged == "aYXb", "got {merged}");
}

#[test]
fn concurrent_bursts_do_not_interleave() {
    // The Fugue-Maximal headline guarantee: contiguous bursts stay contiguous.
    let mut a = List::<char>::new(1);
    let mut b = List::<char>::new(2);

    // Shared starting state: "ab"
    a.insert(0, 'a');
    a.insert(1, 'b');
    b.merge(&a);

    // Concurrently:
    //   a types "Hello" between a and b
    //   b types "World" between a and b
    for (i, c) in "Hello".chars().enumerate() {
        a.insert(1 + i, c);
    }
    for (i, c) in "World".chars().enumerate() {
        b.insert(1 + i, c);
    }

    let mut a2 = a.clone();
    a2.merge(&b);
    let mut b2 = b.clone();
    b2.merge(&a);

    let merged_a = s(&a2);
    let merged_b = s(&b2);
    assert_eq!(merged_a, merged_b);
    assert!(
        merged_a == "aHelloWorldb" || merged_a == "aWorldHellob",
        "interleaving detected: {merged_a}"
    );
}

#[test]
fn three_way_concurrent_burst() {
    let mut a = List::<char>::new(1);
    let mut b = List::<char>::new(2);
    let mut c = List::<char>::new(3);

    // Shared "x|y" starting state.
    a.insert(0, 'x');
    a.insert(1, 'y');
    b.merge(&a);
    c.merge(&a);

    for (i, ch) in "AAA".chars().enumerate() {
        a.insert(1 + i, ch);
    }
    for (i, ch) in "BBB".chars().enumerate() {
        b.insert(1 + i, ch);
    }
    for (i, ch) in "CCC".chars().enumerate() {
        c.insert(1 + i, ch);
    }

    // Merge all three pairwise.
    let mut all = a.clone();
    all.merge(&b);
    all.merge(&c);

    let mut all2 = b.clone();
    all2.merge(&c);
    all2.merge(&a);

    let mut all3 = c.clone();
    all3.merge(&a);
    all3.merge(&b);

    let merged = s(&all);
    assert_eq!(merged, s(&all2));
    assert_eq!(merged, s(&all3));

    // Each three-letter block must remain contiguous.
    for run in ["AAA", "BBB", "CCC"] {
        assert!(
            merged.contains(run),
            "burst {run} got interleaved in {merged}"
        );
    }
    assert!(merged.starts_with('x'), "lost 'x' in {merged}");
    assert!(merged.ends_with('y'), "lost 'y' in {merged}");
}

#[test]
fn concurrent_delete_and_insert() {
    let mut a = List::<char>::new(1);
    let mut b = List::<char>::new(2);
    a.insert(0, 'a');
    a.insert(1, 'b');
    a.insert(2, 'c');
    b.merge(&a);

    a.delete(1); // delete 'b'
    b.insert(2, 'X'); // insert between b and c

    let mut a2 = a.clone();
    a2.merge(&b);
    let mut b2 = b.clone();
    b2.merge(&a);

    assert_eq!(s(&a2), s(&b2));
    // 'b' is deleted; 'X' was inserted between (now-deleted) 'b' and 'c'.
    // Visible order: a, X, c
    assert_eq!(s(&a2), "aXc");
}

#[test]
fn concurrent_deletes_of_same_item() {
    let mut a = List::<char>::new(1);
    let mut b = List::<char>::new(2);
    a.insert(0, 'a');
    a.insert(1, 'b');
    b.merge(&a);

    a.delete(0);
    b.delete(0);

    let mut a2 = a.clone();
    a2.merge(&b);
    let mut b2 = b.clone();
    b2.merge(&a);

    assert_eq!(s(&a2), s(&b2));
    assert_eq!(s(&a2), "b");
}

// ---------------------------------------------------------------------------
// Idempotency
// ---------------------------------------------------------------------------

#[test]
fn applying_same_op_twice_is_no_op() {
    let mut a = List::<char>::new(1);
    let op1 = a.insert(0, 'a');
    let op2 = a.insert(1, 'b');

    let mut b = List::<char>::new(2);
    b.apply(op1.clone()).unwrap();
    b.apply(op2.clone()).unwrap();
    // Apply again — should be silently ignored.
    b.apply(op1).unwrap();
    b.apply(op2).unwrap();

    assert_eq!(s(&b), "ab");
}

#[test]
fn merge_is_idempotent() {
    let mut a = List::<char>::new(1);
    let mut b = List::<char>::new(2);
    a.insert(0, 'h');
    a.insert(1, 'i');
    b.merge(&a);
    b.merge(&a);
    b.merge(&a);
    assert_eq!(s(&b), "hi");
}

#[test]
fn merge_is_commutative() {
    let mut a1 = List::<char>::new(1);
    let mut a2 = List::<char>::new(1);
    let mut b1 = List::<char>::new(2);
    let mut b2 = List::<char>::new(2);
    let mut c1 = List::<char>::new(3);
    let mut c2 = List::<char>::new(3);

    a1.insert(0, 'a');
    a2.insert(0, 'a');
    b1.insert(0, 'b');
    b2.insert(0, 'b');
    c1.insert(0, 'c');
    c2.insert(0, 'c');

    // Path 1: a ← b ← c
    let mut path1 = a1;
    path1.merge(&b1);
    path1.merge(&c1);

    // Path 2: c ← a ← b
    let mut path2 = c2;
    path2.merge(&a2);
    path2.merge(&b2);

    assert_eq!(s(&path1), s(&path2));
}

// ---------------------------------------------------------------------------
// Apply API (manual op delivery)
// ---------------------------------------------------------------------------

#[test]
fn apply_propagates_changes() {
    let mut a = List::<char>::new(1);
    let mut b = List::<char>::new(2);

    let op1 = a.insert(0, 'h');
    let op2 = a.insert(1, 'i');

    b.apply(op1).unwrap();
    b.apply(op2).unwrap();
    assert_eq!(s(&b), "hi");
}

#[test]
fn missing_parent_errors_then_succeeds_after_delivery() {
    let mut a = List::<char>::new(1);
    let mut b = List::<char>::new(2);

    let op1 = a.insert(0, 'h');
    let op2 = a.insert(1, 'i');

    // Delivery out of order: op2 first.
    let err = b.apply(op2.clone()).unwrap_err();
    assert!(matches!(err, abyo_crdt::Error::MissingParent { .. }));

    // Now in correct order.
    b.apply(op1).unwrap();
    b.apply(op2).unwrap();
    assert_eq!(s(&b), "hi");
}

#[test]
fn ops_since_returns_only_unseen() {
    let mut a = List::<char>::new(1);
    a.insert(0, 'x');
    a.insert(1, 'y');

    let initial_v = a.version().clone();
    a.insert(2, 'z');

    let new_ops: Vec<&ListOp<char>> = a.ops_since(&initial_v).collect();
    assert_eq!(new_ops.len(), 1);
    matches!(new_ops[0], ListOp::Insert { value: 'z', .. });
}

#[test]
fn op_has_id() {
    let mut a = List::<char>::new(42);
    let op = a.insert(0, 'a');
    assert_eq!(op.id().replica, 42);
    assert_eq!(op.id().counter, 1);
    if let ListOp::Insert { side, parent, .. } = op {
        assert_eq!(parent, None);
        assert_eq!(side, Side::Right);
    } else {
        panic!("expected Insert");
    }
}

// ---------------------------------------------------------------------------
// GC / log compaction
// ---------------------------------------------------------------------------

#[test]
fn gc_cascade_removes_all_tombstones() {
    let mut a = List::<char>::new(1);
    for (i, c) in "hello".chars().enumerate() {
        a.insert(i, c);
    }
    // Delete every char.
    for _ in 0..5 {
        a.delete(0);
    }
    let frontier = a.version().clone();
    // GC repeatedly — leaves cascade up.
    let mut total = 0;
    loop {
        let n = a.gc(&frontier);
        if n == 0 {
            break;
        }
        total += n;
    }
    assert_eq!(total, 5);
    assert_eq!(a.len(), 0);
    assert!(a.to_vec().is_empty());
}

#[test]
fn gc_preserves_visible_items_with_children() {
    let mut a = List::<char>::new(1);
    a.insert(0, 'a');
    a.insert(1, 'b');
    a.insert(2, 'c');
    a.delete(1); // delete 'b'; b has 'c' as child so can't GC
    let frontier = a.version().clone();
    a.gc(&frontier);
    assert_eq!(a.to_vec(), vec!['a', 'c']);
    // Subsequent insert still works.
    a.insert(2, 'd');
    assert_eq!(a.to_vec(), vec!['a', 'c', 'd']);
}

#[test]
fn gc_respects_empty_frontier() {
    let mut a = List::<char>::new(1);
    a.insert(0, 'x');
    a.delete(0);
    let empty = abyo_crdt::VersionVector::new();
    assert_eq!(a.gc(&empty), 0);
    let full = a.version().clone();
    assert!(a.gc(&full) >= 1);
}

#[test]
fn compact_log_drops_observed_ops() {
    let mut a = List::<char>::new(1);
    a.insert(0, 'a');
    a.insert(1, 'b');
    a.insert(2, 'c');
    let v_after_3 = a.version().clone();
    a.insert(3, 'd');
    assert_eq!(a.ops().len(), 4);
    let removed = a.compact_log(&v_after_3);
    assert_eq!(removed, 3);
    assert_eq!(a.ops().len(), 1);
}

// ---------------------------------------------------------------------------
// Undo / redo
// ---------------------------------------------------------------------------

#[test]
fn undo_insert_tombstones() {
    let mut a = List::<char>::new(1);
    let op = a.insert(0, 'X');
    assert_eq!(s(&a), "X");
    let inv = a.apply_inverse(&op).unwrap();
    assert!(s(&a).is_empty());
    // The inverse is a Delete.
    assert!(matches!(inv, ListOp::Delete { .. }));
}

#[test]
fn undo_delete_re_inserts_value() {
    let mut a = List::<char>::new(1);
    a.insert(0, 'a');
    a.insert(1, 'b');
    a.insert(2, 'c');
    let del_op = a.delete(1); // delete 'b'
    assert_eq!(s(&a), "ac");
    let inv = a.apply_inverse(&del_op).unwrap();
    // Re-inserted 'b' between 'a' and 'c'.
    assert_eq!(s(&a), "abc");
    assert!(matches!(inv, ListOp::Insert { value: 'b', .. }));
}

#[test]
fn concurrent_undo_of_delete_converges() {
    // Replica 1 deletes 'b' then undoes; replica 2 concurrently inserts 'Y'
    // where 'b' was. After full merge, both replicas agree.
    let mut a = List::<char>::new(1);
    a.insert(0, 'a');
    a.insert(1, 'b');
    a.insert(2, 'c');
    let mut b = List::<char>::new(2);
    b.merge(&a);

    // Replica 1: delete 'b' then undo.
    let del = a.delete(1);
    a.apply_inverse(&del); // restores something visible

    // Replica 2 concurrently: insert 'Y' between 'a' and 'b' (a's view: 'b' still there).
    b.insert(1, 'Y');

    // Cross merge.
    let snap = a.clone();
    a.merge(&b);
    b.merge(&snap);

    // Convergence is the only guarantee — exact position of restored 'b'
    // is implementation-defined for concurrent undo, but BOTH replicas must agree.
    assert_eq!(s(&a), s(&b), "diverged: a={:?} b={:?}", s(&a), s(&b));
}

#[test]
fn two_replicas_independently_undo_same_delete_converge() {
    // Both replicas see the same delete and both undo it. Convergence test.
    let mut a = List::<char>::new(1);
    a.insert(0, 'X');
    let mut b = List::<char>::new(2);
    b.merge(&a);

    let del_a = a.delete(0); // a deletes X.
    b.merge(&a); // b sees the delete.
                 // Both have same delete op_id. Each independently undoes.
    a.apply_inverse(&del_a); // a re-inserts X
    b.apply_inverse(&del_a); // b also re-inserts X (different new OpId on b)

    // Cross merge. Two restored-X items will exist (one from each replica).
    let snap = a.clone();
    a.merge(&b);
    b.merge(&snap);

    assert_eq!(s(&a), s(&b), "diverged: a={:?} b={:?}", s(&a), s(&b));
    // Both replicas should see TWO 'X's now (one from each replica's undo).
    assert_eq!(s(&a), "XX");
}

#[test]
fn undo_redo_round_trip() {
    let mut a = List::<char>::new(1);
    let op_a = a.insert(0, 'a');
    let op_b = a.insert(1, 'b');
    let op_c = a.insert(2, 'c');
    assert_eq!(s(&a), "abc");

    // Undo all three.
    let inv_c = a.apply_inverse(&op_c).unwrap();
    let inv_b = a.apply_inverse(&op_b).unwrap();
    let inv_a = a.apply_inverse(&op_a).unwrap();
    assert_eq!(s(&a), "");

    // Redo all three (apply inverses of the inverses).
    a.apply_inverse(&inv_a).unwrap();
    a.apply_inverse(&inv_b).unwrap();
    a.apply_inverse(&inv_c).unwrap();
    assert_eq!(s(&a), "abc");
}

// ---------------------------------------------------------------------------
// to_string convenience
// ---------------------------------------------------------------------------

#[test]
fn list_char_to_string() {
    let mut list = List::<char>::new(1);
    for (i, c) in "abc".chars().enumerate() {
        list.insert(i, c);
    }
    assert_eq!(list.to_string(), "abc");
}
