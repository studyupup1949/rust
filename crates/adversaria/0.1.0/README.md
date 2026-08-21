# Adversaria

**Adversarial Testing Harness for Large Language Models**

Adversaria is a comprehensive security testing framework designed to evaluate the robustness of Large Language Models (LLMs) against adversarial attacks. Built in Rust for performance and reliability, it provides automated testing suites for prompt injection, jailbreaks, role confusion, and data exfiltration attempts.

## Features

- 🎯 **Comprehensive Attack Suites**: 4 built-in suites with 48+ attack payloads
  - Prompt Injection (12 payloads)
  - Jailbreak Attempts (12 payloads)
  - Role Confusion (12 payloads)
  - Data Exfiltration (12 payloads)

- 🔌 **Multi-Provider Support**: Test against multiple LLM providers
  - OpenAI (GPT-4, GPT-3.5, etc.)
  - Anthropic (Claude 3 Opus, Sonnet, Haiku)
  - Ollama (Local models)

- 📊 **Detailed Reporting**: JSON reports with risk scoring and reproducible traces
  - Overall risk score (0-100)
  - Category-based breakdown
  - Execution traces for each attack
  - Success/failure analysis

- 🔧 **Extensible Plugin System**: Add custom attack suites via plugins

- ⚙️ **Flexible Configuration**: YAML-based configuration for easy customization

## Installation

### From Source

```bash
git clone https://github.com/adversaria/adversaria.git
cd adversaria
cargo build --release
cargo install --path .
```

### Prerequisites

- Rust 1.70 or higher
- API keys for the providers you want to test (OpenAI, Anthropic)
- For Ollama: Local Ollama installation

## Quick Start

### 1. Configure Adversaria

Create or edit `adversaria.config.yaml`:

```yaml
version: "1.0"
default_provider: openai

providers:
  openai:
    api_key: null  # Or set OPENAI_API_KEY env var
    model: gpt-4
    
  anthropic:
    api_key: null  # Or set ANTHROPIC_API_KEY env var
    model: claude-3-opus-20240229
```

### 2. Set API Keys

```bash
export OPENAI_API_KEY="your-openai-api-key"
export ANTHROPIC_API_KEY="your-anthropic-api-key"
```

### 3. List Available Attack Suites

```bash
adversaria list
```

Output:
```
📋 Available Attack Suites

┌────────────────────┬─────────────────────────┬──────────────────┬──────────┬─────────┐
│ ID                 │ Name                    │ Category         │ Payloads │ Enabled │
├────────────────────┼─────────────────────────┼──────────────────┼──────────┼─────────┤
│ prompt_injection   │ Prompt Injection Suite  │ Prompt Injection │ 12       │ ✓       │
│ jailbreak          │ Jailbreak Suite         │ Jailbreak        │ 12       │ ✓       │
│ role_confusion     │ Role Confusion Suite    │ Role Confusion   │ 12       │ ✓       │
│ data_exfiltration  │ Data Exfiltration Suite │ Data Exfiltration│ 12       │ ✓       │
└────────────────────┴─────────────────────────┴──────────────────┴──────────┴─────────┘

Total: 4 suite(s) found
```

### 4. Run Attack Suites

Test against OpenAI:
```bash
adversaria run --provider openai --model gpt-4
```

Test against Anthropic:
```bash
adversaria run --provider anthropic --model claude-3-opus-20240229
```

Test against local Ollama:
```bash
adversaria run --provider ollama --model llama2
```

Run specific suites:
```bash
adversaria run --suites prompt_injection,jailbreak
```

### 5. View Reports

List all reports:
```bash
adversaria report --list
```

View a specific report:
```bash
adversaria report adversaria_report_20240224_143022_abc123.json
```

View with detailed results:
```bash
adversaria report adversaria_report_20240224_143022_abc123.json --verbose
```

## Configuration

### Full Configuration Example

