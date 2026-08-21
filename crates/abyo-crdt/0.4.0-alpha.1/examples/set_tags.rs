//! OR-Set demo: tags on a document, with concurrent add+remove.
//!
//! Add wins: if Alice removes "urgent" while Bob concurrently re-adds
//! "urgent", the tag survives in the merged state.

use abyo_crdt::Set;

fn main() {
    let mut alice: Set<&'static str> = Set::new(1);
    let mut bob: Set<&'static str> = Set::new(2);

    alice.add("urgent");
    alice.add("frontend");
    bob.merge(&alice);

    println!("Initial state (both replicas):");
    print_set("alice", &alice);
    print_set("bob", &bob);

    // Alice decides "urgent" is wrong, removes it.
    // Bob concurrently re-adds "urgent" because he disagrees.
    alice.remove(&"urgent");
    bob.add("urgent");

    // They also each add a unique tag.
    alice.add("alice-tag");
    bob.add("bob-tag");

    println!("\nBefore sync:");
    print_set("alice", &alice);
    print_set("bob", &bob);

    // Cross-merge.
    let alice_clone = alice.clone();
    alice.merge(&bob);
    bob.merge(&alice_clone);

    println!("\nAfter cross-merge (add wins):");
    print_set("alice", &alice);
    print_set("bob", &bob);

    // "urgent" survives because bob's add was concurrent with alice's remove.
    assert!(alice.contains(&"urgent"));
    assert!(bob.contains(&"urgent"));
}

fn print_set(name: &str, s: &Set<&'static str>) {
    let mut items: Vec<&&str> = s.iter().collect();
    items.sort();
    println!("  {name}: {items:?}");
}
