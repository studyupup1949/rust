# AAS — Autonomous Agent System

**Status:** Prototype Phase — Functional MVP, Pre-Production Polish
**Language:** Rust (edition 2021)
**Target Binary:** `./target/release/aas` — v0.1.0
**Test Coverage:** 20 tests, all passing, zero warnings

---

## 1. Overview

A multi-domain autonomous monitoring swarm that runs detect→analyze→plan→execute→verify→learn cycles for repository health, log analysis, system metrics, service health, task management, and distributed tracing. CLI-first with a web dashboard for setup and configuration.

### Core Loop

```
┌─────────┐    ┌──────────┐    ┌───────┐    ┌─────────┐    ┌────────┐    ┌───────┐
│ Detect  │───▶│ Analyze  │───▶│ Plan  │───▶│ Execute │───▶│ Verify │───▶│ Learn │
│ (agent) │    │ (LLM)    │    │ (LLM) │    │ (engine)│    │ (agent)│    │ (store)│
└─────────┘    └──────────┘    └───────┘    └─────────┘    └────────┘    └───────┘
                                                     │
                                                     ▼
                                              ┌───────────┐
                                              │ Rollback  │
                                              │ (on fail) │
                                              └───────────┘
```

---

## 2. Architecture

### 2.1 Module Map

```
src/
├── lib.rs                  # Public module declarations
├── main.rs                 # CLI entry point, command dispatch, LLM provider selection
│
├── agents/                 # 6 agent implementations
│   ├── repository.rs       # Git repo monitoring — status, diffs, uncommitted changes
│   ├── logs.rs             # Log file scanning — error/warning thresholds
│   ├── metrics.rs          # System metrics — CPU, memory, disk via sysinfo
│   ├── health.rs           # HTTP endpoint health checks
│   ├── task.rs             # Task management (stub)
│   └── trace.rs            # Distributed tracing (stub, disabled by default)
│
├── cli/                    # Clap-based CLI (25 commands + REPL)
├── config/
│   └── settings.rs         # Hierarchical config, JSON at ~/.aas/config.json, validate()
│
├── dashboard/
│   ├── routes.rs           # Axum 0.7 server, 8 REST endpoints
│   └── static/index.html   # Setup wizard SPA
│
├── execution/
│   └── staged.rs           # Staged rollout: test→validate→execute→verify, RollbackManager
│
├── integrations/
│   ├── git.rs              # GitOps via shell commands (tokio::process::Command)
│   └── github.rs           # GitHub API client (unwired)
│
├── llm/
│   ├── traits.rs           # LLMProvider trait
│   ├── mock.rs             # Canned responses, no external dep
│   ├── hermes.rs           # Local Hermes endpoint
│   └── claude.rs           # Anthropic Claude API
│
├── memory/
│   ├── store.rs            # SQLite (rusqlite) — 8 tables, async CRUD
│   ├── patterns.rs         # PatternEngine — find/create by signature
│   └── predictions.rs      # PredictionEngine — trend analysis
│
├── notifications/
│   └── slack.rs            # Slack webhook + email fallback
│
└── swarm/
    ├── types.rs            # Core domain types
    ├── agent.rs            # Agent trait + AgentContext
    ├── coordinator.rs      # Lifecycle, spawning, status
    └── event_bus.rs        # Tokio broadcast + optional SQLite persistence
```

### 2.2 Data Flow

```
User CLI/Dashboard
       │
       ▼
  ┌──────────┐     ┌──────────────┐     ┌───────────┐
  │Coordinator│────▶│  Agent Loop  │────▶│ EventBus  │
  └──────────┘     │ (each agent) │     │ (tokio tx)│
       │           └──────┬───────┘     └─────┬─────┘
       │                  │                    │
       ▼                  ▼                    ▼
  ┌──────────┐     ┌──────────────┐     ┌───────────┐
  │  Config  │     │ MemoryStore  │     │ Dashboard │
  │(JSON fs) │     │  (SQLite)    │     │  (Axum)   │
  └──────────┘     └──────────────┘     └───────────┘
```

### 2.3 Decision Records

| ID | Decision | Rationale |
|---|---|---|
| ADR-001 | No native build deps (removed `git2`) | `libssh2-sys` fails on macOS; use shell git via `tokio::process::Command` |
| ADR-002 | SQLite system-linked, not bundled | Avoid 2min+ compile of bundled C lib; use `pkg-config` for system sqlite3 |
| ADR-003 | Env vars before config for API keys | `ANTHROPIC_API_KEY` / `AAS_LLM_API_KEY` read first; secrets never logged |
| ADR-004 | MockLLMProvider as default | Agents complete full cycle without any external API |
| ADR-005 | Tokio broadcast + SQLite for event bus | In-memory broadcast for speed, SQLite persistence for audit history |

