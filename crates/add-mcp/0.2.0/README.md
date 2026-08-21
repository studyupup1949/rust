# add-mcp

Install MCP servers into AI client configurations. Rust library + CLI.

## Supported Clients

| Client | Config Format | Global | Local |
|--------|:------------:|:------:|:-----:|
| Claude Code | JSON | yes | yes |
| Claude Desktop | JSON | yes | — |
| Codex | TOML | yes | yes |
| Cursor | JSON | yes | yes |
| Gemini CLI | JSON | yes | yes |
| Goose | YAML | yes | — |
| GitHub Copilot | JSON | yes | yes |
| OpenCode | JSON | yes | yes |
| VS Code | JSON | yes | yes |
| Zed | JSON | yes | yes |

## CLI Usage

```bash
# Install a local binary
add-mcp install /path/to/mcp-server -a claude-code -g

# Install a URL endpoint
add-mcp install https://example.com/mcp -a vscode -a cursor -g

# Install an npm package
add-mcp install @org/mcp-server -a claude-code -g

# Install to all agents
add-mcp install /path/to/server --all -g

# With env vars and extra args
add-mcp install /path/to/server -a claude-code -g -e API_KEY=secret -- --verbose

# List supported agents
add-mcp list-agents

# Detect installed agents
add-mcp detect
add-mcp detect --local
```

## Library Usage

```rust
use add_mcp::{install_command, Agent, Scope};

let binary = std::env::current_exe().unwrap();
let results = install_command(
    "my-server",
    binary.to_str().unwrap(),
    &[],
    &[Agent::ClaudeCode, Agent::Cursor],
    Scope::Global,
);

for result in results {
    match result {
        Ok(r) => println!("Installed to {} at {}", r.agent, r.path),
        Err(e) => eprintln!("Error: {e}"),
    }
}
```

As a dependency (library only, no CLI):

```toml
[dependencies]
add-mcp = { version = "0.1", default-features = false }
```

## License

MIT
