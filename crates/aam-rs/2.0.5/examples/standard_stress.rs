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

use aam_rs::aam::AAM;
use aam_rs::builder::AAMBuilder;
use std::time::Instant;

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
    let path = "generated_stress_test.aam";
    let gen_duration = gen_start.elapsed();
    println!("✅ Generation:  {:?}", gen_duration);

    // ── Parsing ───────────────────────────────────────────────────────────────
    let parse_start = Instant::now();
    let aaml = AAM::load_fast(path).expect("Parsing error");
    let parse_duration = parse_start.elapsed();
    println!("✅ Parsing:     {:?}", parse_duration);

    // ── Lookup ────────────────────────────────────────────────────────────────
    let search_key = format!("user_profile_setting_key_{}", count - 1);
    let search_start = Instant::now();
    let result = aaml.get(&search_key);
    let search_duration = search_start.elapsed();

    println!(
        "✅ Lookup:      {:?}  (found: {})",
        search_duration,
        result.unwrap()
    );

    println!("---");
    println!(
        "📊 Total time (excluding console output): {:?}",
        gen_duration + parse_duration + search_duration
    );
}
