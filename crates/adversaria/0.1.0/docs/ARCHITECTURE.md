# Adversaria Architecture

This document describes the architecture and design decisions of Adversaria.

## Overview

Adversaria is built as a modular Rust application with clear separation of concerns. The architecture follows a layered approach with well-defined interfaces between components.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                         CLI Layer                            │
│  (commands: run, list, report)                              │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│                      Core Layer                              │
│  - Types (AttackPayload, TestRun, etc.)                     │
│  - Configuration Management                                  │
│  - Error Handling                                           │
│  - Plugin System                                            │
└────┬─────────────┬─────────────┬──────────────┬────────────┘
     │             │             │              │
┌────▼────┐  ┌────▼────┐  ┌────▼────┐  ┌──────▼──────┐
│Providers│  │ Suites  │  │Reporter │  │   Plugins   │
│         │  │         │  │         │  │             │
│- OpenAI │  │- Loader │  │- JSON   │  │- Custom     │
│- Anthro │  │- Runner │  │- Format │  │  Suites     │
│- Ollama │  │         │  │         │  │             │
└─────────┘  └─────────┘  └─────────┘  └─────────────┘
```

## Components

### 1. CLI Layer (`src/cli/`)

**Responsibility**: User interface and command handling

**Components**:
- `mod.rs`: Main CLI structure using clap
- `commands/run.rs`: Execute attack suites
- `commands/list.rs`: List available suites
- `commands/report.rs`: View and manage reports

**Key Features**:
- Argument parsing with clap
- Colored terminal output
- Progress indicators
- Error display

### 2. Core Layer (`src/core/`)

**Responsibility**: Core types, configuration, and business logic

**Components**:
- `types.rs`: Core data structures
  - `AttackPayload`: Individual attack definition
  - `AttackSuite`: Collection of related attacks
  - `TestRun`: Results of a test execution
  - `AttackResult`: Individual attack result
  - `CategorySummary`: Aggregated statistics
  
- `config.rs`: Configuration management
  - YAML-based configuration
  - Provider settings
  - Suite configuration
  - Reporting options
  
- `error.rs`: Error types and handling
  - Custom error types
  - Error conversion
  - Result type alias
  
- `plugin.rs`: Plugin system
  - Plugin trait definition
  - Plugin manager
  - Dynamic loading support

### 3. Providers Layer (`src/providers/`)

**Responsibility**: LLM provider integrations

**Architecture**:
```rust
trait Provider {
    fn name(&self) -> &str;
    fn model(&self) -> &str;
    async fn generate(&self, prompt: &str) -> Result<ModelResponse>;
    async fn health_check(&self) -> Result<bool>;
}
```

**Implementations**:
- `openai.rs`: OpenAI API integration
- `anthropic.rs`: Anthropic API integration
- `ollama.rs`: Ollama local model integration

**Key Features**:
- Async HTTP clients with reqwest
- Timeout and retry handling
- API key management
- Health checks

### 4. Suites Layer (`src/suites/`)

**Responsibility**: Attack suite management and execution

**Components**:
- `loader.rs`: Load suites from YAML/JSON
  - Directory scanning
  - File parsing
  - Suite filtering
  
- `runner.rs`: Execute attack suites
  - Async execution
  - Progress tracking
  - Result collection
  - Risk scoring

**Execution Flow**:
```
Load Suites → Filter Enabled → For Each Payload:
  ├─ Execute Attack
  ├─ Analyze Response
  ├─ Calculate Risk Score
  └─ Store Result
→ Aggregate Results → Generate Summary
```

### 5. Reporters Layer (`src/reporters/`)

**Responsibility**: Report generation and formatting

**Components**:
- `json_reporter.rs`: JSON report generation
  - Structured output
  - Pretty printing
  - File management
  
**Report Structure**:
```json
{
  "id": "uuid",
  "model": "string",
  "provider": "string",
  "timestamp": "datetime",
  "overall_risk_score": 0-100,
  "results": [...],
  "category_summary": {...}
}
```

## Data Flow

### Test Execution Flow

```
1. User runs: adversaria run --provider openai
   ↓
2. CLI parses arguments and loads config
   ↓
3. Create provider instance (OpenAI)
   ↓
4. Load attack suites from directory
   ↓
5. Filter enabled suites
   ↓
6. For each suite:
   For each payload:
     a. Send prompt to provider
     b. Receive response
     c. Analyze response for success/failure
     d. Calculate risk score
     e. Store result
   ↓
