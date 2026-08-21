# A2A Server Rust Agent

This project is a minimal A2A (Agent-to-Agent) server agent implementation in Rust.

## Project Overview

`a2a-server-rust` is a sample application that demonstrates how to implement an A2A agent using the `a2a-rs` crate (v0.2.0). It is designed to be deployed as a containerized application on Amazon Lightsail and supports the A2A protocol for agentic communication.

### Key Technologies

*   **Language:** [Rust](https://www.rust-lang.org/) (Edition 2024)
*   **A2A Framework:** [a2a-rs](https://crates.io/crates/a2a-rs) (v0.2.0)
*   **Async Runtime:** [Tokio](https://tokio.rs/)
*   **Containerization:** [Docker](https://www.docker.com/)
*   **Deployment:** [Amazon Lightsail](https://aws.amazon.com/lightsail/)
*   **CLI:** [AWS CLI](https://aws.amazon.com/cli/)

## Features

- **Standard A2A Endpoints:** Implements `/agentcard`, `/tasks`, and `/notifications`.
- **Echo Skill:** A built-in skill that echoes back user messages.
- **Task Management:** In-memory task storage and state management.
- **Streaming Support:** Supports Server-Sent Events (SSE) for task status and artifact updates.
- **Observability:** Integrated tracing and logging via `a2a-rs::observability`.

## Getting Started

This project uses a `Makefile` to simplify common development tasks.

### Prerequisites

*   [Rust Toolchain](https://www.rust-lang.org/tools/install)
*   [Docker](https://docs.docker.com/get-docker/)
*   [AWS CLI](https://aws.amazon.com/cli/) configured with appropriate credentials.

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

The `Makefile` provides targets for common development tasks. Run `make help` to see all available targets.

### Building and Running

*   **Development Build:** `make build`
*   **Release Build:** `make release`
*   **Run Locally:** `make start` (starts on port 8080)
*   **Check Agent Card:** `make card` (requires local server running)
*   **Update Dependencies:** `make update`

### Code Quality

*   **Formatting:** `make format`
*   **Linting:** `make lint` (runs clippy and format check)
*   **Testing:** `make test`

### Deployment

Deployment is handled via the AWS CLI and is orchestrated by the `Makefile`. The default service name is `a2a-lightsail-rust-aws`.

To deploy to Amazon Lightsail:
```bash
make deploy
```

This command:
1.  Builds the Docker image using the multi-stage `Dockerfile`.
2.  Pushes the image to the Amazon Lightsail container service registry.
3.  Creates a new container deployment for the service on Amazon Lightsail.

### Remote Validation

After deployment, you can verify the remote service:
*   **Check Status:** `make status`
*   **Get Endpoint:** `make endpoint`
*   **Get Remote Agent Card:** `make card-remote`
*   **Run Remote Tests:** `make test-remote`
*   **Run Remote Echo Test:** `make a2a-remote`
