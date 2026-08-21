//! Persist a list to disk via bincode, then resume editing on the restored copy.
//!
//! Run with: `cargo run --example persist_and_resume`

use abyo_crdt::List;

fn main() {
    let mut alice = List::<char>::new(1);
    for (i, c) in "Hello, world!".chars().enumerate() {
        alice.insert(i, c);
    }

    // Persist (in-memory here, but in real code you'd write to a file or DB).
    let bytes = bincode::serialize(&alice).expect("serialize");
    println!(
        "Serialized {} bytes for {:?}",
        bytes.len(),
        alice.to_string()
    );

    // ... time passes, process restarts ...
    let mut alice_resumed: List<char> = bincode::deserialize(&bytes).expect("deserialize");
    println!("Restored: {:?}", alice_resumed.to_string());
    assert_eq!(alice_resumed.to_string(), alice.to_string());

    // Continue editing — the restored state is fully usable.
    for (i, c) in " Edited!".chars().enumerate() {
        alice_resumed.insert(13 + i, c);
    }
    println!("After more edits: {:?}", alice_resumed.to_string());

    // Sync to a brand-new replica.
    let mut bob = List::<char>::new(2);
    bob.merge(&alice_resumed);
    println!("Bob, after merging:  {:?}", bob.to_string());
    assert_eq!(bob.to_string(), alice_resumed.to_string());
}
