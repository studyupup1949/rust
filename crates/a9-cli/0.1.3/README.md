# a9 CLI

In-house engineering CLI for Ars Vivendi.

Provides commands for managing internal tools and development operations.

## Usage

```bash
a9 install lint --tag v0.1.19
a9 install lint --force
a9 --json install lint
```

## Development

```bash
# Enable git hooks
git config core.hooksPath .githooks

# Run checks
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo a9-lint --check
```
