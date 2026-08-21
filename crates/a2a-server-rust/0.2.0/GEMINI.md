# Gemini Workspace for `a2a-server-rust`

You are a Rust Developer working with Google Cloud.
You should follow Rust Best practices.
The recommended language level for rust is 2024.

Use [crates.io](https://crates.io) as a resource to lookup rust crates and libraries.

This document provides a developer-focused overview of the `a2a-server-rust` project, tailored for use with Gemini.

a2a crate: [a2a-rs on GitHub](https://github.com/a2aproject/a2a-rs/tree/main)

## Project Overview

`a2a-server-rust` is a minimal A2A (Agent-to-Agent) server agent implementation in Rust, designed to be deployed as a containerized application on Google Cloud Run.

### Key Technologies

*   **Language:** [Rust](https://www.rust-lang.org/) (Edition 2024)
*   **A2A Framework:** [a2a-rs](https://crates.io/crates/a2a-rs) (v0.2.0)
*   **Async Runtime:** [Tokio](https://tokio.rs/)
*   **Containerization:** [Docker](https://www.docker.com/)
*   **Deployment:** [Google Cloud Run](https://cloud.google.com/run)
*   **CI/CD:** [Google Cloud Build](https://cloud.google.com/build)

## Code Structure

- `src/main.rs`: Entry point. Initializes observability, sets up `HttpServer` with `DefaultRequestProcessor`, defines `SimpleAgentInfo`, and adds the `echo` skill.
- `src/common/simple_agent_handler.rs`: Implements `SimpleAgentHandler`, which delegates to `InMemoryTaskStorage` and implements core A2A traits (`AsyncMessageHandler`, `AsyncTaskManager`, `AsyncNotificationManager`, `AsyncStreamingHandler`).
- `src/common/mod.rs`: Module declaration for `common`.

## Getting Started

This project uses a `Makefile` to simplify common development tasks.

### Prerequisites

*   [Rust Toolchain](https://www.rust-lang.org/tools/install)
*   [Docker](https://docs.docker.com/get-docker/)
*   [Google Cloud SDK](https://cloud.google.com/sdk/docs/install)

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
*   **Test:** `make test` (unit) or `make test-remote` (remote validation)
*   **A2A Tests:** `make a2a-local` or `make a2a-remote` (runs Python echo test)
*   **Quality:** `make lint` (runs clippy and fmt check)

### Interacting with the Agent

*   **Agent Card:** Use `make card` (local) or `make card-remote` (remote) to inspect the agent's capabilities and metadata.
*   **Status:** Use `make status` to check if local and remote instances are reachable.
*   **Endpoint:** Use `make endpoint` to get the Cloud Run service URL.

## Deployment

Deployment is handled by Google Cloud Build and defined in `cloudbuild.yaml`.

### Manual Deployment

To manually trigger a deployment, run:
```bash
make deploy
```

This command:
1.  Submits the build to Google Cloud Build.
2.  Builds the Docker image (as defined in `Dockerfile`).
3.  Pushes the image to the project's container registry.
4.  Deploys the new image to Google Cloud Run.

## Interacting with Gemini

You can use Gemini to help you with various tasks in this project:

*   "Add a new skill to the agent in `main.rs`."
*   "Implement a custom request handler in `common/simple_agent_handler.rs`."
*   "Explain the A2A protocol integration in this project."
*   "Write a test for the agent's message processing logic."
*   "Refactor the `SimpleAgentHandler` to use a different storage backend."
