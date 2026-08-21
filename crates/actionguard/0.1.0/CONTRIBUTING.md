# Contributing

```bash
git clone https://github.com/thaicn1712/actionguard
cd actionguard
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

All three must pass before a PR is merged — CI runs the same checks. New policies or public API need a test in `tests/integration.rs`.
