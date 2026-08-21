# adaptive-card-mcp

[![crates.io](https://img.shields.io/crates/v/adaptive-card-mcp.svg)](https://crates.io/crates/adaptive-card-mcp)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

MCP (Model Context Protocol) server that exposes
[`adaptive-card-core`](https://crates.io/crates/adaptive-card-core) tools to any
LLM client over stdio.

Use it from Claude Code, Cursor, Windsurf, GitHub Copilot, ChatGPT — or any
agent that speaks MCP — to validate, optimize, and transform Microsoft Adaptive
Cards v1.6 with a free-form LLM design loop.

## Surface

### Tools (10)

| Tool | Purpose |
|---|---|
| `validate_card` | v1.6 schema + accessibility + optional host compat |
| `analyze_card` | Element count, nesting depth, duplicate IDs |
| `check_accessibility` | 0–100 score with rule-by-rule fix hints |
| `optimize_card` | Auto-fix accessibility / performance / modernize actions |
| `transform_card` | Downgrade version or adapt to a target host |
| `template_card` | Convert literal card → `${expression}` template + sample data |
| `data_to_card` | Auto-generate card from data shape (Table / FactSet / List) |
| `list_examples` | Browse curated knowledge base entries |
| `get_example` | Fetch a single knowledge base entry by id |
| `suggest_layout` | Keyword search across the knowledge base |

### Prompts (3 guided workflows)

| Prompt | Pipeline |
|---|---|
| `review-adaptive-card` | validate → optimize a11y → before/after report |
| `refine-for-host` | validate → transform for host → re-validate |
| `templatize-card` | template → validate → return template + sample data |

### Resources (5 `ac://` URIs)

- `ac://schema/v1.6` — embedded Microsoft v1.6 JSON Schema
- `ac://hosts` — host capability matrix
- `ac://hosts/{name}` — single host detail
- `ac://examples` — knowledge base summary
- `ac://examples/{id}` — single knowledge base entry

## Install

### Pre-built binary (recommended)

```bash
cargo binstall adaptive-card-mcp
```

### From source

```bash
cargo install adaptive-card-mcp
```

## Wire to your MCP client

### Claude Code

```bash
claude mcp add adaptive-card-mcp -- adaptive-card-mcp
```

### Cursor (`.cursor/mcp.json`)

```json
{
  "mcpServers": {
    "adaptive-card-mcp": {
      "command": "adaptive-card-mcp"
    }
  }
}
```

### Windsurf (`~/.codeium/windsurf/mcp_config.json`)

```json
{
  "mcpServers": {
    "adaptive-card-mcp": {
      "command": "adaptive-card-mcp"
    }
  }
}
```

## CLI

```text
adaptive-card-mcp [OPTIONS]

Options:
  --transport <TRANSPORT>     stdio | sse           [env: TRANSPORT, default: stdio]
  --port <PORT>               SSE port              [env: PORT, default: 3001]
  --bind <BIND>               SSE bind address      [env: BIND, default: 127.0.0.1]
  --api-key <API_KEY>         Bearer key for SSE    [env: MCP_API_KEY]
  --log <LOG>                 tracing EnvFilter     [env: RUST_LOG, default: adaptive_card_mcp=info]
  --knowledge-base <DIR>      External KB dir       [env: KNOWLEDGE_BASE_DIR]
```

## Known limitations (v0.1.0)

- **SSE transport not yet supported.** `--transport sse` returns an error
  directing you to `--transport stdio`. An axum-based SSE adapter is planned
  for a follow-up release.
- **Knowledge base ships empty.** Curated samples are added incrementally.

## License

MIT — see `LICENSE` at the workspace root.

The embedded Adaptive Cards v1.6 JSON Schema is sourced from
[`microsoft/AdaptiveCards`](https://github.com/microsoft/AdaptiveCards) under
its MIT license.
