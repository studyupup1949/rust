# a2a-rust Proto-First Design

Status: implementation contract
Date: 2026-03-12

This document is the repository-local design for `a2a-rust`. It intentionally does not modify the earlier external planning note. For work in this repository, this document supersedes earlier notes when they disagree.

## 1. Purpose

`a2a-rust` is a generic Rust SDK for A2A Protocol v1.0. It provides:

- a protocol-accurate type layer
- a server framework for REST, JSON-RPC, and SSE
- a client for discovery and remote invocation
- a stable foundation for downstream projects without any Clawhive-specific logic

The crate is not a concrete agent implementation and does not include hub-specific behavior.

## 2. Normative Sources

The repository follows these sources in this order:

1. Tagged proto: `v1.0.0`, commit `1736957`
2. Latest v1.0 specification text
3. Repository-local design documents

Normative links:

- Spec: <https://a2a-protocol.org/latest/specification/>
- Proto: <https://raw.githubusercontent.com/a2aproject/A2A/v1.0.0/specification/a2a.proto>

Rules:

- The proto is authoritative for data objects and request/response message shapes.
- The spec is authoritative for behavioral requirements, transport semantics, and interoperability guidance unless it conflicts with the proto.
- If spec prose and proto disagree, `a2a-rust` follows the proto for wire shape and documents the divergence.

Version lock:

- A2A protocol version: `1.0`
- Proto package: `lf.a2a.v1`
- Supported upstream release tag: `v1.0.0`

## 3. Scope

### In scope for `1.0.0`

- Full core type system from the tagged proto
- REST server endpoints from the proto HTTP annotations
- JSON-RPC support using the v1.0 method names
- SSE streaming for `SendStreamingMessage` and `SubscribeToTask`
- Agent discovery from `/.well-known/agent-card.json`
- Client support for discovery, unary calls, and streaming calls
- Push-notification request and response types
- Push-notification server and client API surface with default "not supported" behavior
- Extended agent card API surface

### Out of scope for `1.0.0`

- Agent card signature verification
- Generated proto bindings as the public API
- gRPC transport implementation
- Clawhive-specific storage, config, terminology, or adapters

## 4. Known Spec Drift and Repository Policy

The spec and the tagged proto are not perfectly aligned. This crate uses the following policy:

### 4.1 Method names

Use PascalCase JSON-RPC methods from the latest v1.0 binding tables:

- `SendMessage`
- `SendStreamingMessage`
- `GetTask`
- `ListTasks`
- `CancelTask`
- `SubscribeToTask`
- `CreateTaskPushNotificationConfig`
- `GetTaskPushNotificationConfig`
- `ListTaskPushNotificationConfigs`
- `DeleteTaskPushNotificationConfig`
- `GetExtendedAgentCard`

The old slash-style methods from earlier A2A versions are not the primary design target.

### 4.2 JSON-RPC endpoint path

The agent card declares the actual JSON-RPC URL in `supportedInterfaces`. The crate should not hardcode `/jsonrpc` as the protocol contract.

Repository decision:

- server router defaults the JSON-RPC endpoint to `/rpc`
- router configuration may add compatibility aliases such as `/jsonrpc`
- the advertised `AgentInterface.url` must match the actual deployed endpoint

### 4.3 Subscribe REST method

The spec prose and proto HTTP annotation disagree for `SubscribeToTask`.

- proto canonical binding: `GET /tasks/{id}:subscribe`
- spec prose still contains `POST /tasks/{id}:subscribe`

Repository decision:

- server exposes `GET` as canonical behavior
- server may optionally accept `POST` as a compatibility alias
- client uses `GET`

### 4.4 SendMessageConfiguration field naming

The final tagged proto defines:

- `acceptedOutputModes`
- `taskPushNotificationConfig`
- `historyLength`
- `returnImmediately`

Repository decision:

- canonical Rust and JSON field name is `return_immediately`
- canonical push-config field is `task_push_notification_config`
- no RC compatibility alias is required in `1.0.0`

## 5. Crate Architecture

The crate is layered so that `types` remains transport-agnostic and low-dependency.

```text
types/jsonrpc/error
        ^
        |
   server   client
        ^
        |
      store
```

### 5.1 Modules

```text
src/
  lib.rs
  error.rs
  jsonrpc.rs
  types/
    mod.rs
    agent_card.rs
    task.rs
    message.rs
    security.rs
    push.rs
    requests.rs
    responses.rs
  server/
    mod.rs
    handler.rs
    router.rs
    rest.rs
    jsonrpc.rs
    streaming.rs
  client/
    mod.rs
    discovery.rs
    api.rs
  store.rs
```

### 5.2 Feature flags

