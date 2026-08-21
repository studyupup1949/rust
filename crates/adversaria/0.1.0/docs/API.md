# Adversaria API Documentation

This document describes the internal API for extending Adversaria.

## Core Types

### AttackPayload

Represents a single attack payload.

```rust
pub struct AttackPayload {
    pub id: String,
    pub name: String,
    pub description: String,
    pub prompt: String,
    pub category: AttackCategory,
    pub severity: Severity,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}
```

**Fields**:
- `id`: Unique identifier for the payload
- `name`: Human-readable name
- `description`: Detailed description of the attack
- `prompt`: The actual prompt to send to the LLM
- `category`: Category of attack (PromptInjection, Jailbreak, etc.)
- `severity`: Severity level (Low, Medium, High, Critical)
- `tags`: Tags for categorization
- `metadata`: Additional metadata

### AttackSuite

Collection of related attack payloads.

```rust
pub struct AttackSuite {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: AttackCategory,
    pub payloads: Vec<AttackPayload>,
    pub enabled: bool,
}
```

### AttackResult

Result of executing a single attack.

```rust
pub struct AttackResult {
    pub id: Uuid,
    pub payload_id: String,
    pub payload_name: String,
    pub category: AttackCategory,
    pub severity: Severity,
    pub prompt: String,
    pub response: String,
    pub success: bool,
    pub risk_score: u8,
    pub timestamp: DateTime<Utc>,
    pub execution_time_ms: u64,
    pub detection_reason: Option<String>,
    pub metadata: HashMap<String, String>,
}
```

### TestRun

Complete test execution results.

```rust
pub struct TestRun {
    pub id: Uuid,
    pub model: String,
    pub provider: String,
    pub timestamp: DateTime<Utc>,
    pub total_attacks: usize,
    pub successful_attacks: usize,
    pub failed_attacks: usize,
    pub overall_risk_score: u8,
    pub results: Vec<AttackResult>,
    pub category_summary: HashMap<AttackCategory, CategorySummary>,
    pub duration_ms: u64,
}
```

## Provider Trait

Implement this trait to add a new LLM provider.

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    fn model(&self) -> &str;
    async fn generate(&self, prompt: &str) -> Result<ModelResponse>;
    async fn health_check(&self) -> Result<bool>;
}
```

### Example Implementation

```rust
use adversaria::core::{ModelResponse, Result};
use adversaria::providers::Provider;
use async_trait::async_trait;

pub struct CustomProvider {
    model: String,
    api_key: String,
}

impl CustomProvider {
    pub fn new(model: String, api_key: String) -> Self {
        Self { model, api_key }
    }
}

#[async_trait]
impl Provider for CustomProvider {
    fn name(&self) -> &str {
        "custom"
    }
    
    fn model(&self) -> &str {
        &self.model
    }
    
    async fn generate(&self, prompt: &str) -> Result<ModelResponse> {
        // Implementation here
        Ok(ModelResponse {
            content: "response".to_string(),
            model: self.model.clone(),
            usage: None,
        })
    }
    
    async fn health_check(&self) -> Result<bool> {
        // Implementation here
        Ok(true)
    }
}
```

## Reporter Trait

Implement this trait to create custom report formats.

```rust
pub trait Reporter: Send + Sync {
    fn save_report(&self, test_run: &TestRun) -> Result<String>;
    fn format_summary(&self, test_run: &TestRun) -> String;
}
```

### Example Implementation

```rust
use adversaria::core::{Result, TestRun};
use adversaria::reporters::Reporter;

pub struct CustomReporter {
    output_dir: PathBuf,
}

impl Reporter for CustomReporter {
    fn save_report(&self, test_run: &TestRun) -> Result<String> {
        // Save report in custom format
        let filename = format!("report_{}.txt", test_run.id);
        let path = self.output_dir.join(&filename);
        
        std::fs::write(&path, format!("{:?}", test_run))?;
        
        Ok(path.to_string_lossy().to_string())
    }
    
    fn format_summary(&self, test_run: &TestRun) -> String {
        format!(
            "Test Run: {}\nRisk Score: {}/100",
            test_run.id,
            test_run.overall_risk_score
        )
    }
}
```

## Plugin Trait

Implement this trait to create plugins that load custom attack suites.

```rust
#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    async fn load_suites(&self) -> Result<Vec<AttackSuite>>;
}
```

### Example Implementation

```rust
use adversaria::core::{AttackSuite, Result};
use adversaria::core::plugin::Plugin;
use async_trait::async_trait;

pub struct CustomPlugin;

