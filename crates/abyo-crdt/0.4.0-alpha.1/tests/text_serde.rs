//! Serde round-trip tests for the [`Text`] CRDT.

#![cfg(feature = "serde")]

use abyo_crdt::Text;

fn render_string(t: &Text) -> String {
    t.iter_with_marks()
        .map(|(c, marks)| {
            let names: Vec<&str> = marks.iter().collect();
            if names.is_empty() {
                c.to_string()
            } else {
                format!("{c}[{}]", names.join(","))
            }
        })
        .collect()
}

#[test]
fn json_round_trip_with_marks() {
    let mut t = Text::new(1);
    t.insert_str(0, "Hello, world!");
    t.set_mark(0..5, "bold", true);
    t.set_mark(7..12, "italic", true);

    let json = serde_json::to_string(&t).expect("serialize");
    let restored: Text = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(render_string(&t), render_string(&restored));
}

#[test]
fn bincode_round_trip_with_marks() {
    let mut t = Text::new(7);
    t.insert_str(0, "abcdef");
    t.set_mark(2..5, "bold", true);
    t.set_mark(0..3, "italic", true);
    t.delete(0);

    let bytes = bincode::serialize(&t).expect("serialize");
    let restored: Text = bincode::deserialize(&bytes).expect("deserialize");
    assert_eq!(render_string(&t), render_string(&restored));
}

#[test]
fn restored_continues_collaboration() {
    let mut alice = Text::new(1);
    alice.insert_str(0, "Hello");
    alice.set_mark(0..5, "bold", true);

    // Persist + restore.
    let bytes = bincode::serialize(&alice).expect("ser");
    let mut alice_resumed: Text = bincode::deserialize(&bytes).expect("de");

    // Continue editing on the restored copy.
    alice_resumed.insert_str(5, "!");
    alice_resumed.set_mark(0..6, "italic", true);

    // A fresh peer catches up via the original alice's log + the new ops.
    let mut bob = Text::new(2);
    bob.merge(&alice);
    let new_ops: Vec<_> = alice_resumed.ops_since(bob.version()).cloned().collect();
    for op in new_ops {
        bob.apply(op).unwrap();
    }
    assert_eq!(render_string(&alice_resumed), render_string(&bob));
}
