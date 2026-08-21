.PHONY: check fmt clippy test test-all bench audit deny coverage doc build clean

# Run all CI checks locally
check: fmt clippy test audit

# Format check
fmt:
	cargo fmt --all -- --check

# Lint (zero warnings)
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

# Run core tests
test:
	cargo test

# Run tests with all features
test-all:
	cargo test --all-features

# Run benchmarks (criterion)
bench:
	cargo bench --bench benchmarks

# Security audit
audit:
	cargo audit

# Supply-chain checks (license + advisory + source)
deny:
	cargo deny check

# Code coverage
coverage:
	cargo tarpaulin --all-features --skip-clean

# Generate documentation (warnings as errors)
doc:
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

# Build release
build:
	cargo build --release --all-features

# Clean build artifacts
clean:
	cargo clean
	rm -rf coverage/
