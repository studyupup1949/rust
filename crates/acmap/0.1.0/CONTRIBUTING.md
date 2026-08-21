# Contributing

## Development

1. Fork and clone the repository.
2. Create a branch from `master`.
3. Make your change with tests.
4. Run checks locally:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test -q
```

5. Open a pull request with:
- A clear problem statement
- A concise solution summary
- Any benchmark or behavior changes

## Commit Guidelines

- Keep commits focused and atomic.
- Use descriptive commit messages.
- Avoid unrelated formatting-only changes.
