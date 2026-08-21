# cargo-a9-lint

An opinionated Rust style linter. Runs as a Cargo subcommand.

## Install

```sh
cargo install cargo-a9-lint
```

## Usage

```sh
# From your project root — fix mode (default)
cargo a9-lint

# Check-only mode (no modifications)
cargo a9-lint check
```

## Configuration

Add a `[workspace.metadata.a9-lint]` section to your root `Cargo.toml`:

```toml
[workspace.metadata.a9-lint]
scan = ["src", "other-crate/src"]

# Optionally disable specific rules:
# [workspace.metadata.a9-lint.rules]
# disable = ["path-depth"]
```

If no config is found, `a9-lint` walks upward from the current directory looking for a `Cargo.toml` with this section. If none is found, it falls back to scanning `src/` in the current directory.

## Rules

Rules are grouped by feature. Base rules are always active. Theta rules require opting in.

### Pre-pipeline Predicates

These checks run on the raw source text before the parse→unparse pipeline.

| Rule | Description |
|---|---|
| `no-comment` | Plain comments are stripped by the formatter; use doc-comments or self-explanatory code |

### Base Rules

| Rule | Description |
|---|---|
| `cfg-order` | cfg-gated use items must come after unconditional ones, ordered by complexity |
| `module-item-order` | top-level items must follow group order with pub before private, topologically sorted within groups |
| `no-allow-unused-imports` | Forbids #[allow(unused_imports)]; remove the unused import instead |
| `normalized-use-stmt` | Use statements sharing the same root must be merged; duplicate root segments are flagged |
| `path-depth` | Flags redundant path prefixes on already-imported names |
| `optimal-control-flow` | the lighter branch should guard; prefer early exit when the then-body dominates |
| `scope-ident-length-correspondance` | variable name length should correspond to its scope size |
| `use-group-order` | use groups must be ordered: std → external crates → crate/self |
| `use-order` | Top-level items must follow the order: extern crate → use → mod (declaration-only) → everything else |
| `use-toplevel` | use items inside blocks are forbidden |

### Theta Rules

Rules for codebases using the [theta](https://github.com/ars-vivendi/theta) actor framework. Enable by adding `features = ["theta"]` to your workspace metadata:

```toml
[workspace.metadata.a9-lint]
features = ["theta"]
```

| Rule | Description |
|---|---|
| `theta-actors-at-bottom` | `#[actor]` impl blocks must all appear at the bottom; no non-actor item may follow one |
| `theta-actor-fields-gated` | all fields of the actor struct must be gated with `#[cfg(feature = "private")]` |
| `theta-actor-private-gate` | declaration-only `mod private;` must be gated with `#[cfg(feature = "private")]` |
| `theta-no-private-type-leak` | actor message return types must not expose `private::` module types; define a public DTO |

## License

MIT
