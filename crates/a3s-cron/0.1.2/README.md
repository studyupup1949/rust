# a3s-cron

Cron scheduling library for A3S with natural language support.

[![Crates.io](https://img.shields.io/crates/v/a3s-cron.svg)](https://crates.io/crates/a3s-cron)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- **Standard Cron Syntax**: 5-field cron expressions (minute hour day month weekday)
- **Natural Language**: Parse schedules from English and Chinese
- **Persistence**: JSON file-based storage with pluggable backends
- **CRUD Operations**: Create, pause, resume, update, and remove jobs
- **Execution History**: Track job runs with output and status
- **Agent-Mode Jobs**: Schedule AI agent prompts alongside shell commands via `AgentExecutor` trait
- **85 Unit Tests**: Comprehensive test coverage

## Installation

```toml
[dependencies]
a3s-cron = "0.1"
```

## Library Usage

### Shell Jobs

```rust
use a3s_cron::{CronManager, FileCronStore, parse_natural};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse natural language to cron expression
    let cron = parse_natural("every day at 2am")?;  // Returns "0 2 * * *"
    let cron = parse_natural("每天凌晨2点")?;        // Returns "0 2 * * *"

    // Create a manager with file-based storage
    let manager = CronManager::new("/path/to/workspace").await?;

    // Add a shell job
    let job = manager.add_job("backup", "0 2 * * *", "backup.sh").await?;

    // List jobs
    let jobs = manager.list_jobs().await?;

    // Pause/resume
    manager.pause_job(&job.id).await?;
    manager.resume_job(&job.id).await?;

    // Manual execution
    let execution = manager.run_job(&job.id).await?;
    println!("Exit code: {}", execution.exit_code.unwrap_or(-1));

    // Get execution history
    let history = manager.get_history(&job.id, 10).await?;

    Ok(())
}
```

### Agent-Mode Jobs

```rust
use a3s_cron::{CronManager, AgentJobConfig, AgentExecutor};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = CronManager::new("/path/to/workspace").await?;

    // Set an agent executor (implement AgentExecutor trait)
    // manager.set_agent_executor(Arc::new(MyAgentExecutor::new()));

    // Add an agent-mode job — the command is used as the agent prompt
    let config = AgentJobConfig {
        model: "claude-sonnet-4-20250514".to_string(),
        api_key: "sk-ant-...".to_string(),
        workspace: None,
        system_prompt: None,
        base_url: None,
    };
    let job = manager.add_agent_job(
        "daily-review",
        "0 9 * * 1-5",
        "Review open PRs and summarize status",
        config,
    ).await?;

    // Start the scheduler
    manager.start().await?;

    Ok(())
}
```

## CLI Usage (via a3s-tools)

```bash
# Parse natural language to cron expression
TOOL_ARGS='{"action":"parse","input":"every day at 2am"}' a3s-tools cron
# Output: Cron expression: 0 2 * * *

TOOL_ARGS='{"action":"parse","input":"每周一上午9点"}' a3s-tools cron
# Output: Cron expression: 0 9 * * 1

# Create a job with natural language schedule
TOOL_ARGS='{"action":"add","name":"backup","schedule":"every day at 3am","command":"./backup.sh"}' a3s-tools cron

# List all jobs
TOOL_ARGS='{"action":"list"}' a3s-tools cron

# Get job details
TOOL_ARGS='{"action":"get","id":"<job-id>"}' a3s-tools cron

# Manually run a job
TOOL_ARGS='{"action":"run","id":"<job-id>"}' a3s-tools cron

# View execution history
TOOL_ARGS='{"action":"history","id":"<job-id>"}' a3s-tools cron

# Pause a job
TOOL_ARGS='{"action":"pause","id":"<job-id>"}' a3s-tools cron

# Resume a job
TOOL_ARGS='{"action":"resume","id":"<job-id>"}' a3s-tools cron

# Update job schedule
TOOL_ARGS='{"action":"update","id":"<job-id>","schedule":"every monday at 9am"}' a3s-tools cron

# Remove a job
TOOL_ARGS='{"action":"remove","id":"<job-id>"}' a3s-tools cron
```

## Natural Language Support

### English

| Expression | Cron | Description |
|------------|------|-------------|
| `every minute` | `* * * * *` | Every minute |
| `every 5 minutes` | `*/5 * * * *` | Every 5 minutes |
| `every hour` | `0 * * * *` | Every hour |
| `every 2 hours` | `0 */2 * * *` | Every 2 hours |
| `daily at 2am` | `0 2 * * *` | Daily at 2:00 AM |
| `every day at 14:30` | `30 14 * * *` | Daily at 2:30 PM |
| `weekly on monday at 9am` | `0 9 * * 1` | Every Monday at 9:00 AM |
| `monthly on the 15th` | `0 0 15 * *` | 15th of every month |
| `every weekday at 8am` | `0 8 * * 1-5` | Mon-Fri at 8:00 AM |
| `every weekend at 10am` | `0 10 * * 0,6` | Sat-Sun at 10:00 AM |

### Chinese (中文)

| 表达式 | Cron | 描述 |
|--------|------|------|
| `每分钟` | `* * * * *` | 每分钟执行 |
| `每5分钟` | `*/5 * * * *` | 每5分钟执行 |
| `每小时` | `0 * * * *` | 每小时执行 |
| `每2小时` | `0 */2 * * *` | 每2小时执行 |
| `每天凌晨2点` | `0 2 * * *` | 每天凌晨2点 |
| `每天下午3点30分` | `30 15 * * *` | 每天下午3:30 |
| `每周一上午9点` | `0 9 * * 1` | 每周一上午9点 |
| `每月15号` | `0 0 15 * *` | 每月15号 |
| `工作日上午9点` | `0 9 * * 1-5` | 工作日上午9点 |
| `周末上午10点` | `0 10 * * 0,6` | 周末上午10点 |

## Cron Expression Format

Standard 5-field cron format:

```
┌───────────── minute (0-59)
│ ┌───────────── hour (0-23)
│ │ ┌───────────── day of month (1-31)
│ │ │ ┌───────────── month (1-12)
│ │ │ │ ┌───────────── day of week (0-6, Sun=0)
│ │ │ │ │
* * * * *
```

Special characters:
- `*` - any value
- `,` - value list (e.g., `1,3,5`)
- `-` - range (e.g., `1-5`)
- `/` - step (e.g., `*/5`)

## Architecture

```
a3s-cron/
├── src/
│   ├── lib.rs        # Public API
│   ├── types.rs      # CronJob, JobType, AgentJobConfig, AgentExecutor
│   ├── parser.rs     # Cron expression parser
│   ├── natural.rs    # Natural language parser
│   ├── store.rs      # CronStore trait, FileCronStore, MemoryCronStore
│   ├── scheduler.rs  # CronManager with CRUD + agent-mode execution
│   └── telemetry.rs  # OpenTelemetry metrics and spans
└── Cargo.toml
```

## Roadmap

### Phase 1: Core ✅

- [x] Standard 5-field cron expression parsing
- [x] Natural language schedule parsing (English + Chinese)
- [x] JSON file-based persistence with pluggable backends
- [x] CRUD operations (create, pause, resume, update, remove)
- [x] Execution history tracking with output and status
- [x] CLI integration via a3s-tools
- [x] Agent-mode jobs via `AgentExecutor` trait (Shell + Agent job types)
- [x] 85 comprehensive unit tests

### Phase 2: Distributed Scheduling 📋

- [ ] **Cluster-aware Scheduling**: Multi-node job distribution
  - [ ] Leader election for scheduler coordination (etcd / NATS)
  - [ ] Job assignment with node affinity and load balancing
  - [ ] Failover: automatic job reassignment on node failure
  - [ ] Exactly-once execution guarantee (distributed lock)
- [ ] **Advanced Scheduling Strategies**:
  - [ ] Job dependency chains (Job B runs after Job A completes)
  - [ ] Conditional execution (run only if previous job succeeded/failed)
  - [ ] Job groups with shared concurrency limits
  - [ ] Backfill: catch up missed executions after downtime
- [ ] **Observability**:
  - [ ] OpenTelemetry spans for job execution lifecycle
  - [ ] Span: `a3s.cron.execute` with attributes: job_id, job_name, schedule, duration_ms
  - [ ] Metrics: `a3s_cron_job_duration_seconds{job}` histogram
  - [ ] Metrics: `a3s_cron_job_failures_total{job}` counter
  - [ ] Metrics: `a3s_cron_missed_executions_total{job}` counter
- [ ] **Storage Backends**:
  - [ ] Redis backend for distributed state
  - [ ] PostgreSQL backend for durable persistence
  - [ ] Migration tool between storage backends

## License

MIT
