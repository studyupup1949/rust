# AGENTS.md

This file provides guidance to AI coding agents working in this repository.

## Project Overview

`a2a-rust` is a generic Rust SDK for A2A (Agent-to-Agent) Protocol v1.0.

Current implementation status:

- implemented now: `types`, `error`, `jsonrpc`, `server`, `client`, and `store`
- remaining work: docs, examples, and release polish

This crate has zero Clawhive-specific logic. Keep it generic and protocol-focused.

## Source of Truth

Protocol lock:

- tag: `v1.0.0`
- commit: `1736957`
- proto package: `lf.a2a.v1`

Normative source precedence:

1. tagged proto
2. current spec prose
3. repository-local design docs

Use the repo-local design contract at [docs/proto-first-design.md](docs/proto-first-design.md).

Do not treat the old external planning note as the implementation contract.

## Build, Test, and Lint

Use these commands before considering a change done:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --no-default-features -- -D warnings
cargo check --all-features
cargo test --no-default-features
cargo test --all-features
```

Useful variants:

```bash
# Types-only compile
cargo check --no-default-features

# Feature combinations
cargo check --no-default-features --features server
cargo check --no-default-features --features client
cargo check --no-default-features --features server,client
```

CI now checks:

- all-features build, test, clippy, docs
- no-default-features build, test, clippy, docs
- feature-combination compile matrix

## Git Hooks

Install local hooks with:

```bash
just install-hooks
```

Current hooks:

- `pre-commit`: format check, minimal clippy, all-features compile check
- `commit-msg`: Conventional Commits enforcement
- `pre-push`: no-default-features tests and all-features tests

## Current Project Structure

```text
src/
├── lib.rs
├── error.rs
├── jsonrpc.rs
├── store.rs
├── types/
├── server/
└── client/
```

## Key Source Files

- `src/error.rs` - `A2AError` and protocol/HTTP error mapping
- `src/jsonrpc.rs` - JSON-RPC 2.0 envelopes, method constants, error-code constants
- `src/store.rs` - `TaskStore` and `InMemoryTaskStore`
- `src/types/agent_id.rs` - validated helper type for agent naming conventions
- `src/types/agent_card.rs` - Agent card and capability model
- `src/types/auth.rs` - `TASK_STATE_AUTH_REQUIRED` metadata helpers
- `src/types/message.rs` - `Message`, `Part`, `Artifact`, `Role`
- `src/types/task.rs` - `Task`, `TaskStatus`, `TaskState`
- `src/types/security.rs` - security schemes and OAuth flow types
- `src/types/requests.rs` - protocol request types
- `src/types/responses.rs` - protocol response and stream-event types
- `src/server/*` - REST, JSON-RPC, SSE, and handler traits
- `src/client/*` - discovery, dual transport client, and SSE parsing
- `docs/proto-first-design.md` - implementation contract

## Feature Flags

```toml
[features]
default = ["server", "client"]
server = ["dep:async-trait", "dep:axum", "dep:futures-core", "dep:futures-util", "dep:tokio"]
client = ["dep:futures-core", "dep:futures-util", "dep:reqwest"]
```

`default-features = false` must keep the types-only surface working.

## Protocol and Serialization Rules

### JSON field naming

Use `camelCase` JSON field names via `#[serde(rename_all = "camelCase")]`.

### Enum values

Use proto enum strings exactly, for example:

- `TASK_STATE_COMPLETED`
- `ROLE_AGENT`

### JSON-RPC method names

Use PascalCase v1.0 method names, for example:

- `SendMessage`
- `GetTask`
- `ListTasks`
- `SubscribeToTask`

Do not introduce slash-style method names.

### Part

`Part` is a flat struct, not a tagged enum.

- exactly one of `text`, `raw`, `url`, or `data` should be set
- `raw` is modeled as `Vec<u8>` in Rust and serialized as base64 in JSON
- use `validate()` when you need an explicit semantic check

### SecurityScheme

`SecurityScheme` is externally tagged to match proto-style `oneof` JSON:

```json
{"apiKeySecurityScheme":{"location":"header","name":"X-API-Key"}}
```

Important:

- the field is `location`, not OpenAPI's `in`
- `OAuthFlows` is modeled as a oneof-style enum
- deserialization also accepts the Python SDK `type`-discriminator shape for interop
- deprecated OAuth flows still exist in the published v1.0 definitions and remain part of the wire model

### AgentId

The crate exposes an `AgentId` helper for repository-level naming rules:

- lowercase ASCII letters, digits, and `-` only
- length 3-64 characters

This helper is not a proto field.

### AUTH_REQUIRED convention

The crate exposes `AuthRequiredMetadata` plus helper methods on `Message`, `TaskStatus`, and `Task`
for the repository's `TASK_STATE_AUTH_REQUIRED` metadata convention:

- `authUrl`
- `authScheme`
- `scopes`
- `description`

### Required shape corrections already reflected in code

- `Task.context_id` is optional
- `TaskStatusUpdateEvent.context_id` is required
- `TaskArtifactUpdateEvent.context_id` is required
- `ListTaskPushNotificationConfigsResponse.next_page_token` is a string, with empty string meaning no next page
- `TaskPushNotificationConfig` is a flattened object with `url`, `token`, and `authentication`
- `SendMessageConfiguration` uses `return_immediately` and `task_push_notification_config`

### Error codes

Use:

- standard JSON-RPC codes (`-32700` to `-32603`) where appropriate
- A2A-specific codes `-32001` through `-32009`

Do not invent new A2A error codes.

## Code Style

- Derive order: `Debug, Clone, Serialize, Deserialize`
- Imports: std, external crates, crate-local
- No `unwrap()` outside tests
- No `unsafe`
- No Clawhive-specific imports or terminology
- Avoid unnecessary dependencies

## Testing Guidance

The repo now has three test layers:

- inline unit and serde tests in `src/*`
- `tests/server_integration.rs` for axum router coverage
- `tests/client_integration.rs` and `tests/client_wiremock.rs` for client behavior

Prefer:

- canonical spec/proto JSON examples
- explicit invalid-shape tests for `Part`, `SendMessageResponse`, and `StreamResponse`
- `wiremock` for client-only transport and envelope behavior
- local axum server tests for end-to-end server/client behavior

## References

- [Proto-first design](docs/proto-first-design.md)
- [A2A Protocol Spec v1.0](https://a2a-protocol.org/latest/specification/)
- [A2A Proto v1.0.0](https://github.com/a2aproject/A2A/blob/v1.0.0/specification/a2a.proto)
- [JSON-RPC 2.0 Spec](https://www.jsonrpc.org/specification)
