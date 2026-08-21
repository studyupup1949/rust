# Contributing to a2a-rust

Thank you for your interest in contributing to a2a-rust! This document provides guidelines for contributors.

## Code of Conduct

Be respectful, inclusive, and constructive. We're all here to build a great A2A SDK together.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/a2a-rust.git`
3. Add upstream remote: `git remote add upstream https://github.com/longzhi/a2a-rust.git`
4. Create a branch: `git checkout -b feature/your-feature-name`

## Development Setup

### Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- Git

Install local git hooks after cloning:

```bash
just install-hooks
```

### Build & Test

```bash
cargo build
cargo test
cargo test --no-default-features
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --no-default-features -- -D warnings
cargo fmt --all
cargo check --all-features --examples
```

These checks must pass before submitting a PR. CI treats all warnings as errors.

### Feature Flags

Test with different feature combinations:

```bash
cargo test --all-features                          # Full
cargo test --no-default-features                   # Types only
cargo test --no-default-features --features server # Server only
cargo test --no-default-features --features client # Client only
```

## Making Changes

1. **Check existing issues** — see if someone is already working on it
2. **Create an issue first** — for significant changes, discuss before coding
3. **Keep changes focused** — one feature or fix per PR
4. **Write tests** — especially serde round-trip tests for type changes
5. **Update documentation** — rustdoc comments for all public items

### Protocol Compliance

This crate implements A2A Protocol v1.0 RC. When making changes:

- **Check the proto definition**: https://github.com/a2aproject/A2A/blob/v1.0.0-rc/specification/a2a.proto
- **Check the spec**: https://a2a-protocol.org/latest/specification/
- **JSON field names** must be `camelCase`
- **Enum values** must be `SCREAMING_SNAKE_CASE`
- **Type structure** must match proto3 definitions exactly

### Code Style

See [AGENTS.md](AGENTS.md) for detailed code conventions. Key points:

- `#[serde(rename_all = "camelCase")]` on all types
- `thiserror` for error types (this is a library crate)
- `tracing` for logging (never `println!`)
- No `.unwrap()` outside tests
- Derive order: `Debug, Clone, Serialize, Deserialize`

## Commit Convention

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>
```

### Types

| Type | Description |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `refactor` | Code change that neither fixes a bug nor adds a feature |
| `test` | Adding or updating tests |
| `chore` | Maintenance tasks |

### Examples

```
feat(types): add AgentInterface struct for v1.0 RC
fix(server): handle missing Content-Type in JSON-RPC requests
test(serde): add round-trip tests for SecurityScheme variants
docs(readme): add SSE streaming example
```

Local hooks enforce this commit message format and run checks on `git commit` and `git push`.

## Pull Request Process

1. **Update your branch** with the latest upstream changes:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Ensure all checks pass**:
   ```bash
   cargo test --all-features
   cargo clippy --all-targets --all-features -- -D warnings
   cargo fmt --all -- --check
   cargo doc --no-deps --all-features
   ```

3. **Create the PR** with a clear title and description

4. **Fill out the PR template** completely

5. **Respond to review feedback** promptly

## Testing Guidelines

### Type Serialization Tests (Most Important)

Every type must have round-trip serialization tests using A2A spec JSON examples:

```rust
#[test]
fn agent_card_round_trip() {
    let json = r#"{"name":"Test","description":"A test agent",...}"#;
    let card: AgentCard = serde_json::from_str(json).unwrap();
    let reserialized = serde_json::to_string(&card).unwrap();
    let card2: AgentCard = serde_json::from_str(&reserialized).unwrap();
    assert_eq!(card.name, card2.name);
}
```

### Server Integration Tests

Use `tower::ServiceExt` to test handlers without starting a real HTTP server:

```rust
let app = a2a_rust::server::router(MockHandler::new());
let response = app.oneshot(request).await.unwrap();
assert_eq!(response.status(), StatusCode::OK);
```

### Client Integration Tests

Use `wiremock` to mock A2A servers:

```rust
let mock_server = MockServer::start().await;
Mock::given(method("POST")).and(path("/rpc"))
    .respond_with(ResponseTemplate::new(200).set_body_json(/* ... */))
    .mount(&mock_server).await;
```

There are two client test styles in this repo:

- `tests/client_integration.rs` for end-to-end behavior against the local axum server
- `tests/client_wiremock.rs` for client-only transport and wire-shape tests

## Questions?

- Open a [Discussion](https://github.com/longzhi/a2a-rust/discussions)
- Check existing [Issues](https://github.com/longzhi/a2a-rust/issues)
- Read the [A2A Protocol Spec](https://a2a-protocol.org/latest/specification/)
