# aci
<!-- bdg:begin -->
[![crates.io](https://img.shields.io/crates/v/aci.svg)](https://crates.io/crates/aci)
[![license](https://img.shields.io/github/license/f4ah6o/aci-rs.svg)](https://github.com/f4ah6o/aci-rs)
[![CI](https://github.com/f4ah6o/aci-rs/actions/workflows/publish.yaml/badge.svg)](https://github.com/f4ah6o/aci-rs/actions/workflows/publish.yaml)
<!-- bdg:end -->

Rust implementation of "Mount APIs as CLIs".

`aci` mounts HTTP APIs as command-line interfaces in two ways:
- generic raw fetch mode (`aci call ...`)
- config-driven mount mode (`aci --config aci.toml <mount> ...`)

It also includes a coding-agent-oriented quick guide:
- `aci skills`

## Install

```bash
cargo install --path .
```

## Trusted Publishing

This repository is configured for [crates.io trusted publishing](https://crates.io/docs/trusted-publishing).

Create and push a `v<version>` tag to trigger publish:

```bash
just release
```

or:

```bash
git tag v2026.3.0
git push origin v2026.3.0
```

Before first publish, enable trusted publishing for crate `aci` in crates.io and link `f4ah6o/aci-rs`.

## Quick Start

### Help

```bash
aci help
aci --help
```

### 0) Coding agent guide

```bash
aci skills
```

### 1) Generic API client mode

```bash
aci call --url https://api.github.com repos rust-lang rust \
  -H "Accept: application/vnd.github+json" \
  -H "User-Agent: aci"
```

### 2) Config-driven mode (`aci.toml`)

```toml
name = "aci"

[[mounts]]
name = "github"
kind = "remote"
base_url = "https://api.github.com"
```

```bash
aci --config aci.toml github repos rust-lang rust \
  -H "Accept: application/vnd.github+json" \
  -H "User-Agent: aci"
```

## Raw Fetch Flags

- `-X, --method <METHOD>`
- `-H, --header "Key: Value"` (repeatable)
- `-d, --data <json>`
- `--body <json>`
- `--query key=value` (repeatable)
- unknown `--key value` is treated as query for compatibility

## Auth / Token Example

```bash
aci call --url https://api.github.com user \
  -H "Accept: application/vnd.github+json" \
  -H "User-Agent: aci" \
  -H "Authorization: Bearer $GITHUB_TOKEN"
```

## OpenAPI Mounts

Mount with OpenAPI by setting `openapi` in `aci.toml`:

```toml
name = "aci"

[[mounts]]
name = "pet"
kind = "remote"
base_url = "https://petstore3.swagger.io"
base_path = "/api/v3"
openapi = "./openapi.json"
timeout_ms = 10000
```

Behavior:
- `operationId` becomes command name
- missing `operationId` falls back to `method_path` name
- path params are positional args
- query/body params are `--option value`
- OpenAPI mounts always have `raw` subcommand fallback

Example:

```bash
aci --config aci.toml pet findPetsByStatus --status available
aci --config aci.toml pet raw pet findByStatus --query status=available
```

## Exit Codes

- `0`: success
- `1`: upstream/API execution failure
- `2`: usage/config/input error


