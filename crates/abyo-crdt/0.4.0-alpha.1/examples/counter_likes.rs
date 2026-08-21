//! Distributed like-counter — three users incrementing concurrently.

use abyo_crdt::Counter;

fn main() {
    let mut alice = Counter::new(1);
    let mut bob = Counter::new(2);
    let mut carol = Counter::new(3);

    // All three click "like" multiple times while offline.
    alice.increment(3);
    bob.increment(5);
    carol.increment(2);

    // Carol also accidentally clicks "unlike" once.
    carol.decrement(1);

    println!("Local counts before sync:");
    println!("  alice: {}", alice.value());
    println!("  bob:   {}", bob.value());
    println!("  carol: {}", carol.value());

    // Pairwise merges in arbitrary order.
    alice.merge(&bob);
    alice.merge(&carol);
    bob.merge(&carol);
    bob.merge(&alice);
    carol.merge(&alice);

    println!("\nAfter all replicas sync:");
    println!("  alice: {}", alice.value());
    println!("  bob:   {}", bob.value());
    println!("  carol: {}", carol.value());

    println!(
        "\nPN breakdown (alice): +{} / -{}",
        alice.positive_total(),
        alice.negative_total()
    );

    assert_eq!(alice.value(), 9); // 3 + 5 + 2 - 1
    assert_eq!(alice.value(), bob.value());
    assert_eq!(alice.value(), carol.value());
}