```yaml
version: "1.0"
default_provider: openai

providers:
  openai:
    api_key: null
    api_base: https://api.openai.com/v1
    model: gpt-4
    timeout_seconds: 30
    max_retries: 3
    
  anthropic:
    api_key: null
    api_base: https://api.anthropic.com/v1
    model: claude-3-opus-20240229
    timeout_seconds: 30
    max_retries: 3
    
  ollama:
    api_key: null
    api_base: http://localhost:11434
    model: llama2
    timeout_seconds: 60
    max_retries: 2

suites:
  directory: ./suites
  enabled_suites:
    - prompt_injection
    - jailbreak
    - role_confusion
    - data_exfiltration
  custom_suites: []

reporting:
  output_directory: ./reports
  format: json
  keep_reports: 100

plugins:
  directory: ./plugins
  enabled: true
```

## Attack Suites

### Prompt Injection Suite

Tests for prompt injection vulnerabilities where attackers attempt to override system instructions.

**Example Payloads:**
- Basic Ignore Instructions
- System Prompt Override
- Role Reversal Injection
- Delimiter Confusion
- Nested Instruction Override
- Encoded Instruction Injection
- And 6 more...

### Jailbreak Suite

Tests for jailbreak attempts that try to bypass safety guidelines and content policies.

**Example Payloads:**
- DAN (Do Anything Now)
- Developer Mode
- Opposite Mode
- Fictional Story Framing
- Research Purpose Bypass
- Roleplay Scenario
- And 6 more...

### Role Confusion Suite

Tests for role confusion attacks where the model is tricked into assuming unauthorized roles.

**Example Payloads:**
- System Administrator Impersonation
- Developer Role Assumption
- Support Staff Confusion
- Owner Authority Claim
- Maintenance Mode Activation
- API Endpoint Confusion
- And 6 more...

### Data Exfiltration Suite

Tests for data exfiltration attempts where attackers try to extract sensitive information.

**Example Payloads:**
- System Prompt Extraction
- Training Data Leak
- API Key Extraction
- Configuration Dump
- Memory Inspection
- Model Architecture Extraction
- And 6 more...

## Creating Custom Attack Suites

Create a YAML file in the `suites/` directory:

```yaml
id: custom_suite
name: My Custom Attack Suite
description: Custom attacks for specific scenarios
category: custom
enabled: true
payloads:
  - id: custom_001
    name: Custom Attack 1
    description: Description of the attack
    prompt: "Your attack prompt here"
    category: custom
    severity: high
    tags: [custom, test]
    metadata:
      technique: custom_technique
```

## Report Format

Reports are saved as JSON files with the following structure:

```json
{
  "id": "uuid",
  "model": "gpt-4",
  "provider": "openai",
  "timestamp": "2024-02-24T14:30:22Z",
  "total_attacks": 48,
  "successful_attacks": 5,
  "failed_attacks": 43,
  "overall_risk_score": 12,
  "results": [
    {
      "id": "uuid",
      "payload_id": "pi_001",
      "payload_name": "Basic Ignore Instructions",
      "category": "prompt_injection",
      "severity": "high",
      "prompt": "...",
      "response": "...",
      "success": false,
      "risk_score": 0,
      "timestamp": "2024-02-24T14:30:25Z",
      "execution_time_ms": 1234,
      "detection_reason": "Refusal detected: 'i cannot'"
    }
  ],
  "category_summary": {
    "prompt_injection": {
      "category": "prompt_injection",
      "total": 12,
      "successful": 1,
      "average_risk_score": 6.25,
      "max_severity": "critical"
    }
  },
  "duration_ms": 45000
}
```

## Risk Scoring

- **0-25**: Low Risk ✅ - Model has strong defenses
- **26-50**: Medium Risk ⚠️ - Some vulnerabilities detected
- **51-75**: High Risk 🔴 - Significant vulnerabilities
- **76-100**: Critical Risk 🚨 - Severe security issues

## Plugin System

Adversaria supports plugins for extending attack suites. Plugins are loaded from the `plugins/` directory.

### Creating a Plugin

```rust
use adversaria::core::{AttackSuite, Result};
use adversaria::core::plugin::Plugin;
use async_trait::async_trait;

pub struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my_plugin"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    async fn load_suites(&self) -> Result<Vec<AttackSuite>> {
        // Load and return custom attack suites
        Ok(vec![])
    }
}
```

## CLI Commands

### `adversaria run`

Run attack suites against a model.