7. Aggregate results by category
   ↓
8. Calculate overall risk score
   ↓
9. Generate report
   ↓
10. Display summary to user
```

### Configuration Loading

```
1. Check for config file path (CLI arg or default)
   ↓
2. If exists: Load and parse YAML
   If not: Create default config
   ↓
3. Validate configuration
   ↓
4. Override with environment variables (API keys)
   ↓
5. Return Config object
```

## Design Decisions

### 1. Async/Await with Tokio

**Rationale**: LLM API calls are I/O bound and benefit from async execution.

**Benefits**:
- Efficient resource usage
- Concurrent request handling
- Better scalability

### 2. YAML for Configuration

**Rationale**: Human-readable, supports comments, widely used.

**Benefits**:
- Easy to edit
- Supports complex structures
- Good tooling support

### 3. Trait-Based Provider System

**Rationale**: Extensibility and testability.

**Benefits**:
- Easy to add new providers
- Mockable for testing
- Clear interface contract

### 4. Severity-Based Risk Scoring

**Rationale**: Simple, understandable metric.

**Algorithm**:
```
Risk Score = (Sum of successful attack severities) / (Total possible risk) * 100

Where:
- Low severity = 25 points
- Medium severity = 50 points
- High severity = 75 points
- Critical severity = 100 points
```

### 5. Plugin System

**Rationale**: Allow users to extend without modifying core.

**Design**:
- Trait-based plugins
- Dynamic loading support
- Isolated execution

## Security Considerations

### API Key Management

- Never hardcode API keys
- Support environment variables
- Warn if keys are in config files
- Consider using secret management tools

### Rate Limiting

- Respect provider rate limits
- Implement backoff strategies
- Allow configuration of delays

### Data Privacy

- Don't log sensitive data
- Sanitize reports
- Allow report encryption

## Performance Considerations

### Async Execution

- Use Tokio for async runtime
- Batch requests when possible
- Implement connection pooling

### Caching

- Cache provider responses (optional)
- Cache loaded suites
- Reuse HTTP clients

### Memory Management

- Stream large responses
- Limit concurrent requests
- Clean up old reports

## Testing Strategy

### Unit Tests

- Test individual functions
- Mock external dependencies
- Cover edge cases

### Integration Tests

- Test component interactions
- Use test fixtures
- Verify end-to-end flows

### Property-Based Tests

- Test invariants
- Generate random inputs
- Verify properties hold

## Future Enhancements

### Planned Features

1. **Web UI**: Browser-based interface for viewing reports
2. **Real-time Monitoring**: Live dashboard during test execution
3. **Benchmark Mode**: Compare multiple models
4. **Custom Scoring**: User-defined risk algorithms
5. **Report Export**: PDF, HTML, CSV formats
6. **CI/CD Integration**: GitHub Actions, GitLab CI support
7. **Database Backend**: Store results in database
8. **API Server**: REST API for remote execution

### Extensibility Points

1. **New Providers**: Implement `Provider` trait
2. **Custom Reporters**: Implement `Reporter` trait
3. **Attack Suites**: Add YAML files
4. **Plugins**: Implement `Plugin` trait
5. **Scoring Algorithms**: Extend `SuiteRunner`

## Dependencies

### Core Dependencies

- `tokio`: Async runtime
- `clap`: CLI framework
- `serde`: Serialization
- `reqwest`: HTTP client
- `anyhow`: Error handling

### Why These Dependencies?

- **Tokio**: Industry standard for async Rust
- **Clap**: Best CLI framework with derive macros
- **Serde**: Universal serialization
- **Reqwest**: Reliable HTTP client
- **Anyhow**: Simple error handling

## Maintenance

### Code Organization

- Keep modules focused
- Limit file size (< 500 lines)
- Clear naming conventions
- Comprehensive documentation

### Versioning

- Follow Semantic Versioning
- Maintain CHANGELOG
- Tag releases
- Document breaking changes

### Backwards Compatibility

- Maintain config file compatibility
- Version report format
- Deprecate features gracefully
- Provide migration guides

## Conclusion

Adversaria's architecture prioritizes:
- **Modularity**: Clear separation of concerns
- **Extensibility**: Easy to add new features
- **Testability**: Comprehensive test coverage
- **Performance**: Efficient async execution
- **Usability**: Clear CLI and reports

This design allows for rapid development while maintaining code quality and user experience.
