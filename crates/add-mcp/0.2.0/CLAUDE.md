# add-mcp

Rust library + CLI for installing MCP servers into 10 AI client configurations. Published as `add-mcp` on crates.io.

## Project Identity

- **Package**: `add-mcp` (Cargo.toml)
- **Directory**: `rust-add-mcp`
- **Remote**: `git@gitlab.com:ricardo.fgusmao/rust-add-mcp.git`
- **Binary**: `target/release/add-mcp`
- **Version**: 0.1.0

## What It Does

Single function call or CLI command installs an MCP server entry into any combination of 10 AI client config files. Handles three source types (command, URL, npm package), three config formats (JSON, YAML, TOML), and per-agent config shape differences.

## Architecture

### Core Flow

```
Source string → parse_source() → Source enum
Source + agent → transform() → serde_json::Value (agent-specific shape)
read_config() → merge_server() → write_config()
```

All formats normalize to `serde_json::Value` internally. No async — pure sync file I/O.

### File Layout

```
src/
  lib.rs              Public API: install_command, install_url, install, detect_agents
  main.rs             CLI binary (clap, behind "cli" feature)
  error.rs            AddMcpError via thiserror (8 variants)
  types.rs            Agent (10 variants), Scope, Transport, Source, McpServerConfig, InstallResult
  source.rs           Source parsing (URL/npm/command) + name inference
  agent.rs            AgentDef registry: section_key, format, has_local per agent
  paths.rs            Config path resolution per agent/scope (#[cfg(target_os)])
  transform.rs        Per-agent config shape transforms
  install.rs          Read → transform → merge → write engine
  detect.rs           Detect installed agents by config file presence
  config/
    mod.rs            ConfigFormat dispatch + merge_server
    json.rs           JSON read/write (serde_json)
    yaml.rs           YAML read/write (serde_yaml → serde_json::Value)
    toml_config.rs    TOML read/write (toml → serde_json::Value)
tests/
  integration_tests.rs   15 tests covering all agents, formats, and edge cases
```

### 10 Supported Clients

| Agent | Section Key | Format | Global Path | Local Path |
|-------|-----------|--------|-------------|------------|
| Claude Code | `mcpServers` | JSON | `~/.claude.json` | `.mcp.json` |
| Claude Desktop | `mcpServers` | JSON | `~/.config/Claude/claude_desktop_config.json` | — |
| Codex | `mcp_servers` | TOML | `~/.codex/config.toml` | `.codex/config.toml` |
| Cursor | `mcpServers` | JSON | `~/.cursor/mcp.json` | `.cursor/mcp.json` |
| Gemini CLI | `mcpServers` | JSON | `~/.gemini/settings.json` | `.gemini/settings.json` |
| Goose | `extensions` | YAML | `~/.config/goose/config.yaml` | — |
| GitHub Copilot | `mcpServers` | JSON | `~/.copilot/mcp-config.json` | `.vscode/mcp.json` |
| OpenCode | `mcp` | JSON | `~/.config/opencode/opencode.json` | `opencode.json` |
| VS Code | `servers` | JSON | `~/.config/Code/User/mcp.json` | `.vscode/mcp.json` |
| Zed | `context_servers` | JSON | `~/.config/zed/settings.json` | `.zed/settings.json` |

### Agent-Specific Transform Shapes

- **Standard** (Claude Code, Claude Desktop, Cursor, Gemini CLI): `{ "command": "...", "args": [...] }`
- **Codex**: Same as standard but written to TOML
- **Goose**: `{ "type": "stdio", "cmd": "...", "args": [...], "enabled": true }`
- **GitHub Copilot**: Standard + `"tools": ["*"]`
- **OpenCode**: `{ "command": ["cmd", "arg1", ...], "type": "stdio" }`
- **VS Code**: Standard + `"type": "stdio"`
- **Zed**: `{ "source": "custom", "command": { "path": "...", "args": [...] } }`

## Key Types

- `Agent` — enum with 10 variants, `from_str_loose()` for flexible parsing, `ALL` constant
- `Scope` — Global or Local
- `Source` — Command { command, args } | Url { url, transport } | NpmPackage { package }
- `McpServerConfig` — name, source, env, headers
- `InstallResult` — agent, scope, path, created, already_existed
- `AgentDef` — section_key, format, has_local (static config per agent)
- `AddMcpError` — thiserror enum with `Result<T>` alias

## Library API (primary use case)

```rust
use add_mcp::{install_command, Agent, Scope};

// Self-install from another Rust MCP server
let binary = std::env::current_exe()?;
install_command("my-server", binary.to_str().unwrap(), &[], &[Agent::ClaudeCode], Scope::Global);
```

Consume as library only (no CLI deps): `add-mcp = { version = "0.1", default-features = false }`

## CLI Interface

```
add-mcp install <source> [-g] [-a agent]... [-n name] [-t sse|http] [--header K:V]... [-e K=V]... [-y] [-- extra-args...]
add-mcp list-agents
add-mcp detect [--local]
```

## Tech Stack

- **clap v4** derive (optional, behind `cli` feature)
- **serde** + **serde_json** + **serde_yaml** + **toml** for multi-format config I/O
- **dirs v6** for cross-platform home/config directory resolution
- **thiserror v2** for error types
- **url v2** for URL parsing in source/name inference

## Build & Test

```bash
cargo build --release
cargo test                                    # 32 tests (17 unit + 14 integration + 1 doc)
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Design Decisions & Patterns

1. **All formats → serde_json::Value**: YAML and TOML are converted to/from JSON Value for uniform merge logic
2. **`_with_home()` variants**: `install_with_home()`, `detect_with_home()`, `config_path_with_home()` accept explicit home dir for deterministic parallel tests (avoids env var races)
3. **Merge preserves existing config**: Reads full file, merges under section key, writes back — other keys are untouched
4. **`merge_server()` returns bool**: Indicates whether the server name already existed (overwritten vs. added)
5. **Source heuristics**: `http://`/`https://` → URL, starts with `@` → npm, path exists on disk → command, otherwise bare name → npm
6. **npm wrapping**: npm packages become `{ "command": "npx", "args": ["-y", "<package>"] }`
7. **No async**: Config files are tiny; sync I/O is simpler and has no runtime dependency

## Relationship to mcp-watch

`mcp-watch` will depend on `add-mcp` (with `default-features = false`) to self-install:

```toml
[dependencies]
add-mcp = { version = "0.1", default-features = false }
```

## Live Validation

`scripts/live-validate.sh` — interactive shell script that tests add-mcp against real AI client configs. Installs a dummy MCP server into each client, verifies the config file, then cleans up. Use before publishing to crates.io.

```bash
bash scripts/live-validate.sh
```

## CI

GitLab CI (`.gitlab-ci.yml`): fmt check → clippy → test, with cargo caching.
