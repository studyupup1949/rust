# CLAUDE.md

Read `AGENTS.md` first.

## Quick Reference

- Language: Rust, edition 2024
- Protocol: A2A v1.0, locked to tag `v1.0.0`
- Proto package: `lf.a2a.v1`
- Current implemented surface: `types`, `error`, `jsonrpc`, `server`, `client`, `store`
- Remaining work: docs, examples, release polish
- Zero Clawhive dependency

## Current Design Contract

Use the repo-local design doc:

`docs/proto-first-design.md`

Do not treat the old external iCloud note as the implementation contract.

## Before Every Change

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --no-default-features -- -D warnings
cargo clippy --all-targets --all-features -- -D warnings
cargo test --no-default-features
cargo test --all-features
cargo check --all-features --examples
```

Install hooks once per clone:

```bash
just install-hooks
```

## Critical Rules

1. Follow the tagged proto first, then the spec, then local docs
2. Use `camelCase` JSON field names
3. Use proto enum strings in `SCREAMING_SNAKE_CASE`
4. `Part` is a flat struct, not a tagged enum
5. `Part.raw` is `Vec<u8>` and serializes as base64 JSON
6. JSON-RPC method names are PascalCase, not slash-style
7. `SecurityScheme` is externally tagged and API-key uses `location`, not `in`
8. Use A2A-specific error codes `-32001` through `-32009` when applicable
9. Never use `.unwrap()` outside tests
10. Never add Clawhive-specific code
