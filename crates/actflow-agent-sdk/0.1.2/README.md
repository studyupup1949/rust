# Actflow Agent Rust SDK

A Rust SDK for building agents for the Actflow workflow engine.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
actflow-agent-sdk = "0.1.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
serde_json = "1"
```

## Quick Start

```rust
use actflow_agent_sdk::{Agent, AgentServer, AgentOutput, Inputs, LogSender};

struct MyAgent;

impl Agent for MyAgent {
    async fn run(
        &self,
        pid: String,
        nid: String,
        inputs: Inputs,
        log: LogSender,
    ) -> AgentOutput {
        log.send("Starting agent...").await;
        // Your agent logic here
        AgentOutput::success(serde_json::json!({"result": "done"}))
    }
}

#[tokio::main]
async fn main() {
    AgentServer::new(MyAgent)
        .serve("0.0.0.0:50051")
        .await
        .unwrap();
}
```

## API Reference

### Agent Trait

```rust
pub trait Agent: Send + Sync + 'static {
    fn run(&self, pid: String, nid: String, inputs: Inputs, log: LogSender)
        -> impl Future<Output = AgentOutput> + Send;

    fn shutdown(&self) -> impl Future<Output = ()> + Send { async {} }
}
```

### AgentOutput Helpers

| Method | Description |
|--------|-------------|
| `AgentOutput::success(outputs)` | Create a successful output |
| `AgentOutput::failed(err)` | Create a failed output |
| `AgentOutput::exception(msg)` | Create an exception output |

### ExecutionStatus

| Status | Description |
|--------|-------------|
| `Pending` | Agent is pending |
| `Succeeded` | Agent completed successfully |
| `Failed` | Agent failed with error |
| `Exception` | Agent encountered exception |
| `Stopped` | Agent was stopped |
| `Paused` | Agent is paused |

### LogSender

```rust
impl LogSender {
    pub async fn send(&self, msg: impl Into<String>);  // Async send
    pub fn try_send(&self, msg: impl Into<String>);    // Non-blocking send
}
```

## Examples

### Simple Echo Agent

```bash
cargo run --example simple_agent
```

### Calculator Agent (Error Handling)

```bash
cargo run --example calculator_agent
```

### Stateful Agent (Shutdown Handling)

```bash
cargo run --example stateful_agent
```

## License

MIT
