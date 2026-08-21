//! LWW-Map demo: two replicas concurrently set the same key.

use abyo_crdt::Map;

fn main() {
    let mut alice: Map<&'static str, i32> = Map::new(1);
    let mut bob: Map<&'static str, i32> = Map::new(2);

    alice.set("score", 10);
    alice.set("level", 5);

    bob.merge(&alice);
    println!("After initial sync:");
    println!("  alice score = {:?}", alice.get(&"score"));
    println!("  bob   score = {:?}", bob.get(&"score"));

    // Both replicas update "score" while disconnected.
    alice.set("score", 100);
    bob.set("score", 200);

    println!("\nAfter concurrent edits, before sync:");
    println!("  alice score = {:?}", alice.get(&"score"));
    println!("  bob   score = {:?}", bob.get(&"score"));

    // Cross-merge.
    let alice_clone = alice.clone();
    alice.merge(&bob);
    bob.merge(&alice_clone);

    println!("\nAfter cross-merge (LWW resolves to higher OpId):");
    println!("  alice score = {:?}", alice.get(&"score"));
    println!("  bob   score = {:?}", bob.get(&"score"));
    assert_eq!(alice.get(&"score"), bob.get(&"score"));
}
