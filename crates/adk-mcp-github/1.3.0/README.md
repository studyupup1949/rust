# GitHub MCP Server

[![Crates.io](https://img.shields.io/crates/v/adk-mcp-github.svg)](https://crates.io/crates/adk-mcp-github)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)
[![Registry Ready](https://img.shields.io/badge/ADK_Registry-Ready-green.svg)](https://www.zavora.ai)

Complete GitHub and local git integration for [ADK-Rust Enterprise](https://enterprise.adk-rust.com) agents. Provides 38 MCP tools covering the full repository lifecycle — from `create_repository` through `git_push` to `create_release`.

## Architecture

<p align="center">
  <img src="https://raw.githubusercontent.com/zavora-ai/adk-mcp-github/main/docs/architecture.svg" alt="GitHub MCP Architecture" width="800"/>
</p>

## Key Principles

- **Full lifecycle** — create repos, clone, branch, commit, push, open PRs, review, merge, release.
- **Real GitHub API** — all remote tools call the live GitHub REST API with your PAT.
- **Local git operations** — clone, init, add, commit, push, tag, log via local `git` binary.
- **Governed writes** — issue creation, PR reviews, merges, and repo deletion are write operations.
- **Registry-ready** — ships with `mcp-server.toml` for ADK-Rust Enterprise onboarding.

## Tools (38)

### GitHub API — Search & Discovery

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `search_repositories` | Search GitHub repositories by query | Read-only |
| `search_code` | Search code across repositories | Read-only |

### GitHub API — Repositories

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `get_repository` | Get repo details (branch, language, stats) | Read-only |
| `create_repository` | Create a new repo (personal or org) | External write |
| `fork_repository` | Fork a repository to your account | External write |
| `delete_repository` | Delete a repository (irreversible) | Critical write |
| `list_branches` | List branches with protection status | Read-only |
| `list_directory` | Browse repo tree contents | Read-only |
| `list_tags` | List git tags | Read-only |

### GitHub API — Files

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `get_file_contents` | Read file contents (decoded from base64) | Read-only |
| `create_or_update_file` | Push file changes via API | External write |

### GitHub API — Pull Requests

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `list_pull_requests` | List PRs by state | Read-only |
| `get_pull_request` | Get PR with merge status and stats | Read-only |
| `get_pull_request_diff` | Get full diff for a PR | Read-only |
| `create_pull_request` | Open a new PR | External write |
| `merge_pull_request` | Merge a PR (merge/squash/rebase) | External write |
| `create_pull_request_review` | Submit review (APPROVE/REQUEST_CHANGES/COMMENT) | External write |
| `list_pr_comments` | List comments on a PR | Read-only |
| `add_pr_comment` | Add a comment to a PR | External write |

### GitHub API — Branches & Commits

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `create_branch` | Create a new branch from a SHA | External write |
| `delete_branch` | Delete a branch | External write |
| `list_commits` | List commits for a repo or branch | Read-only |
| `get_commit` | Get details of a specific commit | Read-only |

### GitHub API — Issues

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `create_issue` | Create a new issue | External write |
| `update_issue` | Update issue title, body, state, labels | External write |

### GitHub API — CI/Actions

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `list_workflow_runs` | List GitHub Actions workflow runs | Read-only |
| `get_workflow_run` | Get details of a specific run | Read-only |

### GitHub API — Releases

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `list_releases` | List releases for a repository | Read-only |
| `create_release` | Create a GitHub release | External write |
| `create_release_note` | Generate release notes between tags | Read-only |

### Local Git Operations

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `git_clone` | Clone a repository to local filesystem | Local write |
| `git_init` | Initialize a new git repository | Local write |
| `git_status` | Get working tree status | Read-only |
| `git_add` | Stage files for commit | Local write |
| `git_commit` | Create a commit with message | Local write |
| `git_push` | Push commits to remote | External write |
| `git_tag` | Create a git tag | Local write |
| `git_log` | Show recent commit history | Read-only |

## Verified Output

Tested against live GitHub API:

```
> search_repositories("org:zavora-ai mcp")
  ✓ 10 repos found

> get_repository(owner: "zavora-ai", repo: "mcp-a2a")
  ✓ zavora-ai/mcp-a2a | branch: main | lang: Rust

> list_tags(owner: "zavora-ai", repo: "mcp-a2a")
  ✓ v1.1.0 (7b5acb11), v1.0.0 (48e124f1)

> get_file_contents(owner: "zavora-ai", repo: "mcp-a2a", path: "Cargo.toml")
  ✓ 1191 bytes | sha: 6dab006f

> create_issue(owner: "zavora-ai", repo: "mcp-registry", title: "[Test] Integration verified")
  ✓ #1 created → https://github.com/zavora-ai/mcp-registry/issues/1

> git_log(path: ".", count: 3)
  ✓ 98768c9 feat: GitHub MCP v1.1.0 — 24 tools, full PR lifecycle, verified
```

## Installation

### Build from source

```bash
git clone https://github.com/zavora-ai/adk-mcp-github
cd adk-mcp-github
cargo build --release
```

The binary is at `target/release/adk-mcp-github`.

### Configuration

```bash
export GITHUB_TOKEN=ghp_xxxxxxxxxxxx
```

Required scopes: `repo`, `read:org`, `workflow` (for Actions tools).

### Claude Desktop

```json
{
  "mcpServers": {
    "github": {
      "command": "/path/to/adk-mcp-github",
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
      "command": "/path/to/adk-mcp-github",
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
      "command": "/path/to/adk-mcp-github",
      "env": { "GITHUB_TOKEN": "ghp_xxxx" }
    }
  }
}
```

### Windsurf

```json
{
  "mcpServers": {
    "github": {
      "command": "/path/to/adk-mcp-github",
      "env": { "GITHUB_TOKEN": "ghp_xxxx" }
    }
  }
}
```

## Governance

- `delete_repository` is irreversible — requires explicit confirmation in production
- PR reviews require appropriate repo permissions
- `merge_pull_request` respects branch protection rules
- `git_push` requires configured credentials on the local machine
- All write operations are audit-logged when used with MCP Registry

## MCP Server Manifest

```toml
server_id = "mcp_github"
display_name = "GitHub MCP"
version = "1.2.1"
domain = "developer"
risk_level = "medium"
writes_allowed = "gated"
transports = ["stdio"]
governance_gates = ["repo_policy", "codeowners_check"]
```

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START -->
| [<img src="https://github.com/jkmaina.png" width="80px;" alt=""/><br /><sub><b>James Karanja Maina</b></sub>](https://github.com/jkmaina) |
|:---:|
<!-- ALL-CONTRIBUTORS-LIST:END -->

## License

Apache-2.0 — see [LICENSE](LICENSE) for details.

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.

## Registry Compliance

This server implements the [ADK MCP SDK](https://crates.io/crates/adk-mcp-sdk) contract:

- **HealthCheck** — async health probe for registry monitoring
- **mcp-server.toml** — manifest declaring tools, risk classes, and credentials
- **Structured tracing** — `RUST_LOG` env-filter for observability

