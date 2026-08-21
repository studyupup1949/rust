# Adversaria Usage Guide

Complete guide for using Adversaria to test LLM security.

## Table of Contents

1. [Installation](#installation)
2. [Configuration](#configuration)
3. [Basic Usage](#basic-usage)
4. [Advanced Usage](#advanced-usage)
5. [Understanding Reports](#understanding-reports)
6. [Best Practices](#best-practices)
7. [Troubleshooting](#troubleshooting)

## Installation

### From Source

```bash
git clone https://github.com/adversaria/adversaria.git
cd adversaria
cargo build --release
cargo install --path .
```

### Verify Installation

```bash
adversaria --version
```

## Configuration

### Initial Setup

Create a configuration file:

```bash
adversaria list  # This will create adversaria.config.yaml if it doesn't exist
```

### Configuration File

Edit `adversaria.config.yaml`:

```yaml
version: "1.0"
default_provider: openai

providers:
  openai:
    api_key: null  # Set via OPENAI_API_KEY env var
    model: gpt-4
    timeout_seconds: 30
    
  anthropic:
    api_key: null  # Set via ANTHROPIC_API_KEY env var
    model: claude-3-opus-20240229
    
  ollama:
    api_base: http://localhost:11434
    model: llama2

suites:
  directory: ./suites
  enabled_suites:
    - prompt_injection
    - jailbreak
    - role_confusion
    - data_exfiltration
```

### Environment Variables

Set API keys:

```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
```

Or add to `.env` file:

```bash
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
```

## Basic Usage

### List Available Suites

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
```

### Run Tests

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

### Run Specific Suites

```bash
adversaria run --suites prompt_injection,jailbreak
```

### View Reports

List all reports:
```bash
adversaria report --list
```

View specific report:
```bash
adversaria report adversaria_report_20240224_143022_abc123.json
```

View with details:
```bash
adversaria report adversaria_report_20240224_143022_abc123.json --verbose
```

## Advanced Usage

### Custom Configuration Path

```bash
adversaria run --config /path/to/custom/config.yaml
```

### Skip Saving Reports

```bash
adversaria run --no-save
```

### Detailed Suite Information

```bash
adversaria list --verbose
```

### Testing Multiple Models

Create a script to test multiple models:

```bash
#!/bin/bash

models=("gpt-4" "gpt-3.5-turbo" "claude-3-opus-20240229")

for model in "${models[@]}"; do
    echo "Testing $model..."
    adversaria run --provider openai --model "$model"
done
```

### Custom Attack Suites

Create `custom_suite.yaml`:

```yaml
id: my_custom_suite
name: My Custom Suite
description: Custom attacks for my use case
category: custom
enabled: true
payloads:
  - id: custom_001
    name: Custom Attack 1
    description: My custom attack
    prompt: "Your custom prompt here"
    category: custom
    severity: high
    tags: [custom]
    metadata:
      technique: custom_technique
```

Place in `suites/` directory and enable in config:

```yaml
suites:
  enabled_suites:
    - my_custom_suite
```

### Batch Testing

Test multiple providers:

```bash
#!/bin/bash

providers=("openai" "anthropic" "ollama")

for provider in "${providers[@]}"; do
    echo "Testing $provider..."
    adversaria run --provider "$provider"
done
```

### Automated Testing

Add to CI/CD pipeline:

```yaml
# .github/workflows/llm-security-test.yml
name: LLM Security Test

on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Adversaria
        run: cargo install adversaria
      - name: Run Tests
        env:
          OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
        run: adversaria run --provider openai
```

## Understanding Reports

### Report Structure

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
  "results": [...],
  "category_summary": {...},
  "duration_ms": 45000
}
```

### Risk Score Interpretation

- **0-25 (Low)**: ✅ Model has strong defenses
- **26-50 (Medium)**: ⚠️ Some vulnerabilities detected
- **51-75 (High)**: 🔴 Significant vulnerabilities
- **76-100 (Critical)**: 🚨 Severe security issues

### Category Summary

Each category shows:
- Total attacks attempted
- Successful attacks
- Average risk score
- Maximum severity level

### Individual Results

Each attack result includes:
- Payload ID and name
- Category and severity
- Original prompt
- Model response
- Success/failure status
- Risk score
- Execution time
- Detection reason (if failed)

## Best Practices

### 1. Start with Default Suites

Begin with built-in suites to establish baseline:

```bash
adversaria run --provider openai
```

### 2. Test Regularly

Schedule regular security tests:
- Weekly for production models
- After each model update
- Before major releases

### 3. Compare Models

Test multiple models to compare security:

```bash
adversaria run --provider openai --model gpt-4
adversaria run --provider openai --model gpt-3.5-turbo
```

### 4. Review Reports Carefully

Pay attention to:
- Successful attacks (security issues)
- Patterns in failures
- Category-specific vulnerabilities

### 5. Create Custom Suites

Develop suites specific to your use case:
- Industry-specific attacks
- Application-specific scenarios
- Known vulnerability patterns

### 6. Monitor Trends

Track risk scores over time:
- Are they improving?
- New vulnerabilities appearing?
- Impact of model updates

### 7. Document Findings

Keep records of:
- Test results
- Mitigation strategies
- Model configurations
- Security incidents

### 8. Respect Rate Limits

Configure appropriate timeouts:

```yaml
providers:
  openai:
    timeout_seconds: 30
    max_retries: 3
```

### 9. Secure API Keys

Never commit API keys:
- Use environment variables
- Use secret management tools
- Rotate keys regularly

### 10. Test in Isolation

Test in controlled environment:
- Separate test accounts
- Isolated API keys
- Non-production models

## Troubleshooting

### API Key Issues

**Problem**: "API key not found"

**Solution**:
```bash
export OPENAI_API_KEY="your-key"
# or
echo 'OPENAI_API_KEY=your-key' >> .env
```

### Connection Errors

**Problem**: "Failed to connect to provider"

**Solution**:
1. Check internet connection
2. Verify API endpoint
3. Check firewall settings
4. Increase timeout:

```yaml
providers:
  openai:
    timeout_seconds: 60
```

### Rate Limiting

**Problem**: "Rate limit exceeded"

**Solution**:
1. Reduce concurrent requests
2. Add delays between tests
3. Upgrade API plan
4. Use different API key

### Suite Loading Errors

**Problem**: "Failed to load suite"

**Solution**:
1. Check YAML syntax
2. Verify file permissions
3. Check file path in config
4. Validate suite structure

### Report Generation Errors

**Problem**: "Failed to save report"

**Solution**:
1. Check output directory exists
2. Verify write permissions
3. Check disk space
4. Ensure valid JSON structure

### Ollama Connection Issues

**Problem**: "Cannot connect to Ollama"

**Solution**:
1. Start Ollama: `ollama serve`
2. Check port: `http://localhost:11434`
3. Verify model installed: `ollama list`
4. Pull model: `ollama pull llama2`

### Memory Issues

**Problem**: "Out of memory"

**Solution**:
1. Run fewer suites at once
2. Reduce payload size
3. Increase system memory
4. Use streaming responses

### Slow Execution

**Problem**: Tests taking too long

**Solution**:
1. Reduce timeout values
2. Run specific suites only
3. Use faster models
4. Optimize network connection

## Examples

### Example 1: Quick Security Check

```bash
# Quick test with default settings
adversaria run
```

### Example 2: Comprehensive Test

```bash
# Test all suites with verbose output
adversaria run --provider openai --model gpt-4
adversaria report --list
adversaria report <latest-report> --verbose
```

### Example 3: Targeted Testing

```bash
# Test specific vulnerability
adversaria run --suites prompt_injection
```

### Example 4: Comparison Testing

```bash
# Compare two models
adversaria run --provider openai --model gpt-4
adversaria run --provider anthropic --model claude-3-opus-20240229
```

### Example 5: Custom Suite Testing

```bash
# Test with custom suite
adversaria run --suites my_custom_suite
```

## Getting Help

### Command Help

```bash
adversaria --help
adversaria run --help
adversaria list --help
adversaria report --help
```

### Debug Mode

Enable debug logging:

```bash
RUST_LOG=adversaria=debug adversaria run
```

### Community Support

- GitHub Issues: Report bugs
- Discussions: Ask questions
- Documentation: Read guides
- Examples: Check examples/

## Next Steps

1. Run your first test
2. Review the report
3. Create custom suites
4. Integrate into CI/CD
5. Monitor regularly
6. Share findings

Happy testing! 🔍
