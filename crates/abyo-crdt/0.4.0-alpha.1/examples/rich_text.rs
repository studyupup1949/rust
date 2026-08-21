//! Rich-text demo: collaborative bold/italic with concurrent format ops.

use abyo_crdt::Text;

fn main() {
    let mut alice = Text::new(1);
    alice.insert_str(0, "The quick brown fox");

    // Alice bolds "quick brown" and italicises "fox".
    alice.set_mark(4..15, "bold", true);
    alice.set_mark(16..19, "italic", true);

    print_doc("alice", &alice);

    // Bob comes online and merges Alice's edits.
    let mut bob = Text::new(2);
    bob.merge(&alice);
    print_doc("bob (after merge)", &bob);

    // Concurrent: Alice removes the bold; Bob adds underline to the same range.
    alice.set_mark(4..15, "bold", false);
    bob.set_mark(4..15, "underline", true);

    let alice_clone = alice.clone();
    alice.merge(&bob);
    bob.merge(&alice_clone);

    println!("\nAfter cross-merge:");
    print_doc("alice", &alice);
    print_doc("bob", &bob);
    assert_eq!(render_string(&alice), render_string(&bob));
}

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

fn print_doc(label: &str, t: &Text) {
    println!("  {label}: {}", render_string(t));
}
