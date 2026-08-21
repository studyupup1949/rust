# Contributing to AAM CLI

Thanks for contributing! Here's how to get started.

## Setup

```bash
git clone https://github.com/ininids/aam-cli
cd aam-cli
cargo build
```

## Development workflow

- `cargo build` — compile
- `cargo test` — run tests
- `cargo fmt --check` — check formatting
- `cargo clippy -- -D warnings` — lint
- `make all` — runs fmt, lint, test, and check together

## Before submitting

1. Format your code: `cargo fmt`
2. Fix clippy warnings: `cargo clippy -- -D warnings`
3. Run tests: `cargo test`
4. Update `CHANGELOG.md` if your change is user-facing
5. Keep the `README.md` and `src/main.rs` help text aligned with any command or flag changes

## Pull requests

- Target the `main` branch
- Use a clear, concise title
- Link any related issues
- Follow the [pull request template](pull_request_template.md)

## Code style

- Mirror the conventions in `src/main.rs`, `src/tui.rs`, and `src/lsp.rs`
- No unnecessary comments — let the code speak
- Use `anyhow` for error propagation
- Keep dependencies minimal; discuss new dependencies in an issue first

## Issues

Use the issue templates for bugs and feature requests. Search existing issues before opening a new one.
