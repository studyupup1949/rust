.PHONY: credits fmt lint test check all rpm deb pkgs

# Generate CREDITS.html from dependencies
credits:
	cargo about generate about.hbs -o CREDITS.html

# Format code
fmt:
	cargo fmt

# Run clippy lints
lint:
	cargo clippy -- -D warnings

# Run tests
test:
	cargo test

# Run all checks
check: fmt lint test

# Generate credits and run all checks
all: credits check

# Build RPM package (requires cargo-generate-rpm: cargo install cargo-generate-rpm)
rpm:
	cargo build --release
	cargo generate-rpm

# Build DEB package (requires cargo-deb: cargo install cargo-deb)
deb:
	cargo build --release
	cargo deb

# Build all packages (RPM + DEB)
pkgs: rpm deb
