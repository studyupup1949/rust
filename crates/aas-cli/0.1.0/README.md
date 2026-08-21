```
███   ███   ████   
█ ░░█ █ ░░█ █ ░░░░  
█████░█████░ ███░░░ 
█░░░█░█░░░█░░ ░░█   
█░░░█░█░░░█░████░░  
 ░░  ░░░░  ░░░░░░ ░ 
  ░   ░ ░   ░ ░░░░
```

# AAS — Autonomous Agent System

> Self-improving agent swarm that learns from experience, remembers solutions, and improves itself.

## What It Does

AAS runs autonomous agents that detect problems, fix them, remember solutions, and improve over time. No human intervention needed after startup.

**Real autonomy**: Agents don't just *simulate* fixes. They execute them, verify they work, and cache solutions for next time.

## Quick Start

### Install

```bash
cargo build --release
./target/release/aas init  # Create config
```

### Run as Daemon

```bash
aas daemon start
aas status      # Check running agents
aas logs follow # Watch agent output
```

### Connect Integrations

```bash
aas connect claude-code     # Enable Claude Code CLI for code edits
aas connect openclaw        # Enable OpenClaw for external tasks
aas config show             # View all connections
```

### One-Off Run

```bash
aas run --duration 5m       # Run for 5 minutes
aas run --until-solved      # Run until all issues resolved
```

## Architecture

### Agents (Per-Domain)

- **Repository Agent** — Git commits, branch management, cleanup
- **Logs Agent** — Error detection, log rotation, cleanup
- **Health Agent** — Service monitoring, restarts, alerts
- **Metrics Agent** — Disk/memory cleanup, optimization

### Learning Pipeline

Each agent:

1. **Detects** issues (uncommitted changes, high error rate, service down)
2. **Checks cache** — solved this before? Reuse solution (0 LLM calls)
3. **If miss** — analyze + plan via LLM
4. **Executes** — actually run the commands
5. **Verifies** — confirm problem is gone
6. **Learns** — cache successful solution
7. **Improves** — RSI engine adjusts confidence thresholds

### Cost Model

- **First occurrence of issue**: 2 LLM calls (analyze + plan)
- **Repeat occurrences**: 0 LLM calls (cached solution)
- **100 cycles, 10 unique issues**: ~90% LLM savings

## Features

### ✅ Real Execution

- Agents run actual commands, not simulations
- Git operations, log cleanup, service restarts all real
- Failures rolled back, successes cached

### ✅ Learning from Experience

- Successful solutions stored in `learned_solutions` HashMap
- Identical issues skip LLM entirely on repeat
- Observable in logs: "learned solution for X", "reusing learned solution for X"

### ✅ Recursive Self-Improvement

- RSI Engine tracks agent success rates
- Adjusts confidence thresholds dynamically
- Agents that succeed become more aggressive
- Agents that fail become more conservative

### ✅ Multi-Provider LLM Routing

- **Claude API** — deep reasoning (analyze, plan)
- **Hermes** — fast local analysis
- **Claude Code** — file edits and code modifications
- **OpenClaw** — external tasks (notifications, integrations)
- **Fallback** — mock responses if all else fails

### ✅ Event-Driven Architecture

- Agents emit events (issue detected, action completed)
- Other agents react (hyperfocus on critical issues)
- Dashboard subscribes to all events
- Perfect observability

## Configuration

```bash
# View current config
aas config show

# Edit config
aas config edit

# Set specific values
aas config set agents.repository.enabled true
aas config set rsi.confidence_threshold 0.65
```

### Config File (~/.config/aas/config.toml)

```toml
[agents]
repository = { enabled = true, interval = "60s" }
logs = { enabled = true, interval = "30s" }
health = { enabled = true, interval = "10s" }
metrics = { enabled = true, interval = "120s" }

[rsi]
enabled = true
min_confidence_threshold = 0.3
max_confidence_threshold = 0.95
min_interval_secs = 5

[integrations.claude_code]
enabled = false  # Set to true after: aas connect claude-code

[integrations.openclaw]
enabled = false
endpoint = "http://localhost:3001"
```

