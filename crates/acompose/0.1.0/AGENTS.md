# acompose

Agent composition orchestrator built in Rust using the Agent Client Protocol (ACP).

## Goal

Provide a single binary that can:

1. Spawn persistent Kimi Code agents (`kimi acp`) in arbitrary working directories.
2. Send each spawned agent an initial system message (the "charter") that tells it how to behave, what crons to set up, what skills to activate, etc.
3. Keep the agents alive so they can receive follow-up prompts from the orchestrator or from other agents.
4. Allow the main agent to spin up sub-agents (an ensemble) and delegate tasks to them.

## Why ACP?

ACP (Zed's Agent Client Protocol) lets a client drive a long-running agent over stdio/HTTP. Each `kimi acp` process is an ACP Agent server. The orchestrator is an ACP Client that:

- Spawns the process.
- Calls `initialize` to negotiate capabilities.
- Calls `session/new` in the target working directory.
- Calls `session/prompt` with the charter.
- Later routes further prompts or inter-agent messages via `session/prompt`.

Because the agent process stays alive, it is truly persistent: its session state survives across individual prompts (unlike one-shot CLI invocations).

## Current state

### Implemented

- Rust project in `~/work/acompose`.
- Config loading from `acompose.toml` (`kimi_binary`, list of sessions with `name`, `cwd`, `charter`).
- `src/acp_client.rs`: spawns `kimi acp`, runs ACP handshake, creates session, sends charter, auto-approves permission requests.
- `src/main.rs`: loads config, starts all sessions concurrently, keeps the orchestrator alive until `SIGINT`/`SIGTERM` so agent processes stay connected.
- `src/orchestrator.rs`: shared session registry; tracks active sessions and can create/list/send messages to them. Sessions are registered immediately after `session/new`/`session/resume` and carry a lifecycle status (`initializing` / `ready` / `error`).
- `src/state.rs`: persistent `state.json` mapping `session.name -> session_id`; loaded on startup and updated when sessions are created.
- `src/mcp_server.rs`: HTTP/SSE MCP server (rmcp) exposing compose tools on `http://127.0.0.1:19094/mcp` by default.
- `process-compose.yaml` + `.pc_env`: run acompose under process-compose on port 19093 for persistence, restarts, and log inspection.
- `.mcp.json`: exposes process-compose control plane as MCP tools at `http://localhost:19093/sse`.
- Symlink `~/work/kimi-code/acompose -> ~/work/acompose` so the project is reachable from the main working session.

### Verified

```bash
cd ~/work/acompose
process-compose up -D --env .pc_env
process-compose -p 19093 process logs acompose
```

Logs show successful `initialize` → `session/new` → `session/prompt(charter)` → `EndTurn`, then the orchestrator keeps the connection alive.

The integrated MCP server is available at `http://127.0.0.1:19094/mcp` (configurable; see `.mcp.json` for an example client config) and exposes:

- `compose_list_sessions()` — list active sessions with lifecycle status.
- `compose_create_session(name, cwd, charter, allowed_tool_kinds = [])` — spawn and charter a new agent. Returns immediately while the charter prompt runs in the background.
- `compose_send_message(target, content, wait = true, timeout_ms = null)` — send a follow-up prompt.
  - `wait=true` and no timeout: block until the agent finishes its turn.
  - `wait=true` with `timeout_ms`: block up to the timeout; if the turn is not done, return `{"status":"pending","prompt_id":"..."}`.
  - `wait=false`: return `{"status":"pending","prompt_id":"..."}` immediately.
- `compose_get_prompt_result(prompt_id)` — poll the result of a pending/background prompt. Returns the full `PromptJob` (target, content, status, result/error).

## Next tasks

### 1. Session persistence across restarts ✅

Implemented: `state.json` is stored next to `acompose.toml`. On startup, if a persisted `session_id` exists, acompose tries `session/resume`; if that fails, it falls back to `session/new` and updates `state.json`. `session/load` is still available in the protocol but not currently used.

Note: `session/resume` works only if the new `kimi acp` process can restore the session history. Whether Kimi Code CLI persists history across process restarts is an implementation detail of `kimi acp`; acompose itself keeps only the `name -> session_id` mapping.

### 2. Call strategy and context-overflow handling

Different agent workloads need different lifecycle strategies. Add configurable per-session (or per-call) behavior for:

- **Undo / reset after each call** — ACP has no native "rewind last turn" method. The practical options are:
  - `session/delete` + `session/new` with the same charter (start from a clean slate).
  - `session/close` + `session/resume` if the agent supports it.
  - Keep a snapshot of the charter and, on demand, recreate the session.
- **Forking (unstable)** — ACP has `session/fork` behind `sessionCapabilities.fork`. It creates a new session based on the context of an existing one without affecting the original. Useful for "what-if" branches.
- **Creating with known turns** — ACP does not allow injecting message history into `session/new`. The practical workaround is:
  - Create a new session.
  - Send a single reconstructed prompt that contains the compacted history you want to preserve.
  - Or use `session/fork` if the agent supports it.
- **Context-window monitoring** — ACP sends `UsageUpdate` notifications with `used` and `size` tokens. Track the ratio and trigger an action before the agent hits the limit:
  - Create a fresh session and continue the task there.
  - Use `session/fork` and continue in the fork.
  - Use `session/load` with a truncated history if the agent supports it.
  - Let the agent compact automatically, but only if it advertises a compaction capability.
- **Configurable policy** — e.g. `after_each_call = "reset"`, `context_limit_policy = "new_session"`, `context_threshold = 0.8`.

### 3. Session fork / snapshot tools

Expose fork and snapshot behavior through MCP:

- `compose_fork_session(name_or_id, new_name)` — call `session/fork` if supported, register the fork under `new_name`.
- `compose_snapshot_session(name_or_id)` — return the charter + last N user/agent message pairs so the caller can reconstruct history later.

### 3. Expose compose tools to spawned agents

`kimi acp` does **not** load project `.mcp.json`. It only exposes built-in tools and user-scope MCP servers (e.g. `context7-mcp`). To give spawned agents access to the acompose MCP server, pass it explicitly in `NewSessionRequest.mcp_servers`:

```rust
request.mcp_servers.push(McpServer::Http(McpServerHttp::new(
    "acompose",
    "http://127.0.0.1:19094/mcp",
)));
```

Then agents can call `compose_list_sessions`, `compose_create_session`, etc. to spawn sub-agents and talk to siblings.

### 4. Tool / permission sandboxing for spawned agents

Spawned `kimi acp` agents inherit the full Kimi Code CLI built-in toolset (`Bash`, `Read`, `Write`, `Edit`, `WebSearch`, etc.) plus user-scope MCP servers. There is currently no CLI flag or config option to disable individual built-in tools.

Implemented:

- **Permission-level allowlist** — `SessionConfig.allowed_tool_kinds` is enforced in `acp_client.rs`. If the agent sends a `RequestPermissionRequest` for a disallowed `ToolKind`, the orchestrator responds with `Cancelled`.

Known limitation:

- `kimi acp` does **not** ask for permission before executing `Bash`/`Execute` tools; it only reports them as `SessionUpdate` notifications. Therefore the allowlist cannot block shell execution at the ACP layer. For real isolation we need OS-level sandboxing.

Future strategies:

- **OS-level sandboxing** — run each `kimi acp` process in a chroot, restricted user, or container so that `Bash` cannot escape the session `cwd`.
- **MCP server filtering** — only pass the `acompose` MCP server (or a subset of servers) in `NewSessionRequest.mcp_servers` so the agent cannot call arbitrary external MCP tools.
- **Proxy architecture** — run acompose as an ACP `Proxy` between the controller and the agent to intercept and drop disallowed requests/notifications.
- **Custom agent binary** — for high-trust scenarios, provide a stripped-down agent executable that exposes only the needed tools.

### 5. Session deletion / lifecycle tools

Add MCP tools so clients can manage sessions explicitly:

- `compose_delete_session(name_or_id)` — call `session/delete` if supported, then drop the handle.
- `compose_close_session(name_or_id)` — call `session/close` to free resources without deleting history.
- `compose_session_status(name_or_id)` — return ready state and latest `UsageUpdate`.

## MCP client example

Connect Kimi Code CLI or any MCP client to:

```
http://127.0.0.1:19094/mcp
```

Then call tools such as:

```json
{
  "name": "compose_create_session",
  "arguments": {
    "name": "review-agent",
    "cwd": "/Users/joe/work/some-repo",
    "charter": "You review pull requests. Be concise.",
    "allowed_tool_kinds": ["read", "search", "think"]
  }
}
```

## MCP config files

- `.mcp.json` — used by the outer Kimi Code CLI (`kimi`) when it runs in this project. It is **not** picked up by spawned `kimi acp` agents; remote servers need a `transport` field.
- `mcp_servers.json` — used by `mcp-cli`. Compatible with Claude Desktop / Gemini / VS Code format; remote servers use `url` only (no `transport` field).

## Using `mcp-cli`

From the repo root:

```bash
cd ~/work/acompose
mcp-cli -d                              # list servers and tools
mcp-cli call acompose compose_list_sessions '{}'
mcp-cli call acompose compose_create_session '{"name":"review-agent","cwd":"/Users/joe/work/some-repo","charter":"You review pull requests. Be concise.","allowed_tool_kinds":["read","search","think"]}'

# blocking call
mcp-cli call acompose compose_send_message '{"target":"review-agent","content":"Summarize open issues."}'

# background call + poll
mcp-cli call acompose compose_send_message '{"target":"review-agent","content":"Read every file and summarize.","wait":false}'
# returns {"status":"pending","prompt_id":"1"}
mcp-cli call acompose compose_get_prompt_result '{"prompt_id":"1"}'
```

`mcp-cli` looks for config in `./mcp_servers.json` (or `-c <path>`, `~/.mcp_servers.json`, `~/.config/mcp/mcp_servers.json`). For long-running calls, use `MCP_NO_DAEMON=1` to avoid the daemon timeout.

## Configuration example

```toml
kimi_binary = "kimi"

[mcp_server]
enabled = true
bind_address = "127.0.0.1:19094"

[[session]]
name = "infra-agent"
cwd = "/Users/joe/work/infra-repo"
allowed_tool_kinds = ["read", "search", "think", "fetch"]
charter = """
You are the infrastructure agent for this repository.
Set up a daily cron that runs `make audit` at 09:00.
Watch for changes to `terraform/` and notify the main agent.
"""

[[session]]
name = "docs-agent"
cwd = "/Users/joe/work/docs-repo"
allowed_tool_kinds = ["read", "search", "think"]
charter = """
You are the documentation agent.
Every morning check `docs/` for outdated pages and propose updates.
"""
```

## Architecture

- `src/main.rs` — CLI entry point, config loading, lifecycle.
- `src/acp_client.rs` — ACP stdio client wrapper around `agent-client-protocol` SDK.
- `src/config.rs` — TOML/JSON config parsing.
- `src/orchestrator.rs` — tracks active sessions, exposes compose MCP server, routes messages, manages persistence.
- `src/state.rs` — persistent mapping of session names to IDs.
- `src/mcp_server.rs` — HTTP/SSE MCP server exposing compose tools.

## Constraints

- Built with Rust.
- Uses `agent-client-protocol` crate from crates.io.
- Targets `kimi acp` as the ACP agent binary.
- Keep dependencies minimal; use async runtime of choice (tokio recommended by SDK examples).
