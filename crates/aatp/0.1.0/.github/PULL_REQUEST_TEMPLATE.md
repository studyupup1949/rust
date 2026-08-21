## What & why

Describe the change and the motivation.

## Checklist

- [ ] `cargo test --all` passes
- [ ] `cargo fmt --all --check` clean
- [ ] `cargo clippy --all-targets -- -D warnings` clean
- [ ] Builds `no_std`: `cargo build --lib --target thumbv7em-none-eabihf`
- [ ] No new dependencies (runtime or dev); no `unsafe`
- [ ] New/changed behaviour is tested (branches pinned; decoders fuzz-covered; constants
      pinned by known-answer/snapshot/digest tests)
- [ ] `CHANGELOG.md` updated under `[Unreleased]`

## Notes

Anything reviewers should pay special attention to.