#[async_trait]
impl Plugin for CustomPlugin {
    fn name(&self) -> &str {
        "custom_plugin"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    async fn load_suites(&self) -> Result<Vec<AttackSuite>> {
        // Load custom suites
        Ok(vec![])
    }
}
```

## Suite Loader

Load attack suites from files.

```rust
use adversaria::suites::SuiteLoader;

// Load single suite
let suite = SuiteLoader::load_suite("path/to/suite.yaml")?;

// Load all suites from directory
let suites = SuiteLoader::load_suites_from_directory("./suites")?;

// Filter enabled suites
let enabled = SuiteLoader::filter_enabled(suites, &["suite1", "suite2"]);
```

## Suite Runner

Execute attack suites against a provider.

```rust
use adversaria::suites::SuiteRunner;
use adversaria::providers;
use std::sync::Arc;

// Create provider
let provider = providers::create_provider("openai", &config)?;

// Create runner
let runner = SuiteRunner::new(provider);

// Run suites
let test_run = runner.run_suites(suites).await?;
```

## Configuration

Load and manage configuration.

```rust
use adversaria::core::Config;

// Load from file
let config = Config::load("adversaria.config.yaml")?;

// Create default
let config = Config::default();

// Create default file
Config::create_default("adversaria.config.yaml")?;
```

## Error Handling

All functions return `Result<T>` where the error type is `AdversariaError`.

```rust
use adversaria::core::error::{AdversariaError, Result};

fn my_function() -> Result<()> {
    // Your code here
    Ok(())
}
```

### Error Types

```rust
pub enum AdversariaError {
    Config(String),
    Provider(String),
    Suite(String),
    Plugin(String),
    Io(std::io::Error),
    Serialization(serde_json::Error),
    Yaml(serde_yaml::Error),
    Http(reqwest::Error),
    InvalidPayload(String),
    ReportNotFound(String),
    Unknown(String),
}
```

## Async Runtime

Adversaria uses Tokio for async execution.

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Your async code here
    Ok(())
}
```

## Examples

### Custom Provider Example

```rust
use adversaria::core::{Config, ModelResponse, Result};
use adversaria::providers::Provider;
use adversaria::suites::{SuiteLoader, SuiteRunner};
use async_trait::async_trait;
use std::sync::Arc;

struct MyProvider {
    model: String,
}

#[async_trait]
impl Provider for MyProvider {
    fn name(&self) -> &str { "my_provider" }
    fn model(&self) -> &str { &self.model }
    
    async fn generate(&self, prompt: &str) -> Result<ModelResponse> {
        Ok(ModelResponse {
            content: format!("Response to: {}", prompt),
            model: self.model.clone(),
            usage: None,
        })
    }
    
    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let provider: Arc<dyn Provider> = Arc::new(MyProvider {
        model: "my-model".to_string(),
    });
    
    let suites = SuiteLoader::load_suites_from_directory("./suites")?;
    let runner = SuiteRunner::new(provider);
    let test_run = runner.run_suites(suites).await?;
    
    println!("Risk Score: {}", test_run.overall_risk_score);
    
    Ok(())
}
```

### Custom Reporter Example

```rust
use adversaria::core::{Result, TestRun};
use adversaria::reporters::Reporter;
use std::path::PathBuf;

struct CsvReporter {
    output_dir: PathBuf,
}

impl Reporter for CsvReporter {
    fn save_report(&self, test_run: &TestRun) -> Result<String> {
        let filename = format!("report_{}.csv", test_run.id);
        let path = self.output_dir.join(&filename);
        
        let mut csv = String::new();
        csv.push_str("payload_id,success,risk_score\n");
        
        for result in &test_run.results {
            csv.push_str(&format!(
                "{},{},{}\n",
                result.payload_id,
                result.success,
                result.risk_score
            ));
        }
        
        std::fs::write(&path, csv)?;
        Ok(path.to_string_lossy().to_string())
    }
    
    fn format_summary(&self, test_run: &TestRun) -> String {
        format!(
            "Model: {}\nRisk: {}/100\nAttacks: {}/{} successful",
            test_run.model,
            test_run.overall_risk_score,
            test_run.successful_attacks,
            test_run.total_attacks
        )
    }
}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        let result = my_function();
        assert!(result.is_ok());
    }
}
```

### Async Tests

```rust
#[tokio::test]
async fn test_async_function() {
    let result = my_async_function().await;
    assert!(result.is_ok());
}
```

## Best Practices

1. **Error Handling**: Always use `Result<T>` for fallible operations
2. **Async**: Use `async/await` for I/O operations
3. **Traits**: Use traits for extensibility
4. **Testing**: Write tests for all public APIs
5. **Documentation**: Document all public items
6. **Logging**: Use `tracing` for logging

## Resources

- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Documentation](https://tokio.rs/)
- [Serde Documentation](https://serde.rs/)
- [Clap Documentation](https://docs.rs/clap/)
