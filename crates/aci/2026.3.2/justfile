set shell := ["bash", "-cu"]

fmt:
  cargo fmt --all

fmt-check:
  cargo fmt --all -- --check

clippy:
  cargo clippy --all-targets --all-features -- -D warnings

test:
  cargo test --all --all-features

build:
  cargo build --release --all-features

check: fmt-check clippy test build

release-check:
  cargo test --all --all-features
  cargo build --release --all-features
  cargo publish --dry-run --locked

release: release-check
  @bash -ceu 'set -euo pipefail; \
    version="$(grep -m1 "^version = " Cargo.toml | cut -d "\"" -f2)"; \
    tag="v${version}"; \
    if git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; then \
      echo "tag already exists locally: ${tag}" >&2; \
      exit 1; \
    fi; \
    if git ls-remote --tags origin "refs/tags/${tag}" | grep -q .; then \
      echo "tag already exists on origin: ${tag}" >&2; \
      exit 1; \
    fi; \
    git tag "${tag}"; \
    git push origin "${tag}"'