---

## 3. Functional Status

### 3.1 Implemented ✅

| Component | Status | Notes |
|---|---|---|
| CLI (25 commands) | Complete | init, run, stop, restart, status, dashboard, config, history, memory, trigger, approve, reject, rollback, explain, logs, errors, alerts, performance, export-config, backup, restore, validate-config, version, update, interactive REPL |
| Config system | Complete | JSON load/save at `~/.aas/config.json`, defaults, `validate()` |
| Memory store | Complete | 8 tables (issues, analyses, actions, action_results, decisions, patterns, predictions, events), indexed, async CRUD |
| Pattern engine | Complete | Signature-based matching via SQL LIKE with weighted scoring |
| Prediction engine | Complete | Trend analysis from historical pattern data |
| Event bus | Complete | Tokio broadcast channel + optional SQLite persistence |
| Execution engine | Complete | 4-stage test→validate→execute→verify with rollback |
| Mock LLM provider | Complete | Canned analysis, decision, chat responses |
| Hermes provider | Complete | Local endpoint chat completions |
| Claude provider | Complete | Anthropic Messages API |
| Dashboard API | Complete | 8 REST endpoints + setup wizard SPA |
| GitOps | Complete | Shell git status/diff/commit/log/branch/rollback |
| GitHub client | **Exists, unwired** | `create_issue`, `create_pr`, `test_connection` — never called |
| Notification manager | Complete | Slack webhook + email logging |
| Config validation | Complete | 8 validation rules in `validate()` |
| Pattern matching | Complete | Signature LIKE with weighted ordering |
| Unit/integration tests | Complete | 20 tests covering config, memory, event bus, LLM, types |

### 3.2 Not Implemented 🚧

| Component | Status | Blockers |
|---|---|---|
| Agent full cycle completion | Agents `detect` → store issues, but `analyze`/`plan`/`execute`/`verify`/`learn` return stubs | None — MockLLMProvider is ready, just needs wiring |
| GitHub API wiring | `GitHubClient` exists but never instantiated or called | None |
| Health agent auto-restart | Detects down endpoints but doesn't attempt restart | None |
| E2E integration test | No test starts coordinator + agents end-to-end | None |
| Config auto-create | `config.example.json` exists but no auto-init on first `run` | None |
| `install.sh` improvements | Builds + opens dashboard but doesn't register `~/.cargo/bin` path | None |
| Metadata usability | `metadata: [(String, String)].into()` pattern is verbose | Cosmetic |

---

## 4. Issues

### P0 — Blocking Production

| ID | Title | File | Description |
|---|---|---|---|
| AAS-001 | Agent analyze/plan/execute/verify/learn all return stubs | `src/agents/*.rs` | All 6 agents implement the full Agent trait but only `detect` does real work. `analyze` should call `ctx.llm.analyze()`, `plan` should call `ctx.llm.decide()`, `execute` should run real commands, `verify` should check results, `learn` should store patterns. This is the single highest-impact task — until it's done, the system detects but never acts. |
| AAS-002 | GitHub API client unwired | `src/integrations/github.rs` | `GitHubClient::create_issue()` and `create_pr()` are implemented but never called. The repo agent's `execute` method should use them to create issues/PRs for detected problems. |

### P1 — Core Completeness

| ID | Title | File | Description |
|---|---|---|---|
| AAS-003 | Health agent has no auto-restart logic | `src/agents/health.rs` | `detect` finds down endpoints but `execute` returns a no-op. Should attempt restart via shell command (e.g., `systemctl`, `launchctl`, `docker restart`) then recheck. |
| AAS-004 | No end-to-end integration test | `tests/integration.rs` | No test spawns the coordinator, triggers an agent, and asserts the full detect→analyze→plan→execute→verify→learn cycle completes. Needed before any further refactoring. |
| AAS-005 | `config.example.json` not auto-created on first run | `src/main.rs` | When running `aas run` with no config, the system silently uses defaults but never writes `~/.aas/config.json`. First `run` should auto-create from defaults + write to disk. |

### P2 — Quality of Life

| ID | Title | File | Description |
|---|---|---|---|
| AAS-006 | `install.sh` doesn't register binary in PATH | `install.sh` | Script builds to `target/release/aas` but doesn't copy to `~/.cargo/bin/aas` or add to shell PATH. User must manually find the binary. |
| AAS-007 | Metadata HashMap construction is verbose | `src/agents/*.rs` | Pattern `[("key".to_string(), "val".to_string())].into()` appears 20+ times. Should use a builder or `HashMap::from([(...)])`. |

