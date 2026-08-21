# Agent guidelines for adaptive-timeout

## Project overview

This is a small, focused Rust library crate that provides adaptive timeout
computation based on observed latency percentiles. It is inspired by the
adaptive timeout logic in Facebook's LogDevice but is fully generic and
domain-agnostic.

## Repository layout

```
Cargo.toml                      Crate manifest (single dependency: hdrhistogram)
src/
  lib.rs                        Public API re-exports and crate-level docs
  clock.rs                      Instant trait (abstracts over time sources)
  config.rs                     TrackerConfig, TimeoutConfig (compact, Copy types)
  histogram.rs                  SlidingWindowHistogram (internal, not public)
  tracker.rs                    LatencyTracker<D, M> (per-destination latency tracking)
  timeout.rs                    AdaptiveTimeout (timeout selection logic)
benches/
  adaptive_timeout.rs           Criterion microbenchmarks
```

## Key conventions

- **Efficiency is a top priority.** Hot paths must avoid allocations, float
  conversions, and unnecessary `Duration` construction. Config structs use
  `NonZeroU32` milliseconds instead of `Duration`. The histogram maintains an
  incremental merge to avoid rebuilding on every percentile query.
- **All time-dependent methods accept an `Instant` parameter** rather than
  calling `Instant::now()` internally. This makes tests deterministic and
  avoids hidden syscalls on the hot path.
- **Time is abstracted via the `Instant` trait** (`clock::Instant`).
  `std::time::Instant` implements it out of the box. Users can plug in custom
  clocks (simulated time, async-runtime instants, etc.) by implementing the
  trait. `LatencyTracker` defaults the `I` parameter to `std::time::Instant`
  so existing callers don't need to specify it.
- **The crate is single-threaded by design** (`Send` but not `Sync`). No
  internal locking. Users wrap in `Mutex`/`RefCell` if they need shared access.
- **`LatencyTracker` is generic** over destination key (`D`), message key
  (`M`), and instant type (`I`). Do not introduce domain-specific types into
  the core library.
- **`SlidingWindowHistogram` is `pub(crate)`** -- it is an internal
  implementation detail and should not be exposed publicly.

## Working with the code

### Build and test

```sh
cargo test          # 38 tests (unit + doc)
cargo bench         # Criterion benchmarks
```

### Linting and formatting

**Always** run both of these after every change:

```sh
cargo clippy --all-targets    # must produce zero warnings
cargo fmt --all               # must produce no diffs
```

Fix any clippy warnings or formatting issues before considering a change
complete.

### Benchmarks

**Always** run the full benchmark suite after every change:

```sh
cargo bench
```

Compare the results against the performance expectations table below. If any
hot-path benchmark regresses beyond noise (>5%), **stop and discuss the results
with the user before making further changes**. Do not attempt to "fix forward"
a regression without understanding and communicating the cause first.

### Running a specific benchmark

```sh
cargo bench --bench adaptive_timeout -- "select_timeout"
```

### Code style

- Use `#[inline]` on small, hot-path public methods.
- Prefer `_ms` suffixed methods (returning `u64` milliseconds) for internal and
  performance-sensitive paths; provide `Duration`-returning convenience methods
  as thin wrappers.
- Keep doc comments free of LogDevice-specific terminology (store, appender,
  wave, shard, etc.). The README may reference LogDevice as inspiration, but the
  API and docs should be generic.
- Tests use deterministic time injection -- never call `Instant::now()` as the
  sole time source in a test. Use a base `now` and offset from it.

## Performance expectations

Any change to the hot paths (`record_latency_ms`, `percentile_ms`,
`select_timeout_ms`) should be validated against the existing benchmarks.
Regressions beyond noise (>5%) should be investigated before merging.

| Operation | Expected | Notes |
|---|---|---|
| `record_latency_ms` | < 100 ns | Steady state, no rotation |
| `percentile_ms` | < 100 ns | Incremental merge, no alloc |
| `select_timeout_ms` (1 dest) | < 100 ns | Single HashMap lookup + integer math |
| `exponential_backoff_ms` | < 5 ns | Pure arithmetic, no tracker |
