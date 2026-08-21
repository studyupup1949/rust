# a2a-types

Pure Rust implementation of the Agent-to-Agent (A2A) Protocol types.

## Overview

This crate provides the complete type definitions for the A2A Protocol, enabling interoperability between AI agents. It includes:

- JSON-RPC 2.0 message types
- A2A protocol-specific types (Tasks, Messages, Parts)
- Error definitions
- Request/Response structures

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
a2a-types = "0.0.2"
```

Then use the types in your code:

```rust
use a2a_types::{Task, Message, MessageRole, Part, TaskState};

// Create a new message
let message = Message {
    role: MessageRole::User,
    content: vec![Part::Text {
        text: "Hello, agent!".to_string(),
        metadata: None,
    }],
    metadata: None,
};

// Work with task states
let state = TaskState::Working;
```

## Types

### Core Protocol Types
- `Task` - Represents an agent task with status and history
- `Message` - Agent communication messages
- `Part` - Content parts (text, file, data)
- `TaskState` - Task lifecycle states (Submitted, Working, Completed, etc.)

### JSON-RPC Types
- `JSONRPCRequest` - Standard JSON-RPC 2.0 request
- `JSONRPCResponse` - Success and error responses
- `JSONRPCError` - Error structure with codes

### A2A Request Types
- `SendMessageRequest` - Send a message to an agent
- `SendStreamingMessageRequest` - Stream messages
- `GetTaskRequest` - Retrieve task details
- `CancelTaskRequest` - Cancel a running task

## License

Apache-2.0