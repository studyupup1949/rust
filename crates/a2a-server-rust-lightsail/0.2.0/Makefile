# Include and export AWS credentials if they exist
-include .aws_creds
export

export AWS_PAGER :=

# Variables
SERVICE_NAME := a2a-server-rust

# AWS Lightsail Configuration
AWS_REGION ?= us-east-1
LIGHTSAIL_SERVICE_NAME ?= a2a-lightsail-rust-aws
IMAGE_NAME ?= a2a-server-image

.PHONY: all build run start clean release test lint format check-fmt clippy check docker-build deploy publish doc endpoint status card card-remote help lightsail deploy-lightsail lightsail-status push-lightsail create-deployment aws-destroy a2a-local a2a-remote test-remote update

# The default target
all: build

# Build the project for development
build:
	@echo "Building the Rust project..."
	@cargo build

# Run the project
run:
	@echo "Running the Rust project..."
	@cargo run

# Start the A2A server locally
start:
	@echo "Starting the A2A Rust server on port 8080..."
	@PORT=8080 cargo run

# Get the AWS Lightsail service endpoint
endpoint:
	@aws lightsail get-container-services --service-name $(LIGHTSAIL_SERVICE_NAME) --region $(AWS_REGION) \
		--query 'containerServices[0].url' \
		--output text

# Check local and remote service status
status: lightsail-status
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
	@URL=$$(aws lightsail get-container-services --service-name $(LIGHTSAIL_SERVICE_NAME) --region $(AWS_REGION) --query 'containerServices[0].url' --output text 2>/dev/null | sed 's|/$$||'); \
	if [ -n "$$URL" ] && [ "$$URL" != "None" ]; then \
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

# Target to show AWS Lightsail status
lightsail-status:
	@echo "Checking AWS Lightsail service status for $(LIGHTSAIL_SERVICE_NAME)..."
	@aws lightsail get-container-services --service-name $(LIGHTSAIL_SERVICE_NAME) --region $(AWS_REGION) \
		--query 'containerServices[0].{State:state, Power:power, Deployment:currentDeployment.state, URL:publicEndpoint.url}' \
		--output table

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
	@URL=$$(aws lightsail get-container-services --service-name $(LIGHTSAIL_SERVICE_NAME) --region $(AWS_REGION) --query 'containerServices[0].url' --output text 2>/dev/null | sed 's|/$$||'); \
	if [ -n "$$URL" ] && [ "$$URL" != "None" ]; then \
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
	@URL=$$(aws lightsail get-container-services --service-name $(LIGHTSAIL_SERVICE_NAME) --region $(AWS_REGION) --query 'containerServices[0].url' --output text 2>/dev/null); \
	if [ -n "$$URL" ] && [ "$$URL" != "None" ]; then \
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
	@docker build --platform linux/amd64 -t $(IMAGE_NAME) .

# Deploy to Amazon Lightsail
deploy-lightsail: docker-build push-lightsail create-deployment
lightsail: deploy-lightsail
deploy: lightsail

push-lightsail:
	@echo "Pushing the Docker image to Lightsail..."
	@aws lightsail push-container-image --region $(AWS_REGION) --service-name $(LIGHTSAIL_SERVICE_NAME) --label $(IMAGE_NAME) --image $(IMAGE_NAME)

create-deployment:
	@echo "Creating a new deployment for Lightsail Container Service..."
	@IMAGE_IDENTIFIER=$$(aws lightsail get-container-images --service-name $(LIGHTSAIL_SERVICE_NAME) --query "containerImages[0].image" --output text) && \
	URL=$$(aws lightsail get-container-services --service-name $(LIGHTSAIL_SERVICE_NAME) --region $(AWS_REGION) --query 'containerServices[0].url' --output text | sed 's|https://||;s|/||') && \
	aws lightsail create-container-service-deployment --region $(AWS_REGION) \
		--service-name $(LIGHTSAIL_SERVICE_NAME) \
		--containers "{\"$(LIGHTSAIL_SERVICE_NAME)\":{\"image\":\"$$IMAGE_IDENTIFIER\",\"ports\":{\"8080\":\"HTTP\"},\"environment\":{\"ALLOWED_HOSTS\":\"*,$$URL,0.0.0.0,localhost,127.0.0.1\"}}}" \
		--public-endpoint "{\"containerName\":\"$(LIGHTSAIL_SERVICE_NAME)\",\"containerPort\":8080,\"healthCheck\":{\"path\":\"/\"}}"

aws-destroy:
	@echo "Destroying AWS resources for $(LIGHTSAIL_SERVICE_NAME)..."
	@aws lightsail delete-container-service --service-name $(LIGHTSAIL_SERVICE_NAME) --region $(AWS_REGION) 2>/dev/null || true
	@echo "AWS resources destroyed."

help:
	@echo "Makefile for the Rust project"
	@echo ""
	@echo "Usage:"
	@echo "    make <target>"
	@echo ""
	@echo "Targets:"
	@echo "    all              (default) same as 'build'"
	@echo "    build            Build the project for development"
	@echo "    run              Run the project"
	@echo "    start            Start the local A2A server"
	@echo "    clean            Clean the project"
	@echo "    release          Build the project for release"
	@echo "    test             Run tests"
	@echo "    test-remote      Run remote A2A validation test"
	@echo "    a2a-local        Run local A2A echo test"
	@echo "    a2a-remote       Run remote A2A echo test"
	@echo "    lint             Lint the code (clippy + fmt check)"
	@echo "    format           Format the code"
	@echo "    clippy           Lint the code"
	@echo "    check            Check the code"
	@echo "    update           Update dependencies"
	@echo "    publish          Publish the crate"
	@echo "    doc              Generate documentation"
	@echo "    endpoint         Get AWS Lightsail service endpoint"
	@echo "    status           Check local and remote service status"
	@echo "    card             Get the local agent card (pretty-printed)"
	@echo "    card-remote      Get the remote agent card (pretty-printed)"
	@echo "    docker-build     Build the Docker image"
	@echo "    deploy           Deploy the agent to AWS Lightsail"
	@echo "    lightsail-status Check AWS Lightsail status"
	@echo "    aws-destroy      Destroy AWS Lightsail resources"
