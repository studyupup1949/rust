# cargo-a9-lint

An opinionated Rust style linter. Runs as a Cargo subcommand.

## Install

```sh
cargo install cargo-a9-lint
```

## Usage

```sh
# From your project root
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

| Rule | Description |
|---|---|
| `cfg-order` | cfg-gated use items must come after unconditional ones, ordered by complexity |
| `item-order` | Top-level items must follow the order: extern crate → use → mod (declaration-only) → everything else |
| `no-allow-unused-imports` | Forbids #[allow(unused_imports)]; remove the unused import instead |
| `normalized-use-stmt` | Use statements sharing the same root must be merged; duplicate root segments are flagged |
| `path-depth` | Flags redundant path prefixes on already-imported names |
| `use-group-order` | use groups must be ordered: std → external crates → crate/self |
| `use-toplevel` | use items inside blocks are forbidden |

## License

MIT
