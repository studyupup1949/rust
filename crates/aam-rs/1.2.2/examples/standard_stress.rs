//! Example: stress test — parsing 30 000 000 key-value pairs
//!
//! Measures:
//! - Time to generate a large in-memory AAML document.
//! - Time to parse that document.
//! - Time to look up the last inserted key.
//!
//! Run with:
//! ```sh
//! cargo run --release --example standard_stress
//! ```

use std::time::Instant;
use aam_rs::aaml::AAML;
use aam_rs::builder::AAMBuilder;

fn main() {
    let count = 30_000_000;
    println!("🚀 Starting stress test with {} key-value pairs...", count);

    // ── Generation ────────────────────────────────────────────────────────────
    let gen_start = Instant::now();
    let mut builder = AAMBuilder::with_capacity(count * 40);
    for i in 0..count {
        let key = format!("user_profile_setting_key_{}", i);
        let val = format!("value_string_number_{}", i);
        builder.add_line(&key, &val);
    }
    let content = builder.build();
    let gen_duration = gen_start.elapsed();
    println!("✅ Generation:  {:?}", gen_duration);

    // ── Parsing ───────────────────────────────────────────────────────────────
    let parse_start = Instant::now();
    let aaml = AAML::parse(&content).expect("Parsing error");
    let parse_duration = parse_start.elapsed();
    println!("✅ Parsing:     {:?}", parse_duration);

    // ── Lookup ────────────────────────────────────────────────────────────────
    let search_key = format!("user_profile_setting_key_{}", count - 1);
    let search_start = Instant::now();
    let result = aaml.find_obj(&search_key);
    let search_duration = search_start.elapsed();

    match result {
        Some(v) => println!("✅ Lookup:      {:?}  (found: {})", search_duration, v.as_str()),
        None    => println!("❌ Lookup:      {:?}  (not found)", search_duration),
    }

    println!("---");
    println!(
        "📊 Total time (excluding console output): {:?}",
        gen_duration + parse_duration + search_duration
    );
    println!(
        "📦 Buffer size: {:.2} MB",
        content.len() as f64 / 1_048_576.0
    );
}