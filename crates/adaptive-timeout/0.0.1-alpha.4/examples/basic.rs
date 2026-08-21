//! Basic usage of the adaptive-timeout crate.
//!
//! Simulates a service sending requests to multiple backend nodes, tracking
//! latencies per destination, and computing adaptive timeouts.
//!
//! Run with: `cargo run --example basic`

use std::time::{Duration, Instant};

use adaptive_timeout::{
    AdaptiveTimeout, BackoffInterval, LatencyTracker, MillisNonZero, TimeoutConfig, TrackerConfig,
};

const NODE_A: u32 = 1;
const NODE_B: u32 = 2;
const NODE_C: u32 = 3;

fn main() {
    let now = Instant::now();

    // -- Configure the latency tracker (one per service) --
    let tracker_config = TrackerConfig {
        window_ms: MillisNonZero::new(10_000).unwrap(), // 10s sliding window
        min_samples: 10, // need >=10 samples before estimates are valid
        ..TrackerConfig::default()
    };
    let mut get_tracker = LatencyTracker::<u32, Instant>::new(tracker_config);
    let mut put_tracker = LatencyTracker::<u32, Instant>::new(tracker_config);

    // -- Configure the adaptive timeout --
    // Parse timeout range from a string: "5ms..30s" sets floor=5ms, ceiling=30s.
    let backoff: BackoffInterval = "5ms..30s".parse().expect("valid timeout range");
    println!("Parsed timeout range: {backoff}\n");

    // Convert to TimeoutConfig (quantile and safety_factor get defaults).
    // Override safety_factor if needed:
    let timeout_config = TimeoutConfig {
        safety_factor: 2.0,
        ..backoff.into()
    };
    let timeout = AdaptiveTimeout::new(timeout_config);

    // -----------------------------------------------------------------------
    // Phase 1: No data — falls back to exponential backoff
    // -----------------------------------------------------------------------
    println!("=== Phase 1: No latency data (exponential backoff) ===\n");
    for attempt in 1..=4 {
        let t = timeout.select_timeout(&mut get_tracker, &[NODE_A], attempt, now);
        println!("  attempt {attempt}: timeout = {t:?}");
    }

    // -----------------------------------------------------------------------
    // Phase 2: Record latency observations
    // -----------------------------------------------------------------------
    println!("\n=== Phase 2: Recording latency samples ===\n");

    // Node A: fast (~10ms), Node B: moderate (~50ms), Node C: slow (~200ms)
    let latencies = [(NODE_A, 10u64), (NODE_B, 50), (NODE_C, 200)];

    for &(node, base_latency_ms) in &latencies {
        for i in 0..50 {
            let jitter = (i % 5) as u64 * base_latency_ms / 20;
            let latency_ms = base_latency_ms + jitter;
            get_tracker.record_latency_ms(&node, latency_ms, now);
        }
    }

    println!("  Recorded 50 GET samples per node (150 total)");

    // -----------------------------------------------------------------------
    // Phase 3: Adaptive timeouts based on observed latencies
    // -----------------------------------------------------------------------
    println!("\n=== Phase 3: Adaptive timeouts (with data) ===\n");

    for &(node, label) in &[
        (NODE_A, "A (fast)"),
        (NODE_B, "B (moderate)"),
        (NODE_C, "C (slow)"),
    ] {
        let t = timeout.select_timeout(&mut get_tracker, &[node], 1, now);
        println!("  Node {label}: timeout = {t:?}");
    }

    // Multi-destination: takes the max across all destinations
    println!();
    let all_nodes = [NODE_A, NODE_B, NODE_C];
    let t = timeout.select_timeout(&mut get_tracker, &all_nodes, 1, now);
    println!("  All nodes [A, B, C]: timeout = {t:?}  (max across destinations)");

    // -----------------------------------------------------------------------
    // Phase 4: Retries with adaptive backoff
    // -----------------------------------------------------------------------
    println!("\n=== Phase 4: Retries with adaptive backoff ===\n");
    for attempt in 1..=5 {
        let t = timeout.select_timeout(&mut get_tracker, &[NODE_C], attempt, now);
        println!("  Node C, attempt {attempt}: timeout = {t:?}");
    }

    // -----------------------------------------------------------------------
    // Phase 5: Per-service tracking (separate trackers)
    // -----------------------------------------------------------------------
    println!("\n=== Phase 5: Per-service tracking ===\n");

    // PUT latencies are higher than GET
    for _ in 0..50 {
        put_tracker.record_latency(&NODE_A, Duration::from_millis(80), now);
    }

    let t_get = timeout.select_timeout(&mut get_tracker, &[NODE_A], 1, now);
    let t_put = timeout.select_timeout(&mut put_tracker, &[NODE_A], 1, now);
    println!("  Node A, GET tracker:  timeout = {t_get:?}");
    println!("  Node A, PUT tracker:  timeout = {t_put:?}");

    // -----------------------------------------------------------------------
    // Phase 6: Pure exponential backoff (no tracker needed)
    // -----------------------------------------------------------------------
    println!("\n=== Phase 6: Pure exponential backoff ===\n");
    for attempt in 1..=6 {
        let t = timeout.exponential_backoff(attempt);
        println!("  attempt {attempt}: timeout = {t:?}");
    }
}