## CLI Commands

### Daemon Management

```bash
aas daemon start              # Start background daemon
aas daemon stop               # Stop daemon
aas daemon restart            # Restart
aas daemon status             # Check if running
aas daemon logs               # View daemon logs
```

### Integrations

```bash
aas connect claude-code       # Enable Claude Code (requires binary in PATH)
aas connect openclaw          # Enable OpenClaw (requires endpoint)
aas connect list              # Show all available integrations
aas disconnect <name>         # Disable an integration
```

### Monitoring

```bash
aas status                    # Agent health snapshot
aas logs follow               # Stream agent logs
aas logs filter repository    # Logs from repository agent only
aas metrics                   # LLM calls saved, cache hit rate, etc.
aas dashboard                 # Open TUI dashboard
```

### Development

```bash
aas run                       # One-off run (foreground)
aas run --duration 5m         # Run for specific duration
aas run --dry-run             # Detect issues but don't execute
aas test <agent>              # Test single agent
aas bench                     # Performance benchmark
```

## Learning in Action

### First Run: Detects "Uncommitted Changes"

```
[INFO] repository: detecting issues...
[INFO] repository: found 1 issue: "Uncommitted changes accumulating"
[INFO] repository: analyzing...                      # LLM call 1
[INFO] repository: planning...                       # LLM call 2
[INFO] repository: executing: git add . && git commit
[INFO] repository: verifying... success ✓
[INFO] repository: learned solution for repository:uncommitted_changes
```

### Second Run: Same Issue, No LLM

```
[INFO] repository: detecting issues...
[INFO] repository: found 1 issue: "Uncommitted changes accumulating"
[INFO] repository: reusing learned solution for repository:uncommitted_changes
[INFO] repository: executing: git add . && git commit (cached)
[INFO] repository: verifying... success ✓
```

### Metrics Over 100 Cycles

```
repo     │ learned: 5 unique solutions │ cache hits: 68 (87%)
logs     │ learned: 8 unique solutions │ cache hits: 52 (71%)
health   │ learned: 3 unique solutions │ cache hits: 45 (94%)
metrics  │ learned: 2 unique solutions │ cache hits: 28 (88%)

Total LLM calls saved: ~180 (89% reduction)
```

## Development

### Build

```bash
cargo build --release
```

### Test

```bash
cargo test
cargo test --release
```

### Run with Tracing

```bash
RUST_LOG=debug ./target/release/aas run
RUST_LOG=aas::swarm=trace ./target/release/aas run
```

## Project Structure

```
aas/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Core library
│   ├── swarm/               # Agent framework
│   │   ├── agent.rs         # Agent trait, run_cycle, learning cache
│   │   ├── coordinator.rs   # Agent orchestration, daemon mode
│   │   ├── types.rs         # Issue, Action, Analysis types
│   │   └── event_bus.rs     # Event broadcasting
│   ├── memory/              # Learning & storage
│   │   ├── learning.rs      # issue_signature(), cache logic
│   │   ├── store.rs         # SQLite storage
│   │   ├── patterns.rs      # Pattern matching
│   │   └── predictions.rs   # Trend prediction
│   ├── execution/           # Safe execution
│   │   ├── staged.rs        # Execution engine with verification
│   │   └── verification.rs  # Problem validation
│   ├── agents/              # Per-domain agents
│   │   ├── repository.rs
│   │   ├── logs.rs
│   │   ├── health.rs
│   │   └── metrics.rs
│   ├── llm/                 # LLM providers & routing
│   │   ├── traits.rs        # LLMProvider trait
│   │   ├── router.rs        # TaskType → Provider routing
│   │   └── providers/       # Claude, Hermes, etc.
│   ├── integrations/        # External service connectors
│   │   ├── claude_code.rs   # Claude Code CLI subprocess
│   │   └── openclaw.rs      # OpenClaw REST API
│   ├── rsi/                 # Recursive self-improvement
│   │   └── engine.rs        # Threshold & interval adjustment
│   ├── cli/                 # Command-line interface
│   │   ├── daemon.rs
│   │   ├── config.rs
│   │   ├── integrations.rs
│   │   └── dashboard.rs
│   └── config/
│       └── settings.rs      # Config management
├── tests/
│   ├── integration/
│   └── e2e/
├── Cargo.toml
├── README.md                # This file
├── CONTRIBUTING.md
├── .github/
│   ├── workflows/           # CI/CD
│   ├── ISSUE_TEMPLATE/
│   └── pull_request_template.md
└── docs/
    ├── architecture.md
    ├── learning.md
    └── integrations.md
```