### P3 — Polish

| ID | Title | File | Description |
|---|---|---|---|
| AAS-008 | Trace and task agents are stubs | `src/agents/task.rs`, `src/agents/trace.rs` | Both implement Agent trait but `detect` returns empty vec. Disabled by default, no user-facing impact. |

---

## 5. Technical Specifications

### 5.1 Dependencies

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }      # Async runtime
axum = "0.7"                                          # HTTP server (dashboard)
clap = { version = "4", features = ["derive"] }       # CLI
serde = { version = "1", features = ["derive"] }      # Serialization
serde_json = "1"                                      # JSON config
rusqlite = { version = "0.31", features = ["column_decltype"] }  # SQLite (no bundled)
chrono = { version = "0.4", features = ["serde"] }    # Time handling
reqwest = { version = "0.12", features = ["json"] }   # HTTP client (LLM, GitHub)
tracing = "0.1"                                       # Structured logging
tracing-subscriber = "0.3"                            # Log output
uuid = { version = "1", features = ["v4"] }           # ID generation
async-trait = "0.1"                                   # Async trait support
rustyline = "14"                                      # REPL
sysinfo = "0.30"                                      # System metrics
dirs = "5"                                            # Home directory
tokio-util = "0.7"                                    # Utilities
tower-http = { version = "0.5", features = ["fs"] }   # Static file serving
```

### 5.2 Configuration Schema

Location: `~/.aas/config.json`

```json
{
  "version": "1.0",
  "llm": {
    "provider": "mock",
    "endpoint": "http://localhost:5001",
    "model": "hermes-3-llama-3.1-8b",
    "timeout_seconds": 30,
    "api_key": ""
  },
  "agents": {
    "repository": { "enabled": true, "detection_interval": "1h" },
    "logs": { "enabled": true, "detection_interval": "continuous" },
    "metrics": { "enabled": true, "detection_interval": "1m" },
    "health": { "enabled": true, "detection_interval": "30s" },
    "task": { "enabled": true, "detection_interval": "10m" },
    "trace": { "enabled": false, "detection_interval": "5m" }
  },
  "execution": { "mode": "staged_rollout", "max_concurrent_actions": 3 },
  "learning": { "enabled": true, "prediction_confidence_threshold": 0.85 },
  "notifications": { "channels": [] }
}
```

### 5.3 Database Schema (SQLite — 8 tables)

| Table | Purpose | Key Columns |
|---|---|---|
| `issues` | Agent-detected problems | id, domain, agent, title, severity, signature, stage |
| `analyses` | LLM analysis results | id, issue_id, root_cause, impact, suggested_fix, confidence |
| `actions` | Executed actions | id, issue_id, action_type, command, params |
| `action_results` | Action outcomes | id, action_id, success, exit_code, output |
| `decisions` | Full decision records | id, issue_id, status, created_at |
| `patterns` | Learned patterns | id, name, domain, indicators, confidence, occurrences |
| `predictions` | Future predictions | id, agent, predicted_issue, confidence, status |
| `events` | Agent event audit log | id, event_type, agent, payload |

### 5.4 API Endpoints (Dashboard)

| Method | Path | Purpose |
|---|---|---|
| GET | `/api/status` | Swarm status |
| GET | `/api/config` | Current config |
| POST | `/api/config` | Update config (validates) |
| GET | `/api/history` | Decision history |
| GET | `/api/patterns` | Learned patterns |
| GET | `/api/predictions` | Active predictions |
| GET | `/api/agents` | Per-agent status |
| POST | `/api/trigger/{agent}` | Manual agent trigger |

---

## 6. Build & Run Commands

```bash
# Build
cargo build --release                       # 330KB optimized binary
cargo test                                  # 20 tests, zero warnings
cargo clippy                                # lint (no current violations)

# Run
./target/release/aas init                   # interactive setup
./target/release/aas run                    # start all enabled agents
./target/release/aas status                 # show swarm status
./target/release/aas dashboard              # launch web UI
./target/release/aas interactive            # REPL mode
./target/release/aas validate-config        # check ~/.aas/config.json
./target/release/aas memory stats           # show learned patterns/predictions
./target/release/aas backup                 # backup ~/.aas/ completE
```

---

## 7. Known Constraints

- **macOS**: No `launchctl` restart logic in health agent (only `systemctl`)
- **No bundled SQLite**: Requires `sqlite3` on `PKG_CONFIG_PATH` or system-installed
- **Demo mode**: `MockLLMProvider` used when no `ANTHROPIC_API_KEY` set — all LLM responses are canned
- **Shell git**: `GitOps` invokes `git` as subprocess — performance is ~10ms/call for status
