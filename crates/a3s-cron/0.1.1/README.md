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
- **71 Unit Tests**: Comprehensive test coverage

## Installation

```toml
[dependencies]
a3s-cron = "0.1"
```

## Library Usage

```rust
use a3s_cron::{CronManager, FileCronStore, parse_natural};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse natural language to cron expression
    let cron = parse_natural("every day at 2am")?;  // Returns "0 2 * * *"
    let cron = parse_natural("每天凌晨2点")?;        // Returns "0 2 * * *"

    // Create a manager with file-based storage
    let store = FileCronStore::new("/path/to/storage").await?;
    let manager = CronManager::new(store);

    // Add a job
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
│   ├── types.rs      # CronJob, JobStatus, JobExecution
│   ├── parser.rs     # Cron expression parser
│   ├── natural.rs    # Natural language parser
│   ├── store.rs      # CronStore trait, FileCronStore, MemoryCronStore
│   └── scheduler.rs  # CronManager with CRUD operations
└── Cargo.toml
```

## License

MIT
