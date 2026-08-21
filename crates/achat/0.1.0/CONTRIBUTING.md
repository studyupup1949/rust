# Contributing to achat

Thank you for your interest in contributing.

## Getting Started

```sh
git clone https://github.com/quadra-a/achat.git
cd achat
cargo build
cargo test              # 28 tests (unit + integration)
cargo clippy            # zero warnings required
cargo fmt --check       # formatting must pass
```

## Before Submitting

- Open an issue before starting large changes
- Run `cargo test` and `cargo clippy` — CI will reject warnings
- Run `cargo fmt` before committing
- Keep commits focused: one logical change per PR

## Code Standards

- `clippy::pedantic` is enforced
- `unsafe_code` is denied
- Error handling: use `anyhow::Result` with `.context()`
- Follow Rust naming conventions ([RFC 430](https://github.com/rust-lang/rfcs/blob/master/text/0430-finalizing-naming-conventions.md))

## Reporting Bugs

Open a GitHub issue with:

1. What you did (command / steps to reproduce)
2. What you expected
3. What actually happened
4. `achat --version` output and OS
