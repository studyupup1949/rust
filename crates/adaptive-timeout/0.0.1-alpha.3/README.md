# adaptive-timeout

[![Crates.io](https://img.shields.io/crates/v/adaptive-timeout.svg)](https://crates.io/crates/adaptive-timeout)
[![Documentation](https://docs.rs/adaptive-timeout/badge.svg)](https://docs.rs/adaptive-timeout)
[![CI](https://github.com/AhmedSoliman/adaptive-timeout/workflows/CI/badge.svg)](https://github.com/AhmedSoliman/adaptive-timeout/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)

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
   tracker is generic over destination type, so it works with any transport or
   RPC system.

2. An **`AdaptiveTimeout`** queries the tracker for a high quantile (e.g.
   P99.99) of recent latencies, applies a configurable safety factor, an
   exponential backoff multiplier based on the attempt number, and clamps the
   result between a floor and ceiling.

3. When insufficient data is available (cold start or sparse traffic), the
   system falls back gracefully to pure exponential backoff.

### Timeout selection algorithm

For each destination in a request's target set:

```
timeout = clamp(safety_factor * quantile_estimate * 2^(attempt-1), min, max)
```

The final timeout is the **maximum** across all destinations, ensuring it is
long enough for the slowest expected peer.

## Quick start

```rust
use std::time::{Duration, Instant};
use adaptive_timeout::{AdaptiveTimeout, LatencyTracker};

let now = Instant::now();

// Create a tracker and timeout selector with default configs.
let mut tracker = LatencyTracker::<u32, Instant>::default();
let timeout = AdaptiveTimeout::default();

// Initially there's no data -- we get exponential backoff from min_timeout.
let t = timeout.select_timeout(&mut tracker, &[1u32], 1, now);
assert_eq!(t, Duration::from_millis(250));

// Record some latency observations (e.g. from real RPCs).
for _ in 0..100 {
    tracker.record_latency(&1u32, Duration::from_millis(50), now);
}

// Now the timeout adapts based on observed latencies.
let t = timeout.select_timeout(&mut tracker, &[1u32], 1, now);
assert!(t >= Duration::from_millis(50));
```

## Recording latency

Three methods are available depending on what information you have at hand:

```rust
// From a Duration (e.g. after timing an RPC with std::time):
tracker.record_latency(&dest, Duration::from_millis(42), now);

// From raw milliseconds (fastest path — no Duration conversion):
tracker.record_latency_ms(&dest, 42, now);

// From two instants (computes the difference for you):
let latency = tracker.record_latency_from(&dest, send_time, now);
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
let mut tracker = adaptive_timeout::LatencyTracker::<u32, FakeInstant>::default();
tracker.record_latency_ms(&1, 50, FakeInstant(1_000_000));
```

When using `std::time::Instant` (the default), you don't need to specify the
clock type parameter at all.

A `tokio::time::Instant` implementation is also provided behind the optional
`tokio` feature.

## Architecture

```
src/
  lib.rs              Public re-exports, crate-level docs
  clock.rs            Instant trait (abstracts over time sources)
  config.rs           TrackerConfig, TimeoutConfig (compact, Copy types)
  histogram.rs        SlidingWindowHistogram (time-bucketed ring of HdrHistograms)
  parse.rs            BackoffInterval, ParseError (duration-range string parsing)
  sync_tracker.rs     SyncLatencyTracker (Send + Sync, feature = "sync")
  tracker.rs          LatencyTracker<D, I, H, N> (per-destination latency tracking)
  timeout.rs          AdaptiveTimeout (percentile-based timeout selection)
```

### Key design decisions

| Aspect | Choice | Rationale |
|---|---|---|
| Histogram backend | `hdrhistogram` crate | Proven, widely used, handles wide dynamic ranges natively without log-space transforms |
| Sliding window | Ring of N sub-window histograms with incremental merge | Avoids rebuilding a merged histogram on every quantile query; rotation subtracts expired buckets |
| Duration representation | `NonZeroU32` milliseconds in config structs | 4 bytes vs 16 for `Duration`; `TimeoutConfig` fits in 24 bytes; hot-path arithmetic stays in integer domain |
| Thread safety | Single-threaded (`Send` but not `Sync`) | No synchronization overhead; caller wraps in `Mutex`/`RefCell` if sharing is needed. Optional `sync` feature provides `SyncLatencyTracker` for lock-free concurrent access. |
| Time abstraction | `Instant` trait (`clock::Instant`), impl'd for `std::time::Instant` | Pluggable clocks for simulated time, async runtimes, etc. |
| Time injection | All methods accept an `Instant` parameter | Deterministic tests without mocking; zero overhead in production |
| Generics | `LatencyTracker<D, I, H, N>` over destination, instant, hasher, and sub-window count | Works with any transport layer and clock without coupling |

## Configuration

### `TrackerConfig` (defaults)

| Field | Default | Description |
|---|---|---|
| `window_ms` | 60,000 (60s) | Total sliding window duration |
| `min_samples` | 3 | Minimum samples before quantile estimates are trusted |
| `max_trackable_latency_ms` | 60,000 (60s) | Upper clamp for recorded latencies |

The number of sub-windows (`N`) is a const generic on `LatencyTracker` with a
default of `DEFAULT_SUB_WINDOWS` (10). With the default `window_ms` of 60s this
gives 6-second sub-windows: old data is shed in 10% increments every 6 seconds.

### `TimeoutConfig` (defaults)

| Field | Default | Description |
|---|---|---|
| `backoff` | `250ms..1min` | Floor and ceiling as a `BackoffInterval` |
| `quantile` | 0.9999 | Quantile of the latency distribution to use (e.g. 0.9999 = P99.99) |
| `safety_factor` | 2.0 | Multiplier on the quantile estimate |

### `BackoffInterval`

`BackoffInterval` holds the `min_ms` and `max_ms` bounds and can be constructed
by parsing a human-readable duration-range string:

```rust
use adaptive_timeout::BackoffInterval;

let b: BackoffInterval = "250ms..1m".parse().unwrap();
assert_eq!(b.min_ms.get(), 250);
assert_eq!(b.max_ms.get(), 60_000);
```

Supported units are compatible with [jiff's friendly duration format](https://docs.rs/jiff/latest/jiff/fmt/friendly/index.html):
`ms`, `s`, `m`, `h`, `d` (and verbose forms like `seconds`, `minutes`, etc.).
Fractional values (`0.5s`) and spaces between number and unit (`10 ms`) are
accepted.

## Optional features

| Feature | Default | Description |
|---|---|---|
| `sync` | off | Enables `SyncLatencyTracker`, a `Send + Sync` concurrent tracker backed by `DashMap` |
| `tokio` | off | Implements `Instant` for `tokio::time::Instant` |

### Thread-safe tracker (`sync` feature)

When the `sync` feature is enabled, `SyncLatencyTracker` is available. It has
the same API as `LatencyTracker` but takes `&self` instead of `&mut self`,
making it safe to share across threads without an external `Mutex`:

```rust
// Cargo.toml: adaptive-timeout = { features = ["sync"] }
use adaptive_timeout::SyncLatencyTracker;

let tracker = std::sync::Arc::new(SyncLatencyTracker::<u32>::default());
// Can be cloned into multiple threads and called concurrently.
tracker.record_latency_ms(&1u32, 50, now);
```

`AdaptiveTimeout` gains `select_timeout_sync` and `select_timeout_sync_ms`
companion methods that accept `&SyncLatencyTracker` instead of
`&mut LatencyTracker`.

## Benchmarks

Run with `cargo bench`:

```
record_latency_ms (steady state, no rotation)     < 100 ns
quantile_query                                    < 100 ns
select_timeout (1 dest, adaptive path)            < 100 ns
exponential_backoff_only (no tracker)             < 5 ns
window_rotation (1 sub-window rotated + record)   ~1-3 µs
```

## Minimum Supported Rust Version (MSRV)

Requires Rust 1.92.0 or later.

## License

MIT
