# Makefile for a2a-server-rust

# Azure Configuration
AZ_LOCATION ?= westus2
AZ_RESOURCE_GROUP ?= a2a-rg-westus2
AZ_ACR_NAME ?= a2aacr$(shell hostname | tr -cd '[:alnum:]' | tr '[:upper:]' '[:lower:]' | cut -c1-10)v2
AZ_ACA_ENV_NAME ?= a2a-env-$(shell hostname | tr -cd '[:alnum:]' | tr '[:upper:]' '[:lower:]' | cut -c1-10)
AZ_ACA_NAME ?= a2a-app-$(shell hostname | tr -cd '[:alnum:]' | tr '[:upper:]' '[:lower:]' | cut -c1-10)
IMAGE_NAME ?= a2a-server-image
IMAGE_TAG ?= $(shell git rev-parse --short HEAD 2>/dev/null || date +%s)

# Derived Azure variables
ACR_LOGIN_SERVER := $(AZ_ACR_NAME).azurecr.io
FULL_IMAGE_NAME := $(ACR_LOGIN_SERVER)/$(IMAGE_NAME):$(IMAGE_TAG)

# Project Variables
SERVICE_NAME := a2a-server-rust

.PHONY: all build run start clean release test lint format check-fmt clippy check docker-build deploy status local-status aca-status endpoint push-acr az-login acr-create acr-login aca-logs aca-destroy destroy logs card card-remote a2a-local a2a-remote test-remote update check-requirements

# Check for required tools
check-requirements:
	@command -v az >/dev/null 2>&1 || { echo >&2 "Azure CLI (az) is required but not installed. Aborting."; exit 1; }
	@command -v docker >/dev/null 2>&1 || { echo >&2 "Docker is required but not installed. Aborting."; exit 1; }
	@echo "All requirements met."

# The default target
all: build

# Build the project for development
build:
	@echo "Building the Rust project..."
	@cargo build

# Run the project locally
run:
	@echo "Running the Rust project..."
	@cargo run

# Start the A2A server locally on port 8080
start:
	@echo "Starting the A2A Rust server on port 8080..."
	@PORT=8080 cargo run

# Check local and remote service status
status: aca-status
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
	@FQDN=$$(az containerapp show --name $(AZ_ACA_NAME) --resource-group $(AZ_RESOURCE_GROUP) --query "properties.configuration.ingress.fqdn" --output tsv 2>/dev/null); \
	if [ -n "$$FQDN" ] && [ "$$FQDN" != "None" ]; then \
		URL="https://$$FQDN"; \
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
		fi; \
	else \
		echo "NOT DEPLOYED"; \
	fi

# Check ACA Status from Azure
aca-status:
	@echo "Checking Azure Container App status for $(AZ_ACA_NAME)..."
	@az containerapp show --name $(AZ_ACA_NAME) --resource-group $(AZ_RESOURCE_GROUP) --query "{Name:name, ProvisioningState:properties.provisioningState, FQDN:properties.configuration.ingress.fqdn}" --output table 2>/dev/null || echo "Container App $(AZ_ACA_NAME) not found."

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
	@FQDN=$$(az containerapp show --name $(AZ_ACA_NAME) --resource-group $(AZ_RESOURCE_GROUP) --query "properties.configuration.ingress.fqdn" --output tsv 2>/dev/null); \
	if [ -n "$$FQDN" ] && [ "$$FQDN" != "None" ]; then \
		URL="https://$$FQDN"; \
		if curl -s -f $$URL/agentcard > /dev/null; then \
			curl -s $$URL/agentcard | python3 -m json.tool; \
		elif curl -s -f $$URL/agent-card > /dev/null; then \
			curl -s $$URL/agent-card | python3 -m json.tool; \
		else \
			echo "Error: Agent card not found at $$URL/agentcard or $$URL/agent-card"; \
			exit 1; \
		fi; \
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
	@FQDN=$$(az containerapp show --name $(AZ_ACA_NAME) --resource-group $(AZ_RESOURCE_GROUP) --query "properties.configuration.ingress.fqdn" --output tsv 2>/dev/null); \
	if [ -n "$$FQDN" ] && [ "$$FQDN" != "None" ]; then \
		python3 tests/test_aca_validation.py https://$$FQDN; \
	else \
		echo "Error: Service not deployed or URL not found."; \
		exit 1; \
	fi

# Run local A2A echo test
a2a-local:
	@echo "Running local A2A echo test..."
	@python3 tests/echo_test.py http://localhost:8080

# Run remote A2A echo test
a2a-remote:
	@echo "Running remote A2A echo test..."
	@FQDN=$$(az containerapp show --name $(AZ_ACA_NAME) --resource-group $(AZ_RESOURCE_GROUP) --query "properties.configuration.ingress.fqdn" --output tsv 2>/dev/null); \
	if [ -n "$$FQDN" ] && [ "$$FQDN" != "None" ]; then \
		python3 tests/echo_test.py https://$$FQDN; \
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

update:
	@cargo update --verbose

# Build the Docker image
docker-build:
	@echo "Building the Docker image..."
	@docker build -t $(IMAGE_NAME) .

