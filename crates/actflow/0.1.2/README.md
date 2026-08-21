# Actflow

Actflow is a lightweight, event-driven workflow engine written in Rust. It is designed to be embedded in your applications to orchestrate complex business logic with ease.

## Features

- **Event-Driven Architecture**: Built on top of a robust event bus, ensuring high decoupling and scalability.
- **Async Execution**: Powered by `tokio`, supporting high-concurrency workflow execution.
- **Pluggable Storage**: Supports in-memory storage for testing and PostgreSQL for production persistence.
- **Flexible Workflow Definition**: Define workflows using JSON, supporting various node types and control flows.
- **Automatic Process Management**: Automatically manages the lifecycle of processes, including execution, suspension, and cleanup.

## Supported Actions

| Action | Description |
|--------|-------------|
| `start` | Entry point of the workflow |
| `end` | End point of the workflow |
| `http_request` | HTTP request with support for GET/POST/PUT/DELETE, authentication (Bearer/Basic/Custom), headers, params, and body |
| `if_else` | Conditional branching based on variable comparisons (equals, not_equals, contains, greater_than, etc.) |
| `code` | Execute JavaScript or Python code with variable inputs and JSON outputs |
| `agent` | Call remote agent service via gRPC with streaming support for logs and outputs |

## Template Variables

Actflow supports template variables to reference outputs from other nodes:

```
{{#nodeId.key#}}
{{#nodeId.key.subkey#}}
```

Example: `{{#n1.body.data.user.name#}}` references the `name` field from node `n1`'s output.

You can also reference environment variables from the `Context`:

```
{{$VAR_NAME$}}
```

Example: `{{$API_KEY$}}` references the `API_KEY` environment variable.

## Quick Start

Here is a simple example of how to define and run a workflow:

```rust
use std::collections::HashMap;

use actflow::{ChannelEvent, ChannelOptions, EdgeModel, EngineBuilder, NodeModel, WorkflowModel};
use serde_json::json;

fn main() {
    // 1. Initialize Engine
    let engine = EngineBuilder::new().build().unwrap();

    // 2. Launch the engine
    engine.launch();

    // 3. Define a Workflow
    let workflow = WorkflowModel {
        id: "hello_world".to_string(),
        name: "hello_world".to_string(),
        desc: "A simple hello world workflow".to_string(),
        env: HashMap::new(),
        nodes: vec![
            NodeModel {
                id: "n1".to_string(),
                title: "Step 1".to_string(),
                desc: "Start node".to_string(),
                uses: "start".to_string(),
                action: json!({}),
                ..Default::default()
            },
            NodeModel {
                id: "n2".to_string(),
                title: "Step 2".to_string(),
                desc: "HTTP Request node".to_string(),
                uses: "http_request".to_string(),
                action: json!({
                    "url": "https://httpbin.org/get",
                    "method": "GET",
                    "auth": {
                        "auth_type": "no_auth"
                    },
                    "headers": {},
                    "params": {},
                    "body": {
                        "content_type": "none"
                    },
                    "timeout": 30000
                }),
                ..Default::default()
            },
        ],
        edges: vec![EdgeModel {
            id: "e1".to_string(),
            source: "n1".to_string(),
            target: "n2".to_string(),
            source_handle: "source".to_string(),
        }],
    };

    // 4. Deploy Workflow
    engine.deploy(&workflow).unwrap();

    // 5. Build Process
    let process = engine.build_process(&workflow.id).unwrap();

    // 6. Listen to events
    ChannelEvent::channel(engine.channel(), ChannelOptions::with_pid(process.id().to_string())).on_complete(move |pid| {
        println!("Workflow completed, pid: {}", pid);
    });

    ChannelEvent::channel(engine.channel(), ChannelOptions::with_pid(process.id().to_string())).on_error(move |e| {
        println!("Workflow failed: {:?}", e);
    });

    ChannelEvent::channel(engine.channel(), ChannelOptions::with_pid(process.id().to_string())).on_log(move |e| {
        println!("Workflow log: {:?}", e);
    });

    ChannelEvent::channel(engine.channel(), ChannelOptions::with_pid(process.id().to_string())).on_event(move |e| {
        println!("Event: {:?}", e);
    });

    // 7. Run Workflow
    let pid = engine.run_process(process.clone()).unwrap();
    println!("Started process: {}", pid);

    loop {
        if process.is_complete() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let outputs: serde_json::Value = process.get_outputs().into();
    println!();
    println!("----------------------------------------");
    println!("Outputs: {:#?}", outputs);
}
```

## Architecture

Actflow consists of several core components:

- **Engine**: The entry point of the system, responsible for managing resources, loading workflows, and spawning processes.
- **Channel**: An internal event bus that broadcasts events (Workflow, Node, Log) to subscribers.
- **Process**: An instance of a running workflow. It maintains the execution context and state.
- **Dispatcher**: Responsible for scheduling node execution based on the workflow graph and current state.
- **Store**: Abstracted storage layer (Memory/Postgres) for persisting workflows, process states, and execution history.

## Configuration

You can configure Actflow using the `Config` struct or a TOML file.

```toml
async_worker_thread_number = 16

[store]
store_type = "postgres"

[store.postgres]
database_url = "postgres://user:pass@localhost:5432/actflow"
```
