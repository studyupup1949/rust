#!/usr/bin/env bash
set -euo pipefail

echo "[check] cargo fmt --all -- --check"
cargo fmt --all -- --check

echo "[check] cargo clippy --all-targets --no-default-features -- -D warnings"
cargo clippy --all-targets --no-default-features -- -D warnings

echo "[check] cargo clippy --all-targets --all-features -- -D warnings"
cargo clippy --all-targets --all-features -- -D warnings

echo "[check] cargo test --no-default-features"
cargo test --no-default-features

echo "[check] cargo test --all-features"
cargo test --all-features

echo "[check] cargo check --all-features --examples"
cargo check --all-features --examples
