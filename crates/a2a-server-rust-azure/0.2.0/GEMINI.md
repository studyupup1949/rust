# Gemini Workspace for `a2a-server-rust` (Azure Container Apps)

You are a Rust Developer working with Azure Container Apps (ACA).
You should follow Rust best practices.
The recommended language level for Rust is **Edition 2024**.

Use [crates.io](https://crates.io) as a resource to lookup Rust crates and libraries.

This document provides a developer-focused overview of the `a2a-server-rust` project, specifically tailored to provide Gemini with complete structural and environmental context.

A2A framework crate: [a2a-rs on GitHub](https://github.com/a2aproject/a2a-rs/tree/main)

---

## Project Overview

`a2a-server-rust` is a minimal, asynchronous A2A (Agent-to-Agent) server agent implementation in Rust, designed to be deployed as a containerized application on **Azure Container Apps (ACA)**. It uses the `a2a-rs` SDK (v0.2.0) and communicates over the standardized JSON-RPC based A2A protocol.

### Key Technologies

*   **Language:** [Rust](https://www.rust-lang.org/) (Edition 2024)
*   **A2A Framework:** [a2a-rs](https://crates.io/crates/a2a-rs) (v0.2.0)
*   **Async Runtime:** [Tokio](https://tokio.rs/)
*   **Containerization:** [Docker](https://www.docker.com/) (distroless final stage)
*   **Deployment Platform:** [Azure Container Apps](https://azure.microsoft.com/en-us/products/container-apps)
*   **Container Registry:** [Azure Container Registry](https://azure.microsoft.com/en-us/products/container-registry)
*   **CLI Orchestration:** [Azure CLI](https://learn.microsoft.com/en-us/cli/azure/)

---

## Code Structure

-   `src/main.rs`: Entry point. Initializes tracing-based observability, sets up the `HttpServer` with a `DefaultRequestProcessor`, defines agent metadata via `SimpleAgentInfo`, and adds the default `echo` skill.
-   `src/common/simple_agent_handler.rs`: Implements `SimpleAgentHandler`, which delegates directly to `InMemoryTaskStorage` and implements core asynchronous A2A traits (`AsyncMessageHandler`, `AsyncTaskManager`, `AsyncNotificationManager`, `AsyncStreamingHandler`).
-   `src/common/mod.rs`: Module declaration for `common`.
-   `tests/`: Contains Python-based integration, verification, and end-to-end tests:
    -   `echo_test.py`: Validates the standard A2A message exchange and echo skill.
    -   `test_a2a_client.py`: Local integration test targeting `http://localhost:8080` for discovery and tasks.
    -   `test_aca_validation.py`: Remote cloud verification script for testing the live Azure Container App endpoint.

---

## Getting Started

This project uses a detailed `Makefile` to simplify common development, testing, and deployment tasks.

### Prerequisites

*   [Rust Toolchain](https://www.rust-lang.org/tools/install) (Edition 2024 compatible)
*   [Docker](https://docs.docker.com/get-docker/) (running local daemon)
*   [Azure CLI](https://learn.microsoft.com/en-us/cli/azure/) (authenticated via `az login` with subscription set)

### Initial Setup

1.  **Install Dependencies & Build:**
    ```bash
    cargo build
    ```

2.  **Run the application locally:**
    ```bash
    make start
    ```
    The server will start and listen on port `8080`.

---

## Development Workflow

The `Makefile` contains all necessary targets to build, test, and manage the application.

### Building & Testing
*   **Development Build:** `make build` (debug build)
*   **Release Build:** `make release` (optimized production build)
*   **Unit Tests:** `make test` (runs Rust unit tests)
*   **Local A2A Echo Test:** `make a2a-local` (runs `tests/echo_test.py` against localhost)
*   **Quality Check:** `make lint` (runs `cargo clippy` and format checking)

### Interacting with the Agent
*   **Agent Card (Local):** `make card` - Inspects local capabilities at `http://localhost:8080/agentcard`.
*   **Agent Card (Remote):** `make card-remote` - Inspects remote capabilities at your deployed ACA endpoint.
*   **Status Check:** `make status` - Assesses whether local and remote services are reachable and active.
*   **Endpoint FQDN:** `make endpoint` - Retrieves the live URL of your Azure Container App.

---

## Deployment & Cloud Management

Deployment is handled via Azure CLI commands, fully orchestrated and automated by the `Makefile`.

### Deployment to Azure
To manually trigger a complete deployment, run:
```bash
make deploy
```

This target automatically:
1.  Builds the production Docker image using a multi-stage `Dockerfile`.
2.  Creates the Resource Group and Azure Container Registry (ACR) if they do not exist.
3.  Logs in to the ACR and pushes the tagged image.
4.  Creates/updates the Azure Container Apps environment and deploys the application with public external ingress on port `8080`.

### Logs & Resource Cleanup
*   **Tail Cloud Logs:** `make logs` (streams Container App stdout/stderr).
*   **Teardown Resources:** `make destroy` (deletes Container App, ACA Env, ACR registry, and the Azure Resource Group to prevent cloud charges).

---

## Interacting with Gemini

You can ask Gemini to assist with tasks in this project using the following prompt suggestions:

*   **Adding Skills:** "Add a new skill (e.g., `calculator`) to the agent's `SimpleAgentInfo` definition in `src/main.rs`."
*   **Custom Handlers:** "Implement a custom request handler in `src/common/simple_agent_handler.rs` that overrides `process_message` to add custom logic before calling the inner storage handler."
*   **Tests:** "Write a new Python-based test in the `tests/` directory to validate streaming artifact updates using Server-Sent Events (SSE)."
*   **Deployment Configuration:** "Modify `Makefile` and `Dockerfile` to support cross-compiling the binary to a Musl target for a smaller, fully static distroless deployment."
*   **Storage Backends:** "Refactor `SimpleAgentHandler` to use a persistent SQLite database instead of the `InMemoryTaskStorage` backend using `sqlx`."