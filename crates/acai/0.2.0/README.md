# acai

## Overview

acai is a robust Rust framework for building and integrating with AI agent services. It provides a
seamless interface for agent-to-agent communication, enabling the creation of sophisticated AI
systems that can interact with each other and with external services. acai implements a JSON-RPC
based protocol that makes it easy to build agents that can handle complex tasks, streaming
responses, and asynchronous workflows.

## Features

### Agent-to-Agent Communication

acai excels at enabling reliable communication between different AI agents. Its protocol is designed
to support complex interactions while maintaining a clean separation of concerns:

- **Task-based communication model**: Agents can submit tasks, receive updates, and process
  responses using a well-defined task lifecycle.
- **Bidirectional streaming**: Support for real-time, bidirectional streaming enables responsive
  agent experiences.
- **Asynchronous workflows**: Tasks can be processed asynchronously, with push notifications to keep
  clients informed of status changes.

### Robust Server Implementation

The server-side components of acai are built to handle the complexities of agent services:

- **Task management**: The TaskManager provides a complete lifecycle for tasks, including creation,
  status updates, artifact generation, and task completion.
- **Streaming support**: Server-sent events (SSE) allow for efficient streaming of updates to
  clients.
- **Form handling**: Built-in support for structured form data collection and validation.
- **Push notifications**: Configurable webhook-based notifications to keep external systems informed
  of task progress.

### Client Capabilities

acai includes a fully-featured client that simplifies interaction with agent services:

- **Simple API**: The client API makes it easy to submit tasks, subscribe to updates, and process
  responses.
- **Error handling**: Comprehensive error types and handling to make debugging and recovery
  straightforward.
- **Configurable behavior**: Clients can be configured with custom timeouts, authentication, and
  more.
- **Transport agnostic**: The client can work with HTTP, WebSockets, or custom transport layers.

### Security Features

- **JSON Web Key Sets (JWKS)**: Support for secure key management and authentication.
- **Webhook signature validation**: Secure push notifications with request signing.
- **Flexible authentication**: Multiple authentication schemes supported, including API keys and
  bearer tokens.

## Use Cases

acai is designed to support a variety of agent interaction patterns:

- **AI assistants**: Build AI assistants that can process complex user requests and generate
  structured responses.
- **Multi-agent systems**: Create systems where multiple specialized agents collaborate to solve
  complex problems.
- **Integration hubs**: Connect multiple AI services together to create workflows that leverage the
  strengths of each service.
- **Real-time applications**: Build applications that require real-time updates and streaming
  responses from AI services.
- **Long-running tasks**: Support for tasks that may take extended periods to complete, with status
  updates and push notifications.

## Getting Started

### Server Implementation

Setting up a basic acai server is straightforward:

```rust
use acai::{server, AgentServer};

// Create a server with default configuration
let server = AgentServer::new();

// Register handlers for various endpoints
server.register_handler("example/method", example_handler);

// Start the server
server.start("127.0.0.1:8000").await?;
```

### Client Usage

Connecting to an acai service is equally simple:

```rust
use acai::{Client, ClientConfig};

// Create a client with the target URL
let client = Client::new(ClientConfig::new("https://example.com/agent"));

// Send a task and get a response
let response = client.send("example_task_id", message).await?;

// Or subscribe to streaming updates
let mut stream = client.stream("example_task_id").await?;
while let Some(update) = stream.next().await {
    // Process the update
}
```

## Architecture

acai is built around a few core concepts:

- **Tasks**: The fundamental unit of work in acai. Tasks have a lifecycle with well-defined states
  (Submitted, Working, InputRequired, Completed, Failed, Canceled).
- **Messages**: The content exchanged between agents, composed of parts that can contain text, data,
  or files.
- **Artifacts**: Structured output produced by agents, which can be progressively updated during
  task execution.
- **Push Notifications**: Webhooks that notify external systems of task status changes.

The architecture separates concerns between client and server components while keeping the
communication protocol clear and extensible.

## Status

acai is under active development. While the core functionality is stable, new features are being
added regularly to enhance its capabilities. The API is subject to change as we refine the framework
based on real-world usage and feedback.

Notably absent is any notion of persistence.

## Contributing

Contributions to acai are welcome! Please feel free to submit issues and pull requests. When
contributing code:

- Follow the Rust coding conventions
- Include comprehensive documentation for public APIs
- Add tests for new functionality
- Run `cargo clippy` and `cargo fmt` before submitting

## License

Apache 2.0.  All contributions must retain this license.

## Documentation

Comprehensive API documentation is available at [docs.rs](https://docs.rs/acai/latest/acai/).

For examples and more detailed usage information, check the `examples/` directory in the
[repository](https://github.com/rescrv/acai).
