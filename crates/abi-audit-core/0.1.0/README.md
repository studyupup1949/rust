# cargo-abi-audit

`cargo abi-audit` is a Cargo-native CLI for auditing Rust FFI and ABI boundaries.

This repository ships a phase-4, C-ABI-first MVP. It is designed for Rust crates that intentionally expose a C-facing surface through `extern "C"` or `extern "system"` functions, headers, and `cdylib`/`staticlib` packaging. It is not a blanket verifier for arbitrary C++ ABI compatibility.

The current MVP covers a truthful source/header/build slice toward the long-term product:

- discover outbound Rust exports from source, including nested inline modules,
- inventory FFI-relevant public Rust types and their `repr` usage,
- inspect configured or auto-discovered public C headers,
- build target libraries and capture compiled artifact paths during `snapshot` / `check`,
- inspect compiled `cdylib` exports with host-native symbol tooling when available,
- record explicit cbindgen header-sync workflows and optionally check freshness by file evidence,
- emit a normalized JSON snapshot with canonical signature projections,
- resolve baselines from either a snapshot file or an extracted baseline artifact directory,
- emit SARIF sidecars for CI/code-scanning upload,
- package a composite GitHub Action around the current `check` CLI surface,
- report concrete policy findings in human-readable or JSON form,
- optionally compare the current snapshot with a stored baseline snapshot.

## Commands

```text
cargo abi-audit init
cargo abi-audit snapshot
cargo abi-audit check
```

The binary name is `cargo-abi-audit`, so Cargo dispatch works through `cargo abi-audit ...`.

## Current scope

Implemented in this phase:

- `extern "C"` / `extern "system"` export discovery
- export attribute discovery for `#[no_mangle]` and `#[export_name = "..."]`
- public type inventory for structs, enums, unions, and type aliases that matter to the boundary
- recursive by-value checks for `repr(C)` aggregates, `repr(transparent)` wrappers, and fieldless enums with explicit integer reprs
- basic opaque-handle recognition for raw pointers to local Rust types
- normalized header signature matching for a stable C subset, including simple typedef aliases
- package/header auto-discovery when config is omitted or a target leaves `headers` empty
- crate target metadata checks for `cdylib` / `staticlib`
- JSON snapshot generation and baseline diffing

Explicitly not implemented yet:

- C++ ABI compatibility analysis
- full C parsing or full header/source semantic equivalence
- guaranteed staticlib export truth or archive member interpretation
- cbindgen execution or round-trip header guarantees
- inbound FFI surface auditing

## Quick start

Initialize a config in a target repository:

```bash
cargo run -p cargo-abi-audit-cli -- init
```

The starter config documents the simplest path: point at a package and either list headers explicitly or leave `headers` empty to auto-discover `include/**/*.h` under that package. If you already generate headers with cbindgen, you can also declare a `header_sync` block so the snapshot captures the expected output path, config file, and workflow hint.

For CI, baseline inputs stay explicit on purpose. Supported modes in this phase are:

- a direct snapshot JSON file
- an extracted artifact directory containing a snapshot JSON file, configured through `[baseline] kind = "artifact_dir"`

The CLI does not fetch git tags, releases, or artifacts by itself.

Emit a normalized snapshot:

```bash
cargo run -p cargo-abi-audit-cli -- snapshot --manifest-path Cargo.toml
```

Run checks in human-readable form:

```bash
cargo run -p cargo-abi-audit-cli -- check --manifest-path Cargo.toml
```

Run checks in JSON:

```bash
cargo run -p cargo-abi-audit-cli -- check --manifest-path Cargo.toml --format json
```

Emit SARIF alongside the normal human-readable output:

```bash
cargo run -p cargo-abi-audit-cli -- check --manifest-path Cargo.toml --sarif-output abi-audit/results.sarif
```

Override the baseline input with an extracted artifact directory:

```bash
cargo run -p cargo-abi-audit-cli -- check \
  --manifest-path Cargo.toml \
  --baseline abi-audit/release-baseline \
  --baseline-snapshot snapshot.json
```

## CI integration

The repository root now ships a composite GitHub Action in [`action.yml`](action.yml).

Use it after your workflow has already checked out the audited repository and materialized any configured baseline file or directory:

```yaml
- uses: <owner>/abi-audit@<ref>
  with:
    manifest-path: Cargo.toml
    config: abi-audit.toml
    baseline: abi-audit/release-baseline
    baseline-snapshot: snapshot.json
    sarif-output: abi-audit/results.sarif
```

See [docs/ci.md](docs/ci.md) for the supported baseline modes, rule severity overrides, and example baseline-publishing / PR-check workflows.

## Repository guide

- [docs/architecture.md](docs/architecture.md)
- [docs/ci.md](docs/ci.md)
- [docs/rules.md](docs/rules.md)
- [BUILD.md](BUILD.md)
- [fixtures/outbound-c-api](fixtures/outbound-c-api)
- [fixtures/auto-discovery-ffi](fixtures/auto-discovery-ffi)