- default: `server`, `client`
- `server`: enables `axum`, SSE helpers, router, and server-side error mapping
- `client`: enables `reqwest`, discovery, streaming parsers, and client transport errors

Types, JSON-RPC envelopes, and core errors must compile with `default-features = false`.

## 6. Rust Data Model Decisions

### 6.1 Core mapping rules

Use `serde(rename_all = "camelCase")` on all protocol structs unless a field requires special handling.

Mapping choices:

- `google.protobuf.Struct` -> `JsonObject = serde_json::Map<String, serde_json::Value>`
- `google.protobuf.Value` -> `serde_json::Value`
- `google.protobuf.Timestamp` -> `String` containing RFC 3339 UTC text
- `bytes` -> `Vec<u8>` with custom base64 serde to match ProtoJSON
- proto maps -> `BTreeMap<_, _>` for deterministic serialization in tests and docs

Rationale:

- The public API should match ProtoJSON exactly.
- `types` should not force a time crate on downstream users in `1.0.0`.
- Deterministic map ordering simplifies serde tests and snapshots.

### 6.2 Oneof handling

Use enums for pure wrapper `oneof` messages:

- `SendMessageResponse`
- `StreamResponse`
- `SecurityScheme`
- `OAuthFlows`

Use structs with validation for mixed-content messages where `oneof` fields coexist with shared fields:

- `Part`

Validation helpers will reject invalid states such as:

- `Part` containing zero or more than one of `text`, `raw`, `url`, `data`
- `SendMessageResponse` or `StreamResponse` with impossible payload combinations

### 6.3 Proto-driven corrections to the previous design

These decisions are required by the tagged proto:

- `Task.context_id` is optional
- `TaskStatusUpdateEvent.context_id` is required
- `TaskArtifactUpdateEvent.context_id` is required
- `Role` includes `ROLE_UNSPECIFIED`
- `Artifact` does not have an `index` field
- `SecurityRequirement` is a wrapper object with `schemes`
- `OAuth2SecurityScheme.flows` is a typed object, not raw JSON
- `ListTasksRequest` and `ListTasksResponse` are richer than the earlier planning note
- `SubscribeToTaskRequest` only contains `tenant` and `id`

### 6.4 Compatibility leniency

`a2a-rust` should serialize canonically and deserialize strictly by default. The only planned lenient handling in `1.0.0` is:

- alternate `SecurityScheme` JSON shapes used by known SDKs, if they can be accepted without ambiguity

Any broader compatibility behavior should live behind explicit helper code, not weaken the canonical serializer.

## 7. Protocol Objects to Implement

The type layer must include at least:

- `AgentCard`, `AgentInterface`, `AgentProvider`, `AgentCapabilities`, `AgentExtension`, `AgentSkill`, `AgentCardSignature`
- `Task`, `TaskStatus`, `TaskState`
- `Message`, `Role`, `Part`, `Artifact`
- `TaskStatusUpdateEvent`, `TaskArtifactUpdateEvent`
- `AuthenticationInfo`, `TaskPushNotificationConfig`
- `SecurityRequirement`, `SecurityScheme`, all security scheme variants, `OAuthFlows`, and OAuth flow structs
- all request messages from the tagged proto
- all response messages from the tagged proto

The public type names should match the operation and object names from the tagged proto unless there is a compelling Rust API reason not to.

## 8. Error Model

The core error layer must cover all A2A-specific errors currently defined by the latest spec:

- `TaskNotFoundError` -> `-32001`
- `TaskNotCancelableError` -> `-32002`
- `PushNotificationNotSupportedError` -> `-32003`
- `UnsupportedOperationError` -> `-32004`
- `ContentTypeNotSupportedError` -> `-32005`
- `InvalidAgentResponseError` -> `-32006`
- `ExtendedAgentCardNotConfiguredError` -> `-32007`
- `ExtensionSupportRequiredError` -> `-32008`
- `VersionNotSupportedError` -> `-32009`

Design rules:

- `A2AError` is transport-neutral in the core crate
- JSON-RPC and HTTP mapping helpers live alongside the core error type
- `reqwest` conversions are only compiled under the `client` feature
- server transport errors and parsing failures map to standard JSON-RPC errors or HTTP problem details as appropriate

## 9. Server Design

### 9.1 Handler trait

Expose a user-implemented `A2AHandler` trait with default methods for optional functionality.

Required methods:

- `get_agent_card`
- `send_message`

Defaulted methods:

- `send_streaming_message`
- `get_task`
- `list_tasks`
- `cancel_task`
- `subscribe_to_task`
- push notification config CRUD/list
- `get_extended_agent_card`

