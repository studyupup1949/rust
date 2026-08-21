.ONESHELL:
SHELL := /bin/bash
all: help

################################################################################
# PROJECT CONFIG
PROJECT_NAME := adot

################################################################################
# APP
build: ## build the project
	cargo build

build-release: ## build the project in release mode
	cargo build --release

build-macos: ## cross-compile for macOS (arm64)
	cargo build --release --target aarch64-apple-darwin

build-linux: ## cross-compile for Linux (x86_64)
	cargo zigbuild --release --target x86_64-unknown-linux-gnu

run: ## run the project
	cargo run

clean: ## clean build artifacts
	cargo clean

################################################################################
# TEST
test: ## run all tests
	cargo test -- --test-threads=1

LLVM_TOOLCHAIN_BIN := $(shell rustup run stable rustc --print sysroot)/lib/rustlib/$(shell rustup run stable rustc -vV | grep host | cut -d' ' -f2)/bin
test-coverage: ## run tests with coverage report
	LLVM_COV=$(LLVM_TOOLCHAIN_BIN)/llvm-cov LLVM_PROFDATA=$(LLVM_TOOLCHAIN_BIN)/llvm-profdata cargo llvm-cov --html -- --test-threads=1
	LLVM_COV=$(LLVM_TOOLCHAIN_BIN)/llvm-cov LLVM_PROFDATA=$(LLVM_TOOLCHAIN_BIN)/llvm-profdata cargo llvm-cov report
	@echo ""
	@echo "=== Uncovered lines ==="
	LLVM_COV=$(LLVM_TOOLCHAIN_BIN)/llvm-cov LLVM_PROFDATA=$(LLVM_TOOLCHAIN_BIN)/llvm-profdata cargo llvm-cov report --show-missing-lines
	@echo ""
	@echo "=== Uncovered regions ==="
	@python3 scripts/coverage_regions.py

test-coverage-view: ## open coverage report in browser
	open -a "Google Chrome" target/llvm-cov/html/index.html

test-verbose: ## run all tests with verbose output
	cargo test -- --nocapture

################################################################################
# LINT
lint: ## run clippy linter
	cargo clippy -- -D warnings

lint-fix: ## run clippy with auto-fix
	cargo clippy --fix --allow-dirty -- -D warnings

format: ## format code
	cargo fmt

format-check: ## check code formatting
	cargo fmt -- --check

check: format-check lint test ## run all checks (format, lint, test)

################################################################################
# INSTALL
install: ## install the binary locally
	cargo install --locked --path .

################################################################################
# RELEASE
release: format check version-patch release-github release-cargo release-brew release-aur release-commit ## full release: bump + github + cargo + brew + aur + commit

