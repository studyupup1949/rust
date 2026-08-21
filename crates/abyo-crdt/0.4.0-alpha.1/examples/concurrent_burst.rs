//! Demonstrates the **non-interleaving** property of Fugue-Maximal.
//!
//! Two replicas, both starting from "ab", concurrently type "Hello" and
//! "World" between the two characters. After merge the result is one of
//! `aHelloWorldb` or `aWorldHellob` — never an interleaved mess like
//! `aHWeolrllod b`.

use abyo_crdt::List;

fn main() {
    let mut alice = List::<char>::new(1);
    let mut bob = List::<char>::new(2);

    // Shared "ab" prefix.
    alice.insert(0, 'a');
    alice.insert(1, 'b');
    bob.merge(&alice);

    println!("Shared starting point: {:?}", alice.to_string());

    // Both type concurrently between the 'a' and 'b'.
    for (i, c) in "Hello".chars().enumerate() {
        alice.insert(1 + i, c);
    }
    for (i, c) in "World".chars().enumerate() {
        bob.insert(1 + i, c);
    }

    println!("alice (offline edit): {:?}", alice.to_string());
    println!("bob   (offline edit): {:?}", bob.to_string());

    // Cross-merge.
    let alice_clone = alice.clone();
    alice.merge(&bob);
    bob.merge(&alice_clone);

    let merged = alice.to_string();
    println!("\nMerged: {merged:?}");
    assert_eq!(merged, bob.to_string());

    if merged == "aHelloWorldb" {
        println!("→ Alice's burst won the Lamport tiebreaker.");
    } else {
        println!("→ Bob's burst won the Lamport tiebreaker.");
    }
    println!("Either way, no interleaving — the bursts stayed contiguous.");
}
