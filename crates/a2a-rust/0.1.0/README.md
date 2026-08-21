# a2a-rust

[![CI](https://github.com/longzhi/a2a-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/longzhi/a2a-rust/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/a2a-rust.svg)](https://crates.io/crates/a2a-rust)
[![docs.rs](https://docs.rs/a2a-rust/badge.svg)](https://docs.rs/a2a-rust)
[![License](https://img.shields.io/crates/l/a2a-rust.svg)](LICENSE-MIT)

Rust SDK for A2A Protocol v1.0.

`a2a-rust` provides:

- a proto-aligned type layer
- an axum-based server with REST, JSON-RPC, and SSE
- a reqwest-based client with discovery, dual transport, and SSE parsing
- a pluggable `TaskStore` plus `InMemoryTaskStore`

This crate has zero Clawhive-specific logic.

## Status

- Protocol lock: `v1.0.0`
- Proto package: `lf.a2a.v1`
- Implemented transports: `JSONRPC`, `HTTP+JSON`
- Out of scope: gRPC

The tagged proto is the source of truth. The repo-local implementation contract is [docs/proto-first-design.md](docs/proto-first-design.md).

## Features

| Feature | Default | Purpose |
|---|---|---|
| `server` | Yes | Router, handlers, SSE, and `TaskStore` support |
| `client` | Yes | Discovery, dual transport client, and SSE parsing |

Types-only usage:

```toml
[dependencies]
a2a-rust = { version = "0.1", default-features = false }
```

## Quick Start

Add the crate:

```toml
[dependencies]
a2a-rust = "0.1"
```

### Server

Implement `A2AHandler` and mount the router:

```rust
use a2a_rust::server::{A2AHandler, router};
use a2a_rust::types::{
    AgentCapabilities, AgentCard, AgentInterface, Message, Part, Role, SendMessageRequest,
    SendMessageResponse,
};
use a2a_rust::A2AError;

#[derive(Clone)]
struct EchoAgent;

#[async_trait::async_trait]
impl A2AHandler for EchoAgent {
    async fn get_agent_card(&self) -> Result<AgentCard, A2AError> {
        Ok(AgentCard {
            name: "Echo Agent".to_owned(),
            description: "Replies with the same text".to_owned(),
            supported_interfaces: vec![
                AgentInterface {
                    url: "/rpc".to_owned(),
                    protocol_binding: "JSONRPC".to_owned(),
                    tenant: None,
                    protocol_version: "1.0".to_owned(),
                },
                AgentInterface {
                    url: "/".to_owned(),
                    protocol_binding: "HTTP+JSON".to_owned(),
                    tenant: None,
                    protocol_version: "1.0".to_owned(),
                },
            ],
            provider: None,
            version: "1.0.0".to_owned(),
            documentation_url: None,
            capabilities: AgentCapabilities {
                streaming: Some(false),
                push_notifications: Some(false),
                extensions: Vec::new(),
                extended_agent_card: Some(false),
            },
            security_schemes: Default::default(),
            security_requirements: Vec::new(),
            default_input_modes: vec!["text/plain".to_owned()],
            default_output_modes: vec!["text/plain".to_owned()],
            skills: Vec::new(),
            signatures: Vec::new(),
            icon_url: None,
        })
    }

    async fn send_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<SendMessageResponse, A2AError> {
        Ok(SendMessageResponse::Message(Message {
            message_id: "msg-echo-1".to_owned(),
            context_id: request.message.context_id,
            task_id: None,
            role: Role::Agent,
            parts: vec![Part {
                text: Some("pong".to_owned()),
                raw: None,
                url: None,
                data: None,
                metadata: None,
                filename: None,
                media_type: None,
            }],
            metadata: None,
            extensions: Vec::new(),
            reference_task_ids: Vec::new(),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, router(EchoAgent)).await?;
    Ok(())
}
```

Runnable example:

```bash
cargo run --example echo_server --features server
```

### Client

Use discovery and send a message:

```rust
use a2a_rust::client::A2AClient;
use a2a_rust::types::{Message, Part, Role, SendMessageRequest, SendMessageResponse};

#[tokio::main]
async fn main() -> Result<(), a2a_rust::A2AError> {
    let client = A2AClient::new("http://127.0.0.1:3000")?;
    let card = client.discover_agent_card().await?;

    let response = client
        .send_message(SendMessageRequest {
            message: Message {
                message_id: "msg-1".to_owned(),
                context_id: Some("ctx-1".to_owned()),
                task_id: None,
                role: Role::User,
                parts: vec![Part {
                    text: Some("ping".to_owned()),
                    raw: None,
                    url: None,
                    data: None,
                    metadata: None,
                    filename: None,
                    media_type: None,
                }],
                metadata: None,
                extensions: Vec::new(),
                reference_task_ids: Vec::new(),
            },
            configuration: None,
            metadata: None,
            tenant: None,
        })
        .await?;

    println!("agent: {}", card.name);
    match response {
        SendMessageResponse::Message(message) => {
            println!("reply: {:?}", message.parts[0].text);
        }
        SendMessageResponse::Task(task) => {
            println!("task: {}", task.id);
        }
    }

    Ok(())
}
```

Runnable example:

```bash
cargo run --example ping_client --features client
```

## Protocol Surface

### Discovery

- `GET /.well-known/agent-card.json`

### JSON-RPC

- server default endpoint: `POST /rpc`
- compatibility alias: `POST /jsonrpc`
- method names use PascalCase v1.0 bindings such as `SendMessage`, `GetTask`, and `ListTasks`

### REST

Canonical REST endpoints include:

- `POST /message:send`
- `POST /message:stream`
- `GET /tasks`
- `GET /tasks/{id}`
- `POST /tasks/{id}:cancel`
- `GET /tasks/{id}:subscribe`
- `POST /tasks/{task_id}/pushNotificationConfigs`
- `GET /tasks/{task_id}/pushNotificationConfigs/{id}`
- `GET /tasks/{task_id}/pushNotificationConfigs`
- `DELETE /tasks/{task_id}/pushNotificationConfigs/{id}`
- `GET /extendedAgentCard`

Tenant-prefixed variants are also supported.

## Client Behavior

- Discovery caches agent cards with a configurable TTL
- Transport selection follows the server-declared `supported_interfaces` order
- Supported transports: `JSONRPC`, `HTTP+JSON`
- Streaming uses SSE and parses both `\n\n` and `\r\n\r\n` frame delimiters
- `A2A-Version: 1.0` is always sent

## Project Layout

```text
src/
  lib.rs
  error.rs
  jsonrpc.rs
  store.rs
  types/
  server/
  client/
examples/
  echo_server.rs
  ping_client.rs
tests/
  server_integration.rs
  client_integration.rs
  client_wiremock.rs
```

## Development

Core checks:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --no-default-features -- -D warnings
cargo test --all-features
cargo test --no-default-features
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for contributor workflow details.

## References

- [A2A Protocol Spec v1.0](https://a2a-protocol.org/latest/specification/)
- [A2A Proto Source (v1.0.0)](https://github.com/a2aproject/A2A/blob/v1.0.0/specification/a2a.proto)
- [A2A Agent Discovery](https://a2a-protocol.org/latest/topics/agent-discovery/)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
