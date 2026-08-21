set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
  @just --list

fmt:
  cargo fmt --all

fmt-check:
  cargo fmt --all -- --check

clippy:
  cargo clippy --all-targets --all-features -- -D warnings

test:
  cargo test --all-features

check:
  bash scripts/check.sh

fix: fmt

install-hooks:
  bash scripts/install-git-hooks.sh

doc:
  cargo doc --all-features --no-deps

# Publish: bump version, commit, tag, push (triggers CI publish)
release version:
  #!/usr/bin/env bash
  set -euo pipefail
  current=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
  echo "Current version: $current"
  echo "New version:     {{version}}"
  read -p "Confirm? [y/N] " -n 1 -r
  echo
  [[ $REPLY =~ ^[Yy]$ ]] || exit 1
  sed -i '' 's/^version = ".*"/version = "{{version}}"/' Cargo.toml
  cargo check --all-features
  git add Cargo.toml
  git commit -m "chore: release v{{version}}"
  git tag "v{{version}}"
  echo "Done. Run 'git push origin main v{{version}}' to publish."
