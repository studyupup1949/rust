# acompose

Agent composition orchestrator built in Rust using the Agent Client Protocol (ACP).

## What it does

Spawn persistent `kimi acp` agents, keep them alive, and expose them through an MCP server.

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

- `compose_list_sessions` — list active sessions
- `compose_create_session` — spawn a new agent
- `compose_send_message` — send a prompt to an agent
- `compose_get_prompt_result` — poll async prompt results

## License

MIT
