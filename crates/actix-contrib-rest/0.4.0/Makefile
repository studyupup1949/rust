.PHONY: clean build build-test all release test lint fmt-check docs
.DEFAULT_GOAL := all

clean:
	cargo clean

build:
	cargo build

all:
	cargo build --all-features

build-test:
	cargo build --all-features --tests

build-all: all build-test

release:
	cargo build --all-features --release

test:
	RUST_LOG=warn cargo test

lint:
	cargo clippy -- -D warnings

fmt-check:
	cargo fmt -- --check

docs:
	cargo doc --all-features --no-deps
