#!/bin/bash
# Setup a minimal workspace context for building a3s-flow standalone.
# Restructures: ./ = repo root → ./ = workspace root with crates/flow/
set -euo pipefail

CRATE_NAME="flow"

TMPDIR="$(mktemp -d)"
cp -a . "$TMPDIR/$CRATE_NAME"
find . -maxdepth 1 ! -name '.' ! -name '.git' -exec rm -rf {} +
mkdir -p crates
cp -a "$TMPDIR/$CRATE_NAME/." "crates/$CRATE_NAME/"

cat > Cargo.toml << 'EOF'
[workspace]
resolver = "2"
members = [
    "crates/flow",
    "crates/flow/sdk/python",
    "crates/flow/sdk/node",
]

[workspace.package]
version = "0.3.4"
edition = "2021"
license = "MIT"

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
async-trait = "0.1"
EOF

rm -rf "$TMPDIR"
echo "Workspace restructured. Crate at: crates/$CRATE_NAME/"
