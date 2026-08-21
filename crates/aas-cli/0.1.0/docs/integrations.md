# AAS Integrations

Connect AAS to external tools and services.

## Quick Start

```bash
# List available integrations
aas integrations

# Connect Claude Code (for code editing)
aas connect claude-code

# Connect OpenClaw (for external tasks)
aas connect openclaw --endpoint http://localhost:3001

# Check current connections
aas config show integrations

# Disconnect an integration
aas disconnect openclaw
```

## Claude Code

Enable code editing and file modifications via Claude Code CLI.

### Setup

1. Install Claude Code
   ```bash
   # macOS
   brew install anthropics/tap/claude

   # Or download from https://claude.ai
   ```

2. Verify installation
   ```bash
   claude --version
   ```

3. Connect to AAS
   ```bash
   aas connect claude-code
   ✓ Found Claude Code at: /opt/homebrew/bin/claude
   ✓ Config updated
   ```

### Usage

When agents need to edit code, they route to Claude Code:

```rust
// Agents use TaskType::CodeEdit
let provider = ctx.router.route(TaskType::CodeEdit);
let response = provider.chat(&messages, &options).await?;
```

### Examples

- **Fix Python tests**: Agent detects test failure, uses Claude Code to generate fix
- **Generate boilerplate**: Agent needs new file, Claude Code generates it
- **Reformat code**: Agent detects style issue, Claude Code fixes formatting

## OpenClaw

Delegate external tasks (Slack, Discord, webhooks, etc.) to OpenClaw.

### Setup

1. Start OpenClaw server
   ```bash
   # OpenClaw must be running separately
   openclaw start --port 3001
   ```

2. Verify connection
   ```bash
   curl http://localhost:3001/health
   ```

3. Connect to AAS
   ```bash
   aas connect openclaw --endpoint http://localhost:3001
   ✓ OpenClaw responding at http://localhost:3001
   ✓ Config updated
   ```

4. (Optional) Add API key for authentication
   ```bash
   aas connect openclaw --token "your-api-key"
   ```

### Configuration

Edit `~/.config/aas/config.toml`:

```toml
[integrations.openclaw]
enabled = true
endpoint = "http://localhost:3001"
api_key = "optional-auth-token"
timeout_secs = 30
```

### Usage

When agents need external execution:

```rust
// Agents use TaskType::ExternalTask
let provider = ctx.router.route(TaskType::ExternalTask);
let response = provider.chat(&messages, &options).await?;
```

### Examples

- **Slack notifications**: Service down → send alert to #ops
- **Discord webhooks**: New learned pattern → notify team
- **HTTP callbacks**: Action completed → trigger downstream job
- **SMS/Email**: Critical issue → page oncall engineer

## Adding Custom Integrations

### Implement the LLMProvider Trait

```rust
// src/llm/providers/my_provider.rs
use crate::llm::traits::LLMProvider;
use async_trait::async_trait;

pub struct MyProvider {
    // Your fields
}

#[async_trait]
impl LLMProvider for MyProvider {
    async fn chat(
        &self,
        messages: &[Message],
        options: &LLMOptions,
    ) -> Result<LLMResponse, String> {
        // Your implementation
    }
}
```

### Register in LLMRouter

```rust
// src/llm/router.rs
pub fn build_llm_router(config: &Config) -> Arc<LLMRouter> {
    let mut providers = HashMap::new();

    // ... existing providers ...

    if config.integrations.my_provider.enabled {
        providers.insert(
            TaskType::MyTask,
            Arc::new(MyProvider::new(&config.integrations.my_provider)),
        );
    }

    Arc::new(LLMRouter::new(providers))
}
```

### Add to Config

```toml
[integrations.my_provider]
enabled = false
endpoint = "http://localhost:9000"
```

### Create Setup CLI Command

```rust
// src/cli/integrations.rs
"my-provider" => {
    println!("Setting up My Provider...");
    // Your setup logic
}
```

## Integration Matrix

| Integration | Purpose | Type | Status |
|-------------|---------|------|--------|
| Claude Code | Code editing | LLM Provider | ✅ Implemented |
| OpenClaw | External tasks | LLM Provider | ✅ Implemented |
| Claude API | Deep reasoning | LLM Provider | ✅ Implemented |
| Hermes | Fast local LLM | LLM Provider | ✅ Implemented |
| Slack | Notifications | External (via OpenClaw) | ⏳ Coming |
| Discord | Notifications | External (via OpenClaw) | ⏳ Coming |
| PagerDuty | Alerting | External (via OpenClaw) | ⏳ Coming |
| GitHub | PR/Issue ops | External (via OpenClaw) | ⏳ Coming |

## Troubleshooting

### Claude Code not found

```bash
# Check PATH
which claude

# Add to PATH if needed
export PATH="$PATH:/opt/homebrew/bin"

# Try again
aas connect claude-code
```

### OpenClaw connection failed

```bash
# Check if server is running
curl http://localhost:3001/health

# Check firewall
lsof -i :3001

# Verify endpoint
aas connect openclaw --endpoint http://your-host:3001
```

### Integration disabled after config change

```bash
# View config
aas config show integrations

# Re-enable
aas connect <name>

# Or edit manually
aas config edit
```

## Performance Notes

- **Claude Code**: Subprocess overhead ~100ms, fast code generation
- **OpenClaw**: Network latency + OpenClaw processing, good for async tasks
- **Caching**: Learned solutions bypass integrations entirely (0 LLM calls)

Plan ahead: Common tasks should be cached, not delegated each time.
