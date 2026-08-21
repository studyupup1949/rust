# Contributing to abyo-crdt

Contributions are welcome. The library is in active alpha; expect rapid
churn on the public API and the internal representation.

## Development workflow

```bash
# Build + test
cargo build
cargo test

# Stress-test the convergence suite
PROPTEST_CASES=10000 cargo test --release --test convergence

# Long-running stress tests (1+ minute)
cargo test --release --test stress -- --ignored

# Benchmarks
cargo bench

# Lint + format
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings

# Documentation
cargo doc --no-deps --open
```

## What we're looking for

- Bug reports backed by a failing property test or a minimal repro
- Performance improvements with `criterion` numbers (before/after)
- Algorithmic correctness reviews — especially anything that exercises the
  Fugue-Maximal anchor rule under unusual concurrent-edit topologies
- Documentation clarifications

## Coding conventions

- `#![forbid(unsafe_code)]` is non-negotiable. Anything that needs `unsafe`
  needs to be discussed in an issue first.
- Keep `cargo clippy --all-features -- -D warnings` clean. We treat clippy
  warnings as errors in CI.
- Public items must have doc comments (CI enforces `missing_docs`).
- Run `cargo fmt --all` before committing.
- Add tests for any behavioral change, ideally as a `proptest` if it's a
  correctness property rather than a single concrete scenario.

## Licensing

By contributing, you agree that your contributions will be dual-licensed
under MIT and Apache-2.0 (matching the project's licensing).
