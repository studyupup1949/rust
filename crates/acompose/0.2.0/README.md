# acompose

Agent composition server built in Rust using the Agent Client Protocol (ACP).

## What it does

Spawn persistent `kimi acp` agents, keep them alive, and expose them through an MCP server.

## Configuration

Configuration is read from `acompose.toml`:

```toml
kimi_binary = "kimi"

# Optional: control-plane MCP server (agents can reference this via acompose)
[acompose_control_mcp]
bind_address = "127.0.0.1:9092"

# Optional: MCP servers available to agent sessions (referenced by name)
[[mcp_servers]]
name = "kudach"
url = "http://localhost:9091/mcp"
transport = "http"

# Agent sessions
[[session]]
name = "moderator"
cwd = "./agents/moderator"
charter = "You are a moderation agent."
mcp_servers = ["kudach"]

[[session]]
name = "coordinator"
cwd = "./agents/coordinator"
charter = "You are the coordinator."
mcp_servers = ["acompose"]
```

### Session configuration

| Field | Description |
|---|---|
| `name` | Unique session name |
| `cwd` | Working directory for the agent |
| `charter` | System prompt sent to the agent on startup |
| `allowed_tool_kinds` | Optional allowlist of ACP tool kinds |
| `mcp_servers` | List of MCP server names (from `[[mcp_servers]]` or the special `acompose`) |

### `acompose` reference

Any agent can be given access to the acompose control-plane MCP server by including `"acompose"` in its `mcp_servers` list. This lets the agent:
- list active sessions via `list_sessions`
- create new sessions via `create_session`
- send messages to sessions via `send_message`
- recreate sessions via `recreate_session`
- get system documentation via `agent_info`

Agents communicate asynchronously: when session A calls `send_message(target=B, content=..., need_result=true)`, the message is queued for session B. Once B finishes its turn, its response is delivered back to session A as a new incoming message. There is no separate polling API; use `need_result: true` to receive the outcome.

## Quick start

```toml
# acompose.toml
kimi_binary = "kimi"

[[session]]
name = "moderator"
cwd = "/Users/joe/work/kudach"
charter = "You are a moderation agent."
```

```bash
cargo run -- --config acompose.toml
```

## MCP tools

- `agent_info` — get acompose system documentation
- `list_sessions` — list active sessions
- `create_session` — spawn a new agent
- `recreate_session` — restart an agent with an optional new charter
- `send_message` — send a prompt to an agent

  Example:
  ```json
  {
    "target": "acompose-sidekick",
    "content": "Привет! Проверяем связь через acompose.",
    "need_result": true
  }
  ```

  With `need_result: true` the target agent's response is delivered back to the caller as a new message once the target finishes its turn. Set `need_result: false` for fire-and-forget messages.

## License

MIT