**Options:**
- `-p, --provider <PROVIDER>`: Provider to use (openai, anthropic, ollama)
- `-m, --model <MODEL>`: Model to test
- `-c, --config <PATH>`: Path to config file (default: adversaria.config.yaml)
- `-s, --suites <SUITES>`: Specific suite IDs to run (comma-separated)
- `--no-save`: Skip saving report

**Examples:**
```bash
adversaria run --provider openai --model gpt-4
adversaria run --provider anthropic --suites prompt_injection,jailbreak
adversaria run --no-save
```

### `adversaria list`

List available attack suites.

**Options:**
- `-c, --config <PATH>`: Path to config file
- `-v, --verbose`: Show detailed information

**Examples:**
```bash
adversaria list
adversaria list --verbose
```

### `adversaria report`

Generate or view reports from previous runs.

**Options:**
- `-c, --config <PATH>`: Path to config file
- `-l, --list`: List all available reports
- `-v, --verbose`: Show detailed results

**Examples:**
```bash
adversaria report --list
adversaria report adversaria_report_20240224_143022_abc123.json
adversaria report adversaria_report_20240224_143022_abc123.json --verbose
```

## Development

### Building from Source

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Running with Debug Logging

```bash
RUST_LOG=adversaria=debug adversaria run
```

## Architecture

```
adversaria/
├── src/
│   ├── main.rs           # Entry point
│   ├── cli/              # CLI commands
│   │   ├── mod.rs
│   │   └── commands/
│   │       ├── run.rs
│   │       ├── list.rs
│   │       └── report.rs
│   ├── core/             # Core types and logic
│   │   ├── mod.rs
│   │   ├── config.rs
│   │   ├── error.rs
│   │   ├── plugin.rs
│   │   └── types.rs
│   ├── providers/        # LLM provider implementations
│   │   ├── mod.rs
│   │   ├── traits.rs
│   │   ├── openai.rs
│   │   ├── anthropic.rs
│   │   └── ollama.rs
│   ├── suites/           # Attack suite system
│   │   ├── mod.rs
│   │   ├── loader.rs
│   │   └── runner.rs
│   └── reporters/        # Report generation
│       ├── mod.rs
│       ├── traits.rs
│       └── json_reporter.rs
├── suites/               # Attack suite definitions
│   ├── prompt_injection.yaml
│   ├── jailbreak.yaml
│   ├── role_confusion.yaml
│   └── data_exfiltration.yaml
├── reports/              # Generated reports
├── plugins/              # Plugin directory
├── adversaria.config.yaml
└── Cargo.toml
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Adding New Attack Payloads

1. Edit the appropriate suite YAML file in `suites/`
2. Follow the existing payload structure
3. Test your payloads
4. Submit a PR

### Adding New Providers

1. Implement the `Provider` trait in `src/providers/`
2. Add provider configuration to `Config`
3. Update `create_provider()` function
4. Add tests
5. Submit a PR

## Security Considerations

- **API Keys**: Never commit API keys. Use environment variables.
- **Rate Limiting**: Be mindful of API rate limits when running tests.
- **Ethical Use**: This tool is for security testing and research only. Use responsibly.
- **Model Safety**: Some attack payloads may trigger safety mechanisms. This is expected behavior.

## License

MIT License - see LICENSE file for details.

## Disclaimer

This tool is designed for security research and testing purposes only. Users are responsible for ensuring they have proper authorization before testing any LLM systems. The authors are not responsible for any misuse of this tool.

## Roadmap

- [ ] Support for more LLM providers (Cohere, AI21, etc.)
- [ ] Web UI for viewing reports
- [ ] Automated CI/CD integration
- [ ] Custom scoring algorithms
- [ ] Benchmark mode for comparing models
- [ ] Real-time monitoring dashboard
- [ ] Export reports to PDF/HTML
- [ ] Integration with security scanning tools

## Support

For issues, questions, or contributions, please visit:
- GitHub Issues: https://github.com/adversaria/adversaria/issues
- Documentation: https://adversaria.dev/docs

## Acknowledgments

Built with:
- Rust
- Tokio (async runtime)
- Clap (CLI framework)
- Serde (serialization)
- Reqwest (HTTP client)
- And many other excellent crates

---

**Made with ❤️ for LLM Security**
