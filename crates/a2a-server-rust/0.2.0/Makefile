# Makefile for the Rust project

# Variables
SERVICE_NAME := a2a-server-rust
REGION := us-central1
PROJECT_ID := $(shell gcloud config get-value project)

.PHONY: all build run start clean release test lint format check-fmt clippy check docker-build deploy publish doc endpoint status card card-remote help

# The default target
all: build

# Build the project for development
build:
	@echo "Building the Rust project..."
	@cargo build

# Run the project
run:
	@echo "Running the Rust project..."
	PROJECT_ID=$(PROJECT_ID) cargo run

# Start the A2A server locally
start:
	@echo "Starting the A2A Rust server on port 8080..."
	@PORT=8080 PROJECT_ID=$(PROJECT_ID) cargo run

# Get the Cloud Run service endpoint
endpoint:
	@gcloud run services describe $(SERVICE_NAME) --platform managed --region $(REGION) --format 'value(status.url)'

# Check local and remote service status
status:
	@echo "--- Project Configuration ---"
	@echo "Project ID: $(PROJECT_ID)"
	@echo "Service:    $(SERVICE_NAME)"
	@echo "Region:     $(REGION)"
	@echo ""
	@echo "--- Service Status ---"
	@echo -n "Local (8080):  "
	@if curl -s -f http://localhost:8080/agentcard > /dev/null; then \
		NAME=$$(curl -s http://localhost:8080/agentcard | python3 -c "import sys, json; print(json.load(sys.stdin).get('name', 'Unknown'))" 2>/dev/null || echo "Online"); \
		echo "ONLINE ($$NAME)"; \
	elif curl -s -f http://localhost:8080/agent-card > /dev/null; then \
		NAME=$$(curl -s http://localhost:8080/agent-card | python3 -c "import sys, json; print(json.load(sys.stdin).get('name', 'Unknown'))" 2>/dev/null || echo "Online"); \
		echo "ONLINE ($$NAME)"; \
	elif curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/ | grep -q "200"; then \
		echo "ONLINE (Root)"; \
	else \
		echo "OFFLINE"; \
	fi
	@echo -n "Remote (Cloud): "
	@URL=$$(gcloud run services describe $(SERVICE_NAME) --platform managed --region $(REGION) --format 'value(status.url)' 2>/dev/null); \
	if [ -n "$$URL" ]; then \
		if curl -s -f $$URL/agentcard > /dev/null; then \
			NAME=$$(curl -s $$URL/agentcard | python3 -c "import sys, json; print(json.load(sys.stdin).get('name', 'Unknown'))" 2>/dev/null || echo "Online"); \
			echo "ONLINE ($$NAME) - $$URL"; \
		elif curl -s -f $$URL/agent-card > /dev/null; then \
			NAME=$$(curl -s $$URL/agent-card | python3 -c "import sys, json; print(json.load(sys.stdin).get('name', 'Unknown'))" 2>/dev/null || echo "Online"); \
			echo "ONLINE ($$NAME) - $$URL"; \
		elif curl -s -o /dev/null -w "%{http_code}" $$URL/ | grep -q "200"; then \
			echo "ONLINE (Root) - $$URL"; \
		else \
			echo "REACHABLE (Non-200) - $$URL"; \
		fi \
	else \
		echo "NOT DEPLOYED"; \
	fi

# Get the local agent card
card:
	@echo "Fetching local agent card..."
	@if curl -s -f http://localhost:8080/agentcard > /dev/null; then \
		curl -s http://localhost:8080/agentcard | python3 -m json.tool; \
	elif curl -s -f http://localhost:8080/agent-card > /dev/null; then \
		curl -s http://localhost:8080/agent-card | python3 -m json.tool; \
	else \
		echo "Error: Local agent not running or card not found at /agentcard or /agent-card"; \
		exit 1; \
	fi

# Get the remote agent card
card-remote:
	@echo "Fetching remote agent card..."
	@URL=$$(gcloud run services describe $(SERVICE_NAME) --platform managed --region $(REGION) --format 'value(status.url)' 2>/dev/null); \
	if [ -n "$$URL" ]; then \
		if curl -s -f $$URL/agentcard > /dev/null; then \
			curl -s $$URL/agentcard | python3 -m json.tool; \
		elif curl -s -f $$URL/agent-card > /dev/null; then \
			curl -s $$URL/agent-card | python3 -m json.tool; \
		else \
			echo "Error: Agent card not found at $$URL/agentcard or $$URL/agent-card"; \
			exit 1; \
		fi \
	else \
		echo "Error: Service not deployed or URL not found."; \
		exit 1; \
	fi

# Clean the project
clean:
	@echo "Cleaning the project..."
	@cargo clean

# Build the project for release
release:
	@echo "Building Release..."
	@cargo build --release

# Run tests
test:
	@echo "Running tests..."
	@cargo test

# Run remote validation test
test-remote:
	@echo "Running remote A2A validation test..."
	@python3 tests/test_cloud_run.py

# Run local A2A echo test
a2a-local:
	@echo "Running local A2A echo test..."
	@python3 tests/echo_test.py http://localhost:8080

# Run remote A2A echo test
a2a-remote:
	@echo "Running remote A2A echo test..."
	@URL=$$(gcloud run services describe $(SERVICE_NAME) --platform managed --region $(REGION) --format 'value(status.url)' 2>/dev/null); \
	if [ -n "$$URL" ]; then \
		python3 tests/echo_test.py $$URL; \
	else \
		echo "Error: Service not deployed or URL not found."; \
		exit 1; \
	fi

# Format the code
format:
	@echo "Formatting code..."
	@cargo fmt --all

# Check formatting
check-fmt:
	@echo "Checking formatting..."
	@cargo fmt --all -- --check

# Lint the code
lint: clippy check-fmt

clippy:
	@echo "Linting code..."
	@cargo clippy -- -D warnings

# Check the code
check:
	@echo "Checking the code..."
	@cargo check

# Publish the crate
publish:
	@echo "Publishing the crate..."
	@cargo publish

update:
	@cargo update --verbose

# Generate documentation
doc:
	@echo "Generating documentation..."
	@cargo doc

# Build the Docker image
docker-build:
	@echo "Building the Docker image..."
	@docker build -t gcr.io/$(PROJECT_ID)/$(SERVICE_NAME):latest .

# Submit the build to Google Cloud Build and deploy to Cloud Run
deploy:
	@echo "Submitting build to Google Cloud Build..."
	@gcloud builds submit . --config cloudbuild.yaml --substitutions=_SERVICE_NAME=$(SERVICE_NAME)

help:
	@echo "Makefile for the Rust project"
	@echo ""
	@echo "Usage:"
	@echo "    make <target>"
	@echo ""
	@echo "Targets:"
	@echo "    all          (default) same as 'build'"
	@echo "    build        Build the project for development"
	@echo "    run          Run the project"
	@echo "    start        Start the local A2A server"
	@echo "    clean        Clean the project"
	@echo "    release      Build the project for release"
	@echo "    test         Run tests"
	@echo "    test-remote  Run remote A2A validation test"
	@echo "    a2a-local    Run local A2A echo test"
	@echo "    a2a-remote   Run remote A2A echo test"
	@echo "    lint         Lint the code (clippy + fmt check)"

	@echo "    format       Format the code"
	@echo "    check-fmt    Check code formatting"
	@echo "    clippy       Lint the code"
	@echo "    check        Check the code"
	@echo "    update       Update dependencies"
	@echo "    publish      Publish the crate"
	@echo "    doc          Generate documentation"
	@echo "    endpoint     Get Cloud Run service endpoint"
	@echo "    status       Check local and remote service status"
	@echo "    card         Get the local agent card (pretty-printed)"
	@echo "    card-remote  Get the remote agent card (pretty-printed)"
	@echo "    docker-build Build the Docker image"
	@echo "    deploy       Deploy the A2A agent to Google Cloud Run (via Cloud Build)"
