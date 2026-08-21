//! Round-trip tests for the v0.2 CRDTs (Map, Counter, Set) under serde.

#![cfg(feature = "serde")]

use abyo_crdt::{Counter, Map, Set};

#[test]
fn map_json_round_trip() {
    let mut a: Map<String, i32> = Map::new(1);
    a.set("x".into(), 1);
    a.set("y".into(), 2);
    a.remove("x".into());
    let json = serde_json::to_string(&a).expect("ser");
    let restored: Map<String, i32> = serde_json::from_str(&json).expect("de");
    assert_eq!(a.get(&"y".into()), restored.get(&"y".into()));
    assert_eq!(
        a.contains_key(&"x".into()),
        restored.contains_key(&"x".into())
    );
    assert_eq!(a.len(), restored.len());
}

#[test]
fn map_bincode_round_trip() {
    let mut a: Map<u32, String> = Map::new(7);
    for i in 0..50u32 {
        a.set(i, format!("v{i}"));
    }
    let bytes = bincode::serialize(&a).unwrap();
    let restored: Map<u32, String> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(a.len(), restored.len());
    for i in 0..50u32 {
        assert_eq!(a.get(&i), restored.get(&i));
    }
}

#[test]
fn counter_json_round_trip() {
    let mut c = Counter::new(1);
    c.add(10);
    c.add(-3);
    c.add(7);
    let json = serde_json::to_string(&c).expect("ser");
    let restored: Counter = serde_json::from_str(&json).expect("de");
    assert_eq!(c.value(), restored.value());
    assert_eq!(c.positive_total(), restored.positive_total());
    assert_eq!(c.negative_total(), restored.negative_total());
}

#[test]
fn set_json_round_trip() {
    let mut s: Set<String> = Set::new(1);
    s.add("a".into());
    s.add("b".into());
    s.add("c".into());
    s.remove(&"a".into());
    let json = serde_json::to_string(&s).expect("ser");
    let restored: Set<String> = serde_json::from_str(&json).expect("de");
    assert_eq!(s.contains(&"a".into()), restored.contains(&"a".into()));
    assert_eq!(s.contains(&"b".into()), restored.contains(&"b".into()));
    assert_eq!(s.contains(&"c".into()), restored.contains(&"c".into()));
    assert_eq!(s.len(), restored.len());
}

#[test]
fn restored_can_continue_collaborating() {
    let mut a = Counter::new(1);
    a.add(5);
    let bytes = bincode::serialize(&a).unwrap();

    let mut a_restored: Counter = bincode::deserialize(&bytes).unwrap();
    a_restored.add(10);

    let mut b = Counter::new(2);
    b.merge(&a);
    // Apply only the new ops.
    let new_ops: Vec<_> = a_restored.ops_since(b.version()).copied().collect();
    for op in new_ops {
        b.apply(op).unwrap();
    }
    assert_eq!(b.value(), 15);
    assert_eq!(a_restored.value(), 15);
}
