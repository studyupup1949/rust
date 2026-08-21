//! Round-trip tests for the `serde` feature.

#![cfg(feature = "serde")]

use abyo_crdt::{List, ListOp};

#[test]
fn json_round_trip_preserves_state() {
    let mut a = List::<char>::new(1);
    for (i, c) in "Hello, world!".chars().enumerate() {
        a.insert(i, c);
    }
    a.delete(0); // drop the 'H'

    let json = serde_json::to_string(&a).expect("serialize");
    let restored: List<char> = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(restored.to_vec(), a.to_vec());
    assert_eq!(restored.len(), a.len());
    assert_eq!(restored.ops().len(), a.ops().len());
}

#[test]
fn bincode_round_trip_preserves_state() {
    let mut a = List::<u32>::new(7);
    for i in 0..50u32 {
        a.insert(i as usize, i);
    }
    for _ in 0..10 {
        a.delete(0);
    }

    let bytes = bincode::serialize(&a).expect("serialize");
    let restored: List<u32> = bincode::deserialize(&bytes).expect("deserialize");

    assert_eq!(restored.to_vec(), a.to_vec());
}

#[test]
fn restored_replica_can_continue_collaborating() {
    let mut a = List::<char>::new(1);
    a.insert(0, 'a');
    a.insert(1, 'b');

    // Persist + restore
    let bytes = bincode::serialize(&a).expect("serialize");
    let mut a_restored: List<char> = bincode::deserialize(&bytes).expect("deserialize");

    // Continue editing on the restored copy.
    a_restored.insert(2, 'c');

    // A different replica that knows the original state should be able to
    // catch up using the new ops.
    let mut b = List::<char>::new(2);
    b.merge(&a); // catches up to "ab"

    // Now apply only the new op from a_restored.
    let new_ops: Vec<ListOp<char>> = a_restored.ops_since(b.version()).cloned().collect();
    assert_eq!(new_ops.len(), 1);
    for op in new_ops {
        b.apply(op).unwrap();
    }
    assert_eq!(b.to_vec(), vec!['a', 'b', 'c']);
}

#[test]
fn op_serializes_alone() {
    let mut a = List::<u8>::new(1);
    let op = a.insert(0, 42);
    let json = serde_json::to_string(&op).expect("serialize");
    let restored: ListOp<u8> = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(op, restored);
}

#[test]
fn version_vector_serializes() {
    let mut a = List::<u8>::new(1);
    a.insert(0, 1);
    a.insert(1, 2);
    let v = a.version().clone();
    let json = serde_json::to_string(&v).expect("serialize");
    let restored: abyo_crdt::VersionVector = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(v, restored);
}
