# A2A Server Rust Agent (Azure Container Apps)

This project is a minimal, high-performance A2A (Agent-to-Agent) server agent implementation in Rust, specifically configured for deployment on **Azure Container Apps (ACA)**.

## Project Overview

`a2a-server-rust` is a sample application that demonstrates how to implement an A2A agent using the type-safe `a2a-rs` crate (v0.2.0). It implements the standardized A2A protocol for agentic communication, allowing this Rust agent to participate in multi-agent workflows orchestrated by other ADK or A2A agents.

### Key Technologies

*   **Language:** [Rust](https://www.rust-lang.org/) (Edition 2024)
*   **A2A Framework:** [a2a-rs](https://crates.io/crates/a2a-rs) (v0.2.0)
*   **Async Runtime:** [Tokio](https://tokio.rs/)
*   **Containerization:** [Docker](https://www.docker.com/)
*   **Deployment:** [Azure Container Apps (ACA)](https://azure.microsoft.com/en-us/products/container-apps)
*   **Container Registry:** [Azure Container Registry (ACR)](https://azure.microsoft.com/en-us/products/container-registry)
*   **CLI Tools:** [Azure CLI (az)](https://learn.microsoft.com/en-us/cli/azure/)

## Features

- **Standard A2A Endpoints:** Fully implements `/agentcard`, `/tasks`, and `/notifications` as defined by the A2A spec.
- **Echo Skill:** A built-in, out-of-the-box skill that demonstrates message receiving and replying by echoing back user inputs.
- **Task Management:** Clean in-memory task storage and robust state management.
- **Streaming Support:** Full support for Server-Sent Events (SSE) to stream task status and artifact updates.
- **Observability:** Pre-integrated structured logging and tracing using `a2a-rs::observability`.

## Getting Started

This project includes a comprehensive `Makefile` to simplify local development, testing, and cloud deployment.

### Prerequisites

*   [Rust Toolchain](https://www.rust-lang.org/tools/install) (latest stable)
*   [Docker](https://docs.docker.com/get-docker/) (running locally)
*   [Azure CLI](https://learn.microsoft.com/en-us/cli/azure/) logged in and configured:
    ```bash
    az login
    # Optionally set your active subscription:
    az account set --subscription <your-subscription-id>
    ```

### Initial Setup

1.  **Install Dependencies & Build:**
    ```bash
    cargo build
    ```

2.  **Run the Server Locally:**
    ```bash
    make start
    ```
    The server will start and listen on port `8080` (accessible at `http://localhost:8080`).

---

## Development Workflow

The `Makefile` serves as the single entry point for all operations. Run `make help` to see a detailed list of all targets.

### Building & Running
*   **Development Build:** `make build` - Compiles the project using debug settings.
*   **Release Build:** `make release` - Compiles the project with full optimizations and symbols stripped.
*   **Run Locally:** `make start` - Starts the HTTP server locally on port `8080`.
*   **Check Agent Card:** `make card` - Retrieves and formats the local agent card (requires the local server to be running).
*   **Update Dependencies:** `make update` - Runs a verbose cargo update.

### Code Quality & Testing
*   **Formatting:** `make format` - Automatically formats all Rust files with `cargo fmt`.
*   **Linting:** `make lint` - Runs `cargo clippy` (with warnings treated as errors) and formatting checks.
*   **Unit Tests:** `make test` - Runs Rust unit tests.
*   **Local A2A Echo Test:** `make a2a-local` - Executes the Python-based A2A client integration test (`tests/echo_test.py`) against the running local server.

---

## Deployment & Cloud Management

All Azure infrastructure provisioning and deployment is automated via the Azure CLI and orchestrated directly by the `Makefile`.

### Deployment to Azure Container Apps
To build, package, and deploy the agent to ACA:
```bash
make deploy
```

This automated target will:
1.  Verify local requirements (Azure CLI & Docker).
2.  Create the Azure Resource Group (defaults to `a2a-rg-westus2`) and Azure Container Registry (ACR) if they do not exist.
3.  Build the production-ready Docker image using the multi-stage `Dockerfile` based on distroless.
4.  Authenticate and push the compiled image to your private ACR.
5.  Create or update the ACA Environment and deploy the Container App with public ingress enabled on port `8080`.

### Remote Verification & Validation
Once deployed, you can verify your remote service using the following targets:
*   **Check ACA Status:** `make status` - Performs a quick check to see if the local and remote endpoints are reachable and serving the agent card.
*   **Get Endpoint FQDN:** `make endpoint` - Prints the public Fully Qualified Domain Name of your deployed Container App.
*   **Get Remote Agent Card:** `make card-remote` - Fetches and prints the remote agent's JSON capabilities card.
*   **Run Remote Validation Tests:** `make test-remote` - Runs the custom integration validation script (`tests/test_aca_validation.py`) against the live ACA endpoint.
*   **Run Remote A2A Echo Test:** `make a2a-remote` - Runs the Python-based A2A echo test against the live ACA endpoint to verify standard message exchange in the cloud.

### Logs & Resource Cleanup
*   **Tail Remote Logs:** `make logs` (or `make aca-logs`) - Streams standard output/error logs from the live Azure Container App directly to your terminal.
*   **Teardown Resources:** `make destroy` (or `make aca-destroy`) - Deletes the deployed Container App, the ACA environment, the ACR registry, and the entire Azure Resource Group to prevent ongoing Azure charges.
