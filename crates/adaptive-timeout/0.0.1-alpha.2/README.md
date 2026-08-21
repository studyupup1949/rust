# adaptive-timeout

[![Crates.io](https://img.shields.io/crates/v/adaptive-timeout.svg)](https://crates.io/crates/adaptive-timeout)
[![Documentation](https://docs.rs/adaptive-timeout/badge.svg)](https://docs.rs/adaptive-timeout)
[![CI](https://github.com/AhmedSoliman/adaptive-timeout/workflows/CI/badge.svg)](https://github.com/AhmedSoliman/adaptive-timeout/actions)
[![License](https://img.shields.io/badge/license-Apache%202.0%20OR%20MIT-blue.svg)](http://www.apache.org/licenses/LICENSE-2.0)

Adaptive timeout computation based on observed latency percentiles.

This crate provides a mechanism for computing request timeouts that automatically
adapt to observed network conditions. The approach is deeply inspired by the
adaptive timeout logic in Facebook's
[LogDevice](https://github.com/facebookincubator/LogDevice), generalized into a
reusable, domain-agnostic Rust library.

## The problem

Fixed timeouts are fragile. Set them too low and you get false positives during
transient slowdowns; set them too high and you waste time waiting for genuinely
failed requests. Exponential backoff helps with retries but has no awareness of
actual network conditions.

## How it works

1. A **`LatencyTracker`** records round-trip times for requests, maintaining
   per-destination sliding-window histograms of recent latency samples. The
   tracker is generic over destination and message key types, so it works with
   any transport or RPC system.

2. An **`AdaptiveTimeout`** queries the tracker for a high percentile (e.g.
   P99.99) of recent latencies, applies a configurable safety factor, an
   exponential backoff multiplier based on the attempt number, and clamps the
   result between a floor and ceiling.

3. When insufficient data is available (cold start or sparse traffic), the
   system falls back gracefully to pure exponential backoff.

### Timeout selection algorithm

For each destination in a request's target set:

```
timeout = clamp(safety_factor * percentile_estimate * 2^(attempt-1), min, max)
```

The final timeout is the **maximum** across all destinations, ensuring it is
long enough for the slowest expected peer.

## Quick start

```rust
use std::time::{Duration, Instant};
use adaptive_timeout::{AdaptiveTimeout, LatencyTracker, TimeoutConfig, TrackerConfig};

let now = Instant::now();

// Create a tracker and timeout selector with default configs.
let mut tracker = LatencyTracker::<u32>::default();
let timeout = AdaptiveTimeout::default();

// Initially there's no data -- we get exponential backoff from min_timeout.
let t = timeout.select_timeout(&mut tracker, &[1u32], 1, now);
assert_eq!(t, Duration::from_millis(10));

// Record some latency observations (e.g. from real RPCs).
for i in 0..100u64 {
    tracker.record_send(1u32, i, now);
    let reply_time = now + Duration::from_millis(50);
    tracker.record_reply(&i, reply_time);
}

// Now the timeout adapts based on observed latencies.
let t = timeout.select_timeout(&mut tracker, &[1u32], 1, now);
assert!(t >= Duration::from_millis(50));
```

## Custom clocks

All time-dependent types and methods are generic over the `Instant` trait. You
can supply your own implementation for simulated time, async runtimes, or other
custom clocks:

```rust
use std::time::Duration;
use adaptive_timeout::Instant;

#[derive(Clone, Copy)]
struct FakeInstant(u64); // nanoseconds

impl Instant for FakeInstant {
    fn duration_since(&self, earlier: Self) -> Duration {
        Duration::from_nanos(self.0.saturating_sub(earlier.0))
    }
    fn add_duration(&self, duration: Duration) -> Self {
        FakeInstant(self.0 + duration.as_nanos() as u64)
    }
}

// Use it with LatencyTracker:
let mut tracker = adaptive_timeout::LatencyTracker::<u32, u64, FakeInstant>::default();
tracker.record_latency_ms(&1, 50, FakeInstant(1_000_000));
```

When using `std::time::Instant` (the default), you don't need to specify the
third type parameter at all.

## Architecture

```
src/
  lib.rs          Public re-exports, crate-level docs
  clock.rs        Instant trait (abstracts over time sources)
  config.rs       TrackerConfig, TimeoutConfig (compact, Copy types)
  histogram.rs    SlidingWindowHistogram (time-bucketed ring of HdrHistograms)
  tracker.rs      LatencyTracker<D, M> (per-destination latency tracking)
  timeout.rs      AdaptiveTimeout (percentile-based timeout selection)
```

### Key design decisions

| Aspect | Choice | Rationale |
|---|---|---|
| Histogram backend | `hdrhistogram` crate | Proven, widely used, handles wide dynamic ranges natively without log-space transforms |
| Sliding window | Ring of N sub-window histograms with incremental merge | Avoids rebuilding a merged histogram on every percentile query; rotation subtracts expired buckets |
| Duration representation | `NonZeroU32` milliseconds in config structs | 4 bytes vs 16 for `Duration`; `TimeoutConfig` fits in 24 bytes; hot-path arithmetic stays in integer domain |
| In-flight tracking | `HashMap` with monotonic sequence counter | O(1) for `record_reply` (the common path); eviction scan only runs when at capacity (rare) |
| Thread safety | Single-threaded (`Send` but not `Sync`) | No synchronization overhead; caller wraps in `Mutex`/`RefCell` if sharing is needed |
| Time abstraction | `Instant` trait (`clock::Instant`), impl'd for `std::time::Instant` | Pluggable clocks for simulated time, async runtimes, etc. |
| Time injection | All methods accept an `Instant` parameter | Deterministic tests without mocking; zero overhead in production |
| Generics | `LatencyTracker<D, M, I>` over destination, message key, and instant types | Works with any transport layer and clock without coupling |

## Configuration

### `TrackerConfig` (defaults)

| Field | Default | Description |
|---|---|---|
| `window_ms` | 10,000 (10s) | Sliding window duration |
| `num_sub_windows` | 10 | Granularity of window expiry |
| `min_samples` | 30 | Minimum samples before estimates are trusted |
| `max_in_flight` | 10,000 | Bounded in-flight request tracking |
| `significant_value_digits` | 2 | HdrHistogram precision (~1%) |
| `max_trackable_latency_ms` | 60,000 (60s) | Upper clamp for recorded latencies |

### `TimeoutConfig` (defaults)

| Field | Default | Description |
|---|---|---|
| `min_timeout_ms` | 10ms | Floor -- timeout never goes below this |
| `max_timeout_ms` | 60,000ms | Ceiling -- timeout never exceeds this |
| `percentile` | 99.99 | Percentile of the latency distribution to use |
| `safety_factor` | 2.0 | Multiplier on the percentile estimate |

## Benchmarks

Run with `cargo bench`:

```
record_latency_ms           ~80 ns/op    (steady state, no rotation)
send_reply_cycle            ~140 ns/op   (record_send + record_reply pair)
percentile_query             ~30-73 ns/op (scales with histogram density)
select_timeout (1 dest)      ~83 ns/op
select_timeout (10 dests)   ~770 ns/op
exponential_backoff_only     ~1.5 ns/op  (no tracker interaction)
window_rotation              ~2.8 us/op  (1 sub-window rotation + record)
in_flight_eviction (10k)     ~10 us/op   (worst case, at capacity)
```
## Minimum Supported Rust Version (MSRV)

Requires Rust 1.92.0 or later.

## License

MIT
