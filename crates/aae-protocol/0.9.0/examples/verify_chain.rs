//! Verify a JSONL audit chain, value-based (C-3-safe).
//!
//! Usage: cargo run --example verify_chain -- CHAIN.jsonl

use aae::hashchain::verify_chain_values;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: verify_chain CHAIN.jsonl");
    let raw = std::fs::read_to_string(&path).expect("read chain file");
    let events: Vec<serde_json::Value> = raw
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("parse event"))
        .collect();
    match verify_chain_values(&events) {
        Ok(tip) => println!("rust verified {} events; tip {}", events.len(), tip),
        Err(e) => {
            eprintln!("rust verification FAILED: {e}");
            std::process::exit(1);
        }
    }
}