# --- Azure CLI Targets ---

az-login:
	@echo "Logging in to Azure..."
	@az login

# ACR related targets
acr-create:
	@echo "Ensuring Resource Group $(AZ_RESOURCE_GROUP) exists..."
	@az group create --name $(AZ_RESOURCE_GROUP) --location $(AZ_LOCATION)
	@echo "Checking if ACR $(AZ_ACR_NAME) exists..."
	@az acr show --name $(AZ_ACR_NAME) --resource-group $(AZ_RESOURCE_GROUP) > /dev/null 2>&1 || \
		(echo "Creating ACR $(AZ_ACR_NAME)..." && \
		az acr create --name $(AZ_ACR_NAME) --resource-group $(AZ_RESOURCE_GROUP) --sku Basic --admin-enabled true)

acr-login:
	@echo "Logging in to Azure Container Registry..."
	@az acr login --name $(AZ_ACR_NAME)

push-acr: docker-build acr-create acr-login
	@echo "Tagging and pushing the Docker image to ACR..."
	@docker tag $(IMAGE_NAME) $(FULL_IMAGE_NAME)
	@docker push $(FULL_IMAGE_NAME)

# --- Azure Container Apps (ACA) Deployment ---

deploy: check-requirements aca-deploy

aca-env-create: acr-create
	@echo "Checking if ACA Environment $(AZ_ACA_ENV_NAME) exists..."
	@az containerapp env show --name $(AZ_ACA_ENV_NAME) --resource-group $(AZ_RESOURCE_GROUP) > /dev/null 2>&1 || \
		(echo "Creating ACA Environment $(AZ_ACA_ENV_NAME)..." && \
		az containerapp env create --name $(AZ_ACA_ENV_NAME) --resource-group $(AZ_RESOURCE_GROUP) --location $(AZ_LOCATION))

aca-deploy: push-acr aca-env-create
	@echo "Deploying to Azure Container App..."
	@az containerapp up \
		--name $(AZ_ACA_NAME) \
		--resource-group $(AZ_RESOURCE_GROUP) \
		--environment $(AZ_ACA_ENV_NAME) \
		--image $(FULL_IMAGE_NAME) \
		--target-port 8080 \
		--ingress external \
		--env-vars ALLOWED_HOSTS=*
	@echo "Deployment complete. Endpoint: https://$$(az containerapp show --resource-group $(AZ_RESOURCE_GROUP) --name $(AZ_ACA_NAME) --query properties.configuration.ingress.fqdn -o tsv)"

# Target to get the public endpoint URL
endpoint:
	@az containerapp show --name $(AZ_ACA_NAME) --resource-group $(AZ_RESOURCE_GROUP) --query "properties.configuration.ingress.fqdn" --output tsv 2>/dev/null || echo "Container App $(AZ_ACA_NAME) not found."

# Fetch ACA logs
aca-logs:
	@echo "Fetching logs for $(AZ_ACA_NAME)..."
	@az containerapp logs show --name $(AZ_ACA_NAME) --resource-group $(AZ_RESOURCE_GROUP) --follow

# Alias for aca-logs
logs: aca-logs

# Destroy ACA resources
aca-destroy:
	@echo "Deleting Azure Container App $(AZ_ACA_NAME)..."
	@az containerapp delete --name $(AZ_ACA_NAME) --resource-group $(AZ_RESOURCE_GROUP) --yes 2>/dev/null || echo "Container App $(AZ_ACA_NAME) not found."
	@echo "Deleting ACA Environment $(AZ_ACA_ENV_NAME)..."
	@az containerapp env delete --name $(AZ_ACA_ENV_NAME) --resource-group $(AZ_RESOURCE_GROUP) --yes 2>/dev/null || echo "ACA Environment $(AZ_ACA_ENV_NAME) not found."
	@echo "Deleting ACR $(AZ_ACR_NAME)..."
	@az acr delete --name $(AZ_ACR_NAME) --resource-group $(AZ_RESOURCE_GROUP) --yes 2>/dev/null || echo "ACR $(AZ_ACR_NAME) not found."
	@echo "Deleting Resource Group $(AZ_RESOURCE_GROUP)..."
	@az group delete --name $(AZ_RESOURCE_GROUP) --yes --no-wait 2>/dev/null || echo "Resource Group $(AZ_RESOURCE_GROUP) not found."

# Alias for aca-destroy
destroy: aca-destroy

help:
	@echo "Makefile for a2a-server-rust on Azure"
	@echo ""
	@echo "Usage:"
	@echo "    make <target>"
	@echo ""
	@echo "Targets:"
	@echo "    all              (default) same as 'build'"
	@echo "    build            Build the project for development"
	@echo "    run              Run the project locally"
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
	@echo "    docker-build     Build the Docker image"
	@echo "    deploy           Push to ACR and deploy to ACA"
	@echo "    aca-status       Check ACA status"
	@echo "    endpoint         Fetch the ACA FQDN"
	@echo "    logs             Tail ACA logs"
	@echo "    destroy          Destroy all Azure resources in the group"
