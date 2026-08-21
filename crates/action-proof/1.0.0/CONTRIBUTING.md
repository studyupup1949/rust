# Contributing

`action-proof` should stay small and deterministic. Prefer checks that catch release-blocking action wrapper mistakes without requiring network access.

## Development

```powershell
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

## Rule Changes

- Add or update tests for every new check.
- Keep check IDs stable within a major version.
- Use warnings for opinionated hardening recommendations.
- Use failures for GitHub parser/schema issues that can prevent an action from loading.

