# A2A Protocol Examples

This directory contains examples demonstrating how to use the Acai A2A Protocol implementation.

## Simple Examples

### Simple Server (`simple_server.rs`)

A minimal server example that registers a single method handler for adding two numbers.

```bash
cargo run --example simple_server
```

### Simple Client (`simple_client.rs`)

A minimal client example that connects to a server and sends a request to add two numbers.

```bash
# Make sure the server is running first, then in a separate terminal:
cargo run --example simple_client
```

## Complete Example (`complete.rs`)

A comprehensive example that demonstrates:
- Starting a server with multiple method handlers
- Creating a client that connects to the server
- Making multiple requests with different methods
- Error handling for invalid requests

This example runs the client and server in the same process, making it easy to see the entire flow.

```bash
cargo run --example complete
```

## Streaming and Push Notification Examples

### Streaming Example (`streaming.rs`)

Demonstrates how to implement streaming responses with the A2A protocol, useful for real-time updates.

```bash
cargo run --example streaming
```

### Agent Card Example (`agent_card.rs`)

Shows how to set up an agent with metadata (agent card) that can be discovered by clients.

```bash
cargo run --example agent_card
```

### Push Notification Example (`push_notification.rs`)

Demonstrates how to use push notifications to receive updates about task status changes.

```bash
cargo run --example push_notification
```

## LLM Integration Examples

### Ollama Agent (`ollama_agent.rs`)

This example demonstrates how to build an agent that uses Ollama to generate responses by integrating with the A2A protocol.

#### Prerequisites

1. You need to have [Ollama](https://ollama.ai/) installed and running on your machine
2. You need to have the models you want to use pulled locally (e.g., `ollama pull llama3.2:3b`)

#### How It Works

This agent:

1. Takes user messages through the A2A protocol
2. Sends them to a locally running Ollama instance
3. Returns the complete generated response
4. Updates the task with the final text once generation is complete

#### Running the Example

1. Make sure Ollama is running by executing `ollama serve` in a terminal
2. Pull the LLM model if you haven't already: `ollama pull gemma3:12b-it-qat`
3. Run the agent with:

```bash
cargo run --example ollama_agent
```

The agent will start a server on `http://127.0.0.1:3000`

#### Testing the Agent

You can test the agent by sending a JSON-RPC request to the server. Here's an example using curl:

```bash
curl -X POST http://127.0.0.1:3000 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "test-1",
    "method": "tasks/send",
    "params": {
      "id": "task-1",
      "message": {
        "role": "user",
        "parts": [
          {
            "text": "What are the three laws of robotics?"
          }
        ]
      }
    }
  }'
```

You can then poll the task status to see the streaming progress:

```bash
curl -X POST http://127.0.0.1:3000 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "test-2",
    "method": "tasks/get",
    "params": {
      "id": "task-1"
    }
  }'
```

#### Customizing the Agent

The agent can be customized by modifying the following parameters:

1. **Model Selection**: Change the model in the `main()` function (e.g., from `llama3.2:3b` to another model)
2. **Ollama Host**: If you're running Ollama on a different machine, specify the host URL
3. **Generation Parameters**: Modify the Parameters struct to change temperature, context size, etc.

## Features Demonstrated

These examples demonstrate the following features:
- Basic A2A request/response handling
- Type-safe parameters and responses
- HTTP/2-based client-server communication
- Error handling and validation
- Method routing and dispatching
- Asynchronous request processing with Tokio
- Streaming responses for real-time updates
- Push notifications for event-driven architectures
- Integration with LLM systems like Ollama

## Running the Examples

Each example can be run using `cargo run --example <example_name>`. Some examples require a server to be running first, as noted in their instructions.

### Interactive A2A Chat Client

An interactive command-line client for any A2A-compatible agent:

```bash
cargo run --example a2a_chat
```

Features:
- Connects to http://localhost:3000 by default
- Interactive chat interface with command history
- View conversation history with `/history`
- Start a new conversation with `/new`
- Connect to a different agent with `/connect <url>`
- See all commands with `/help`