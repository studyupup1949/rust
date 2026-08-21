# cargo-advisor

`cargo-advisor` is the decision layer for Cargo: explainable crate recommendation, comparison, explanation, and local dependency review with explicit receipts.

The project is local-first. It keeps the evidence boundary narrow on purpose:

- `recommend` picks from a curated, checked-in catalog by intent and goal.
- `compare` explains tradeoffs across named crates in that catalog.
- `explain` turns one crate into a recommendation narrative.
- `review` inspects local `Cargo.toml` and `Cargo.lock` files for overlapping dependency decisions.
- Every result stays explanation-first and includes confidence, tradeoffs, trust notes, and receipts.

This project uses `review`, not `audit`. It does not fake live crates.io, docs.rs, RustSec, download, or maintenance evidence. If a source is not consulted, the output should say so.

## Install

Install from crates.io:

```bash
cargo install cargo-advisor
```

For local development, you can still install from the workspace path:

```bash
cargo install --path crates/cargo-advisor-cli --locked
```

That installs the `cargo-advisor` binary as a Cargo subcommand, so you can run:

```bash
cargo advisor recommend cli --goal "small binary"
```

You can also run it from the workspace without installing:

```bash
cargo run -p cargo-advisor -- recommend cli --goal "small binary"
```

## Usage

Examples:

```bash
cargo advisor recommend cli-parsing
cargo advisor recommend cli --goal "fastest to ship"
cargo advisor recommend http-client --goal blocking --format json
cargo advisor compare reqwest ureq hyper --for http-client
cargo advisor explain thiserror --for error-handling
cargo advisor review --manifest-path ./Cargo.toml --lockfile ./Cargo.lock
```

Current intent coverage:

- `cli-parsing`
- `config`
- `logging-tracing`
- `http-client`
- `http-server`
- `serialization`
- `async-runtime`
- `error-handling`
- `testing`
- `database-access`

## Output Contract

The default renderer is text, but `--format json` exposes the same explanation-first contract in machine-readable form. Shared concepts across commands:

- `summary`
- `recommendation`
- `confidence`
- `tradeoffs`
- `trust_notes`
- `receipts`

Command-specific JSON fixtures are checked in under `tests/fixtures/contracts/` as release/reference fixtures for the explanation-first output contract.

The current contract is documented in [docs/v0.1-plan.md](/Users/sawyer/github/cargo-advisor/docs/v0.1-plan.md).

## Workspace

The workspace currently has five crates:

- `crates/cargo-advisor-cli`: binary and command parser
- `crates/advisor-core`: shared report models and decision logic
- `crates/advisor-catalog`: curated intent catalog
- `crates/advisor-evidence`: local evidence loading and receipts
- `crates/advisor-output`: text and JSON rendering

## Verification

Phase planning lives in [BUILD.md](/Users/sawyer/github/cargo-advisor/BUILD.md). CI mirrors the local verification bar in [ci.yml](/Users/sawyer/github/cargo-advisor/.github/workflows/ci.yml):

```bash
cargo fmt --all --check
cargo test --workspace
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Release Hygiene

- Dual-licensed under `MIT OR Apache-2.0`; see [LICENSE-MIT](/Users/sawyer/github/cargo-advisor/LICENSE-MIT) and [LICENSE-APACHE](/Users/sawyer/github/cargo-advisor/LICENSE-APACHE).
- Release notes live in [CHANGELOG.md](/Users/sawyer/github/cargo-advisor/CHANGELOG.md).
- The local release checklist lives in [RELEASE.md](/Users/sawyer/github/cargo-advisor/RELEASE.md).

## Design Boundaries

- Recommendations stay deterministic and checked into source.
- Review findings are heuristic and local-only.
- External evidence integrations remain future work until their receipts and trust boundaries are visible in output.
