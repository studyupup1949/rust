# Gemini Workspace for `a2a-server-rust`

You are a Rust Developer working with Amazon AWS.
You should follow Rust Best practices.
The recommended language level for rust is 2024.

Use [crates.io](https://crates.io) as a resource to lookup rust crates and libraries.

This document provides a developer-focused overview of the `a2a-server-rust` project, tailored for use with Gemini.

a2a crate: [a2a-rs on GitHub](https://github.com/a2aproject/a2a-rs/tree/main)

## Project Overview

`a2a-server-rust` is a minimal A2A (Agent-to-Agent) server agent implementation in Rust, designed to be deployed as a containerized application on Amazon Lightsail.

### Key Technologies

*   **Language:** [Rust](https://www.rust-lang.org/) (Edition 2024)
*   **A2A Framework:** [a2a-rs](https://crates.io/crates/a2a-rs) (v0.2.0)
*   **Async Runtime:** [Tokio](https://tokio.rs/)
*   **Containerization:** [Docker](https://www.docker.com/)
*   **Deployment:** [Amazon Lightsail](https://aws.amazon.com/lightsail/)
*   **CLI:** [AWS CLI](https://aws.amazon.com/cli/)

## Code Structure

- `src/main.rs`: Entry point. Initializes observability, sets up `HttpServer` with `DefaultRequestProcessor`, defines `SimpleAgentInfo`, and adds the `echo` skill.
- `src/common/simple_agent_handler.rs`: Implements `SimpleAgentHandler`, which delegates to `InMemoryTaskStorage` and implements core A2A traits (`AsyncMessageHandler`, `AsyncTaskManager`, `AsyncNotificationManager`, `AsyncStreamingHandler`).
- `src/common/mod.rs`: Module declaration for `common`.
- `tests/`: Contains Python-based integration tests (`echo_test.py`, `test_a2a_client.py`) and remote validation scripts.

## Getting Started

This project uses a `Makefile` to simplify common development tasks.

### Prerequisites

*   [Rust Toolchain](https://www.rust-lang.org/tools/install)
*   [Docker](https://docs.docker.com/get-docker/)
*   [AWS CLI](https://aws.amazon.com/cli/)

### Initial Setup

1.  **Install Dependencies:**
    ```bash
    cargo build
    ```

2.  **Run the application locally:**
    ```bash
    make start
    ```
    The server will start on port `8080`.

## Development Workflow

The `Makefile` provides targets for common development tasks. Run `make help` for a full list.

### Building and Testing

*   **Build:** `make build` (dev) or `make release` (release)
*   **Test:** `make test` (unit)
*   **A2A Tests:** `make a2a-local` or `make a2a-remote` (runs Python echo test)
*   **Quality:** `make lint` (runs clippy and fmt check)

### Interacting with the Agent

*   **Agent Card:** Use `make card` (local) or `make card-remote` (remote) to inspect the agent's capabilities and metadata.
*   **Status:** Use `make status` to check if local and remote instances are reachable.
*   **Endpoint:** Use `make endpoint` to get the AWS Lightsail service URL.

## Deployment

Deployment is handled by AWS CLI commands via the Makefile. The default service name is `a2a-lightsail-rust-aws`.

### Manual Deployment

To manually trigger a deployment, run:
```bash
make deploy
```

This command:
1.  Builds the Docker image using multi-stage cross-compilation (as defined in `Dockerfile`).
2.  Pushes the image to Amazon Lightsail container registry.
3.  Creates a new container deployment for the service on AWS Lightsail.

## Interacting with Gemini

You can use Gemini to help you with various tasks in this project:

*   "Add a new skill to the agent in `main.rs`."
*   "Implement a custom request handler in `common/simple_agent_handler.rs`."
*   "Explain the A2A protocol integration in this project."
*   "Write a test for the agent's message processing logic."
*   "Refactor the `SimpleAgentHandler` to use a different storage backend."