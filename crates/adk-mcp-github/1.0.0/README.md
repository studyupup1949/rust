# GitHub MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-github.svg)](https://crates.io/crates/mcp-github)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)

GitHub integration for [ADK-Rust Enterprise](https://enterprise.adk-rust.com) agents. Provides 13 MCP tools for repositories, files, branches, issues, PRs, diffs, reviews, and releases — all via the GitHub REST API.

## Key Principles

- **Real GitHub API** — all tools call the live GitHub REST API with your PAT.
- **Read + governed writes** — search, browse, and read are unrestricted; issue creation and PR reviews are write operations.
- **Diff-aware** — get full PR diffs for code review agents.
- **Registry-ready** — ships with `mcp-server.toml` for ADK-Rust Enterprise onboarding.

## Tools (13)

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `search_repositories` | Search GitHub repositories by query | Read-only |
| `get_repository` | Get repo details (branch, language, stats) | Read-only |
| `list_branches` | List branches with protection status | Read-only |
| `get_file_contents` | Read file contents (decoded from base64) | Read-only |
| `search_code` | Search code across repositories | Read-only |
| `list_pull_requests` | List PRs by state (open/closed/all) | Read-only |
| `get_pull_request` | Get PR with merge status and stats | Read-only |
| `get_pull_request_diff` | Get full diff for a PR | Read-only |
| `create_pull_request_review` | Submit PR review (APPROVE/REQUEST_CHANGES/COMMENT) | External write |
| `create_issue` | Create a new issue | External write |
| `update_issue` | Update issue title, body, state, labels | External write |
| `list_releases` | List releases for a repository | Read-only |
| `create_release_note` | Generate release notes between tags | Read-only |

## Verified Output

Tested against live GitHub API:

```
> search_repositories("zavora-ai/mcp-a2a")

[{ "full_name": "zavora-ai/mcp-a2a", "stargazers_count": 1, "html_url": "https://github.com/zavora-ai/mcp-a2a" }]

> get_repository(owner: "zavora-ai", repo: "mcp-a2a")

{ "full_name": "zavora-ai/mcp-a2a", "default_branch": "main", "language": "Rust", "stargazers_count": 1 }

> get_file_contents(owner: "zavora-ai", repo: "mcp-a2a", path: "Cargo.toml")

{ "path": "Cargo.toml", "size": 1191, "sha": "6dab006f...", "content": "[package]\nname = \"mcp-a2a\"..." }

> list_branches(owner: "zavora-ai", repo: "mcp-a2a")

[{ "name": "main", "protected": false }]
```

## Installation

### Build from source

```bash
git clone https://github.com/zavora-ai/mcp-github
cd mcp-github
cargo build --release
```

### Configuration

Set your GitHub token:

```bash
export GITHUB_TOKEN=ghp_xxxxxxxxxxxx
# or
export GITHUB_PERSONAL_ACCESS_TOKEN=ghp_xxxxxxxxxxxx
```

### Claude Desktop

```json
{
  "mcpServers": {
    "github": {
      "command": "/path/to/mcp-github",
      "env": { "GITHUB_TOKEN": "ghp_xxxx" }
    }
  }
}
```

### Kiro

```json
{
  "mcpServers": {
    "github": {
      "command": "/path/to/mcp-github",
      "env": { "GITHUB_TOKEN": "ghp_xxxx" }
    }
  }
}
```

### Cursor

```json
{
  "mcpServers": {
    "github": {
      "command": "/path/to/mcp-github",
      "env": { "GITHUB_TOKEN": "ghp_xxxx" }
    }
  }
}
```

## Governance

- PR reviews require appropriate repo permissions
- Issue creation respects repository write access
- CODEOWNERS checks recommended for production write operations
- All write operations are audit-logged

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START -->
| [<img src="https://github.com/jkmaina.png" width="80px;" alt=""/><br /><sub><b>James Karanja Maina</b></sub>](https://github.com/jkmaina) |
|:---:|
<!-- ALL-CONTRIBUTORS-LIST:END -->

## License

Apache-2.0 — see [LICENSE](LICENSE) for details.

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.
