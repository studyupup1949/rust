//! Quill / Yjs Delta interop demo.
//!
//! Shows how `Text::to_delta()` produces a JSON-serializable structure
//! identical to what `Y.Text.toDelta()` and Quill emit, and how
//! `Text::from_delta()` reconstructs a `Text` from the same.

use abyo_crdt::Text;

fn main() {
    // Build a rich-text document.
    let mut doc = Text::new(1);
    doc.insert_str(0, "Hello world");
    doc.set_mark(0..5, "bold", true);
    doc.set_mark(6..11, "italic", true);
    doc.set_value_mark(6..11, "color", Some("#ff0000"));

    // Export as Delta + JSON.
    let delta = doc.to_delta();
    let json = serde_json::to_string_pretty(&delta).expect("serialize");
    println!("Delta JSON (Quill / Yjs Y.Text format):\n{json}\n");

    // Round-trip: import the JSON back into a fresh Text.
    let parsed: Vec<abyo_crdt::DeltaOp> = serde_json::from_str(&json).expect("parse");
    let restored = Text::from_delta(2, &parsed);
    println!("Restored text: {:?}", restored.as_string());
    println!(
        "Restored runs: {}",
        restored
            .to_delta()
            .iter()
            .map(|op| {
                let attrs: Vec<&str> = op.attributes.keys().map(String::as_str).collect();
                if attrs.is_empty() {
                    format!("{:?}", op.insert)
                } else {
                    format!("{:?}[{}]", op.insert, attrs.join(","))
                }
            })
            .collect::<Vec<_>>()
            .join(" + ")
    );
    assert_eq!(doc.to_delta(), restored.to_delta());
}
