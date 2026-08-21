# Contributing to Abaco

## Development Workflow

1. Fork the repository
2. Create a branch from `main`
3. Make your changes and ensure `make check` passes
4. Open a pull request to `main`

## Prerequisites

- Rust stable toolchain (MSRV: 1.89)
- rustfmt, clippy (included via `rust-toolchain.toml`)
- Optional: cargo-audit, cargo-deny, cargo-tarpaulin

## Makefile Targets

| Target | Description |
|--------|-------------|
| `make check` | Run fmt + clippy + test + audit |
| `make fmt` | Check formatting |
| `make clippy` | Lint with zero warnings |
| `make test` | Run core tests |
| `make test-all` | Run tests with all features |
| `make bench` | Run criterion benchmarks |
| `make coverage` | Generate coverage report |
| `make doc` | Build docs (warnings as errors) |

## Adding a New Module

1. Create `src/module_name.rs` with doc comment
2. Add `pub mod module_name;` to `src/lib.rs`
3. Re-export key types from `lib.rs`
4. Add unit tests in `#[cfg(test)] mod tests`
5. Feature-gate external dependencies in `Cargo.toml`
6. Update README module table

## Code Style

- `cargo fmt` — mandatory
- `cargo clippy -- -D warnings` — zero warnings
- Doc comments on all public items
- `#[non_exhaustive]` on public enums
- No `unsafe` code
- No `println!` — use `tracing`

## Testing

- Unit tests colocated in modules (`#[cfg(test)] mod tests`)
- Feature-gated tests with `#[cfg(feature = "...")]`
- Target: 90%+ line coverage
- All features must have tests before merge

## License

By contributing, you agree that your contributions will be licensed under AGPL-3.0-only.