Defaults should return the appropriate A2A error instead of panicking.

### 9.2 Router behavior

The server router should:

- expose the well-known discovery document at `/.well-known/agent-card.json`
- expose canonical REST endpoints from the proto annotations
- support tenant-prefixed additional bindings such as `/{tenant}/tasks`
- expose a configurable JSON-RPC endpoint, default `/rpc`
- return SSE with `data: <json>\n\n` framing for streaming endpoints

### 9.3 Streaming behavior

`SendStreamingMessage`:

- message-only flow: emit exactly one `StreamResponse::Message` and close
- task flow: emit initial `StreamResponse::Task`, then zero or more status or artifact updates, then close when interrupted or terminal

`SubscribeToTask`:

- emit current `Task` first
- then stream updates
- reject terminal tasks with `UnsupportedOperationError`

### 9.4 Capability gates

The server must enforce card-declared capabilities:

- if `capabilities.streaming != Some(true)`, streaming operations return `UnsupportedOperationError`
- if `capabilities.push_notifications != Some(true)`, push-config operations return `PushNotificationNotSupportedError`
- if `capabilities.extended_agent_card != Some(true)`, extended card requests return `ExtendedAgentCardNotConfiguredError`

## 10. Client Design

### 10.1 Discovery

`AgentCardDiscovery` should:

- fetch `/.well-known/agent-card.json`
- cache by base URL with TTL
- return the cached result unless refresh is requested

### 10.2 Transport selection

The client should inspect `supportedInterfaces` and prefer:

1. `JSONRPC`
2. `HTTP+JSON`

`GRPC` is out of scope for `1.0.0`.

The client always sends `A2A-Version: 1.0` and optionally `A2A-Extensions`.

### 10.3 JSON-RPC client behavior

The client must:

- use the PascalCase v1.0 method names
- preserve the request `id` on response matching
- distinguish JSON-RPC transport errors from A2A application errors
- parse SSE streams as sequences of `StreamResponse` objects

### 10.4 REST behavior

The client should support REST at least for:

- discovery
- `SendMessage`
- `SendStreamingMessage`
- `GetTask`
- `ListTasks`
- `CancelTask`
- `SubscribeToTask`
- `GetExtendedAgentCard`

Push-notification REST methods should exist in the API surface even if many servers return "not supported".

## 11. TaskStore

`TaskStore` is server-side infrastructure and stays outside `types`.

Required behavior:

- get by task id
- upsert task
- list tasks with cursor pagination semantics
- delete task

The built-in `InMemoryTaskStore` should support:

- TTL-based expiration
- bounded capacity
- deterministic ordering for `ListTasks`

`ListTasks` ordering requirement:

- descending by task status timestamp

## 12. Validation Strategy

Serde alone is not sufficient. The crate should provide explicit validation for:

- required proto-oneof invariants
- object-only metadata and params fields
- page size bounds
- required first-event behavior in subscription streams
- capability and extension checks

Validation should happen in three places:

1. deserialization-time when it is cheap and unambiguous
2. explicit `validate()` helpers on public request and response types
3. server/client operation boundaries before wire transmission

## 13. Testing Strategy

### 13.1 Type tests

Most important tests:

- serde round trips for every protocol object
- canonical examples from spec and proto
- rejection tests for invalid `oneof` states
- JSON base64 handling for `Part.raw`
- security scheme deserialization

### 13.2 Server integration

Use `tower::ServiceExt` against the axum router to test:

- well-known discovery
- tenant and non-tenant REST routes
- JSON-RPC dispatch
- SSE framing and ordering
- push and extended-card default errors

### 13.3 Client integration

Use `wiremock` to test:

- discovery caching
- JSON-RPC unary calls
- REST fallback behavior
- SSE parsing
- error mapping

## 14. Implementation Order

Phase 1:

- `Cargo.toml` feature wiring
- `lib.rs`
- `error.rs`
- `jsonrpc.rs`
- `types/*`

Phase 2:

- `server/*`
- `store.rs`
- server integration tests

Phase 3:

- `client/*`
- client integration tests
- examples and docs cleanup

## 15. Non-Negotiable Constraints

- no Clawhive-specific logic
- no `unsafe`
- no `unwrap()` outside tests
- no invented A2A error codes
- no divergence from tagged proto field names or shapes without an explicitly documented compatibility reason

## 16. Open Issues to Revisit Later

- whether to add optional compatibility aliases for older RC-era field names if real interop requires them
- whether to expose typed timestamp wrappers in a future opt-in feature
- whether to accept additional JSON-RPC endpoint aliases beyond `/rpc`
- whether a separate `compat` module is needed for cross-SDK interoperability quirks beyond `SecurityScheme`