release-github: build-macos build-linux ## create GitHub release with platform binaries
	@VERSION=$$(grep -m1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/'); \
	mkdir -p releases; \
	cp target/aarch64-apple-darwin/release/$(PROJECT_NAME) "releases/macos-$(PROJECT_NAME)-v$${VERSION}-arm64"; \
	cp target/x86_64-unknown-linux-gnu/release/$(PROJECT_NAME) "releases/linux-$(PROJECT_NAME)-v$${VERSION}-x86_64"; \
	echo "Binaries copied to releases/"; \
	if gh release view "v$$VERSION" >/dev/null 2>&1; then \
		echo "Release v$$VERSION already exists"; \
		read -p "Delete and recreate? [y/N] " -n 1 -r; \
		echo; \
		if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
			gh release delete "v$$VERSION" -y; \
		else \
			exit 1; \
		fi; \
	fi; \
	gh release create "v$$VERSION" \
		"releases/macos-$(PROJECT_NAME)-v$${VERSION}-arm64" \
		"releases/linux-$(PROJECT_NAME)-v$${VERSION}-x86_64" \
		--title "v$$VERSION" \
		--notes "$(PROJECT_NAME) v$$VERSION" \
		--latest; \
	echo "Released v$$VERSION"

release-cargo: ## publish to crates.io
	cargo publish

release-brew: ## update brew formula, commit and push to homebrew-tap
	@VERSION=$$(grep -m1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/'); \
	MACOS_URL="https://github.com/Dimfred/$(PROJECT_NAME)/releases/download/v$${VERSION}/macos-$(PROJECT_NAME)-v$${VERSION}-arm64"; \
	LINUX_URL="https://github.com/Dimfred/$(PROJECT_NAME)/releases/download/v$${VERSION}/linux-$(PROJECT_NAME)-v$${VERSION}-x86_64"; \
	echo "Fetching macOS SHA..."; \
	MACOS_SHA=$$(curl -sL "$$MACOS_URL" | shasum -a 256 | cut -d' ' -f1); \
	echo "macOS SHA256: $$MACOS_SHA"; \
	echo "Fetching Linux SHA..."; \
	LINUX_SHA=$$(curl -sL "$$LINUX_URL" | shasum -a 256 | cut -d' ' -f1); \
	echo "Linux SHA256: $$LINUX_SHA"; \
	sed -i '' "s|version \".*\"|version \"$$VERSION\"|" package/brew/$(PROJECT_NAME).rb; \
	sed -i '' "s|url \".*macos-.*\"|url \"$$MACOS_URL\"|" package/brew/$(PROJECT_NAME).rb; \
	sed -i '' "s|url \".*linux-.*\"|url \"$$LINUX_URL\"|" package/brew/$(PROJECT_NAME).rb; \
	sed -i '' "/macos-$(PROJECT_NAME)/{n;s|sha256 \".*\"|sha256 \"$$MACOS_SHA\"|;}" package/brew/$(PROJECT_NAME).rb; \
	sed -i '' "/linux-$(PROJECT_NAME)/{n;s|sha256 \".*\"|sha256 \"$$LINUX_SHA\"|;}" package/brew/$(PROJECT_NAME).rb; \
	echo "Formula updated for v$$VERSION"; \
	cp package/brew/$(PROJECT_NAME).rb ../homebrew-tap/Formula/; \
	cd ../homebrew-tap && git add . && git commit -m "Update $(PROJECT_NAME) to v$$VERSION" && git push; \
	echo "Pushed to homebrew-tap"

release-aur: ## update AUR PKGBUILD, commit and push to AUR
	@VERSION=$$(grep -m1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/'); \
	URL="https://github.com/Dimfred/$(PROJECT_NAME)/releases/download/v$${VERSION}/linux-$(PROJECT_NAME)-v$${VERSION}-x86_64"; \
	echo "Fetching $$URL"; \
	SHA=$$(curl -sL "$$URL" | shasum -a 256 | cut -d' ' -f1); \
	echo "SHA256: $$SHA"; \
	sed -i '' "s|^pkgver=.*|pkgver=$$VERSION|" package/aur/PKGBUILD; \
	sed -i '' "s|^sha256sums=.*|sha256sums=('$$SHA')|" package/aur/PKGBUILD; \
	echo "PKGBUILD updated for v$$VERSION"; \
	echo "Generating .SRCINFO..." && \
	printf '%s\n' \
		"pkgbase = $(PROJECT_NAME)-bin" \
		"	pkgdesc = A minimal dotfile manager" \
		"	pkgver = $$VERSION" \
		"	pkgrel = 1" \
		"	url = https://github.com/Dimfred/$(PROJECT_NAME)" \
		"	arch = x86_64" \
		"	license = MIT" \
		"	provides = $(PROJECT_NAME)" \
		"	conflicts = $(PROJECT_NAME)" \
		"	source = $(PROJECT_NAME)-bin-$$VERSION::https://github.com/Dimfred/$(PROJECT_NAME)/releases/download/v$$VERSION/linux-$(PROJECT_NAME)-v$$VERSION-x86_64" \
		"	sha256sums = $$SHA" \
		"" \
		"pkgname = $(PROJECT_NAME)-bin" > package/aur/.SRCINFO; \
	cp package/aur/PKGBUILD ../$(PROJECT_NAME)-bin/PKGBUILD; \
	cp package/aur/.SRCINFO ../$(PROJECT_NAME)-bin/.SRCINFO; \
	cd ../$(PROJECT_NAME)-bin && git add PKGBUILD .SRCINFO && git commit -m "Update to v$$VERSION" && git push; \
	echo "Pushed to AUR"

release-commit: ## commit and push package updates
	@VERSION=$$(grep -m1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/'); \
	git add package/; \
	git commit --amend --no-edit; \
	git push; \
	echo "Pushed release v$$VERSION"

version-patch: ## bump patch version (0.1.0 -> 0.1.1)
	@CURRENT=$$(grep -m1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/'); \
	IFS='.' read -r MAJOR MINOR PATCH <<< "$$CURRENT"; \
	NEW_PATCH=$$((PATCH + 1)); \
	NEW_VERSION="$$MAJOR.$$MINOR.$$NEW_PATCH"; \
	sed -i '' "s/^version = \".*\"/version = \"$$NEW_VERSION\"/" Cargo.toml; \
	cargo generate-lockfile; \
	echo "Version bumped from $$CURRENT to $$NEW_VERSION"; \
	git add Cargo.toml Cargo.lock; \
	git commit -m "chore: bump v$$NEW_VERSION"; \
	echo "Committed version bump"

################################################################################
# INIT
init: ## initialize dev environment (deps, rust targets, zigbuild)
	cargo fetch
	brew install zig@0.14
	rustup target add aarch64-apple-darwin x86_64-unknown-linux-gnu
	cargo install cargo-zigbuild
	cargo install cargo-llvm-cov
	rustup component add llvm-tools-preview

################################################################################
# HELP
help: ## print this help
	@grep -E '^[a-zA-Z0-9_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%-30s\033[0m %s\n", $$1, $$2}'
