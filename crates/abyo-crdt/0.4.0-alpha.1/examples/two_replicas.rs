//! Two-replica round-trip demo: alice and bob collaborate on a `String`.

use abyo_crdt::List;

fn main() {
    // Each replica gets a unique 64-bit id.
    let mut alice = List::<char>::new(1);
    let mut bob = List::<char>::new(2);

    // --- Alice types "Hi" while Bob is offline. ---
    for (i, c) in "Hi".chars().enumerate() {
        alice.insert(i, c);
    }
    println!("alice: {:?}", alice.to_string());
    println!("bob:   {:?}", bob.to_string());

    // --- Bob comes online; Alice sends her log. ---
    println!("\n[Bob merges Alice's log]");
    bob.merge(&alice);
    println!("alice: {:?}", alice.to_string());
    println!("bob:   {:?}", bob.to_string());

    // --- Both edit concurrently. ---
    alice.insert(2, '!');
    bob.insert(0, ':');
    bob.insert(1, ':');

    println!("\n[After concurrent edits, before sync]");
    println!("alice: {:?}", alice.to_string());
    println!("bob:   {:?}", bob.to_string());

    // --- Cross-merge. ---
    let bob_snapshot = bob.clone();
    bob.merge(&alice);
    alice.merge(&bob_snapshot);

    println!("\n[After cross-merge — both replicas converge]");
    println!("alice: {:?}", alice.to_string());
    println!("bob:   {:?}", bob.to_string());
    assert_eq!(alice.to_string(), bob.to_string());
}
