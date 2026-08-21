
PACKAGE_NAME := address_book 
BIN_NAME := target/debug/$(PACKAGE_NAME)
SRC := $(wildcard src/**/*.rs src/*.rs)  

all: build

build:
	cargo build

run: build
	$(BIN_NAME)

release:
	cargo build --release
	./target/release/$(PACKAGE_NAME)

test:
	cargo test

lint:
	cargo clippy -- -D warnings

format:
	cargo fmt --all

clean:
	cargo clean

precommit: format lint test build

help:
	@echo "Makefile commands:"
	@echo "  make build       - Build the project in debug mode"
	@echo "  make run         - Run the project in debug mode"
	@echo "  make release     - Build and run in release mode"
	@echo "  make test        - Run tests"
	@echo "  make lint        - Run Clippy linter"
	@echo "  make format      - Format the code"
	@echo "  make clean       - Clean the build artifacts"
	@echo "  make precommit   - Format, lint, test, and build before committing"
	@echo "  make help        - Show this help message"

.PHONY: all build run release test lint format clean precommit help