## Integration Guide

### Claude Code

Enable code editing and file modifications:

```bash
aas connect claude-code
# Checks for `claude` binary in PATH
# Enables TaskType::CodeEdit routing
```

Use cases:
- Fixing Python/Rust/JS test failures
- Automated code generation
- File reformatting

### OpenClaw

Enable external task automation:

```bash
aas connect openclaw --endpoint http://localhost:3001
# Connects to OpenClaw API
# Enables TaskType::ExternalTask routing
```

Use cases:
- Send alerts to Slack/Discord
- Trigger external workflows
- Update remote systems

### Custom Provider

Implement the `LLMProvider` trait:

```rust
#[async_trait]
impl LLMProvider for MyProvider {
    async fn chat(&self, messages: &[Message], options: &LLMOptions) -> Result<LLMResponse, String> {
        // Your implementation
    }
}
```

Register in LLMRouter and wire into AgentContext.

## Monitoring & Observability

### Logs

All agent output to `~/.local/share/aas/logs/`:

```bash
aas logs follow                    # All agents
aas logs follow --agent repository # Single agent
aas logs search "error"            # Full-text search
```

### Metrics

Dashboard shows:
- **Success rate** per agent
- **Cache hit rate** (repeated issues solved with 0 LLM)
- **LLM calls saved** (estimated cost reduction)
- **Confidence thresholds** per agent (RSI adjustments)
- **Event timeline** (what happened, when)

### Database

SQLite at `~/.local/share/aas/aas.db`:

```bash
sqlite3 ~/.local/share/aas/aas.db
> SELECT agent_name, success_rate FROM agent_stats;
```

Tables:
- `issues` — detected problems
- `actions` — planned & executed fixes
- `analyses` — LLM outputs
- `patterns` — learned solutions
- `cycle_performance` — RSI metrics

## FAQ

**Q: Is this safe to run on production?**  
A: No, not yet. Currently safe for development/staging. Agent actions are real (git commits, service restarts), so verify carefully. Agents can be disabled per-domain.

**Q: What if an agent breaks something?**  
A: Every action that succeeds verification gets cached. Failures don't get cached. If a cached solution fails next time, it means the world changed (repo state, error type, etc.). Run `aas run --dry-run` first.

**Q: How do I add a custom agent?**  
A: Implement the `Agent` trait in `src/agents/my_agent.rs`, register in `main.rs`, add to config.

**Q: Can agents learn from failures too?**  
A: Currently no. Failures don't get cached. Future: track failure patterns and adjust detection/planning accordingly.

**Q: How much does this cost?**  
A: Depends on your agent config. With caching, ~90% fewer LLM calls than naive approach. See cost model above.

## Roadmap

- [x] MVP learning (cache + reuse)
- [x] Multi-provider LLM routing
- [ ] Cross-agent discovery (agents learn from each other)
- [ ] Meta-improvement (agents improving other agents)
- [ ] Failure pattern learning
- [ ] Export metrics to Prometheus
- [ ] Web dashboard
- [ ] Slack/Discord integration helpers

## License

MIT

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)

---

**Built with**: Rust, Tokio, SQLite, Tracing  
**Maintained by**: [The Great Labs](https://thealxlabs.ca)
