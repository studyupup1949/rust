# acompose

Agent composition compositor built in Rust using the Agent Client Protocol (ACP).

## Goal

Provide a single binary that:

1. Spawns persistent acp agents in arbitrary working directories.
2. Sends each agent a charter (system prompt).
3. Keeps agents alive so they can receive follow-up prompts from the compositor or from other agents.
4. Lets agents spawn sub-agents and coordinate through an MCP control plane.

## Why ACP

ACP (Zed's Agent Client Protocol) lets a client drive a long-running agent over stdio/HTTP. acompose is the ACP client:

- Spawns acp processes.
- Runs the `initialize` handshake.
- Creates or loads sessions.
- Sends the charter and routes later prompts.

Because the agent process stays alive, its session state survives across individual prompts.

## Running

```bash
cd ~/work/acompose
process-compose up -D --env .pc_env
process-compose -p 19093 process logs acompose
```

The process-compose integrated MCP server listens on `http://127.0.0.1:19094/mcp` by default.

## MCP tools

Spawned agents get access to the acompose control-plane MCP server (it is passed explicitly in `NewSessionRequest.mcp_servers`). Available tools:

- `list_sessions()` — list active sessions.
- `create_session(name, cwd, charter, model = None, allowed_tool_kinds = [])` — create a sub-agent. The optional `model` selects a model via ACP session config options (`session/set_config_option` on the agent's `model` config option); it must be one of the values the agent offers, otherwise session creation fails with the list of available values.
- `send_message(target, content, need_result = true)` — send a message to another session.
  ```json
  {
    "target": "acompose-sidekick",
    "content": "Привет! Проверяем связь через acompose.",
    "need_result": true
  }
  ```
  When `need_result` is true and the caller is identified, the target agent's response is delivered back as a new message once the target finishes its turn. There is no separate polling API; this is how agents receive each other's results.
- `recreate_session(name, ...)` — reset a session.
- `delete_session(name)` — delete a session.
- `agent_info()` — get acompose system documentation.

### Agent self-identification

At the start of every session an agent **must** call `list_sessions()` **before doing any other work**. The result of this call tells the agent whether it is running inside acompose or was started outside of it, and reveals its session name. The compositor tags each agent's MCP connection with an `agent` query parameter (e.g. `http://127.0.0.1:10003/mcp?agent=code-reviewer`), so the returned list highlights the caller with ` <- you`. Without this call the agent has no way to tell if it is running inside acompose and cannot identify itself correctly.

## Session persistence

`state.json` stores `session.name -> session_id`. On startup acompose tries `session/load`; if that fails, it falls back to `session/new` and updates `state.json`.

Note: whether the new agent process can restore session history is an implementation detail of agent. acompose itself only persists the name-to-id mapping.

## Tool / permission sandboxing

- `SessionConfig.allowed_tool_kinds` enforces an allowlist for permission requests.
- agent does **not** ask permission before executing `Bash`/`Execute` tools; it only reports them as `SessionUpdate` notifications. Real isolation requires OS-level sandboxing.

## Model selection

`SessionConfig.model` (optional) selects the model through ACP [session config options](https://agentclientprotocol.com/protocol/v1/session-config-options): after `session/new` (and again after `session/load` and `recreate_session`, since those reset config options) acompose finds the agent's config option with `category: "model"` and calls `session/set_config_option`. If the agent doesn't report config options, a warning is logged and the agent's default model is kept; if the requested value isn't offered, session creation fails listing the available values. The model is persisted in `state.json` per session. The Compose WebSocket server's `session/new` also accepts an optional `model` in the request `_meta` (next to `name`).

## Configuration example

```toml
acp_command = "kimi acp"

[mcp_server]
enabled = true
bind_address = "127.0.0.1:10003"

[[session]]
name = "infra-agent"
cwd = "/Users/joe/work/infra-repo"
model = "kimi-code/k3"
allowed_tool_kinds = ["read", "search", "think", "fetch"]
charter = """
You are the infrastructure agent for this repository.
"""
```
