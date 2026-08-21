use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub metadata: Metadata,
    pub llm: LLMConfig,
    pub agents: AgentConfigs,
    pub execution: ExecutionConfig,
    pub learning: LearningConfig,
    pub notifications: NotificationConfig,
    pub advanced: AdvancedConfig,
    #[serde(default)]
    pub external_agents: Option<ExternalAgentsConfig>,
    #[serde(default)]
    pub rsi: Option<RSIConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub created_at: Option<String>,
    pub user: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub provider: String,
    pub endpoint: String,
    pub api_key: Option<String>,
    pub model_name: Option<String>,
    pub timeout_seconds: u64,
    pub fallback_provider: Option<String>,
    pub fallback_endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfigs {
    pub repository: Option<RepositoryAgentConfig>,
    pub logs: Option<LogsAgentConfig>,
    pub metrics: Option<MetricsAgentConfig>,
    pub health: Option<HealthAgentConfig>,
    pub task: Option<TaskAgentConfig>,
    pub trace: Option<TraceAgentConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryAgentConfig {
    pub enabled: bool,
    pub detection_interval: String,
    pub platforms: Vec<String>,
    pub github: Option<GitHubConfig>,
    pub fixes: FixesConfig,
    pub max_complexity: String,
    pub auto_commit: bool,
    pub require_pr_approval: bool,
    pub max_actions_per_run: u32,
    pub local_repos: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub organization: Option<String>,
    pub token: Option<String>,
    pub repositories: Vec<String>,
    pub private_repos: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixesConfig {
    pub failing_tests: bool,
    pub performance_regressions: bool,
    pub security_vulnerabilities: bool,
    pub code_quality: bool,
    pub dependency_updates: bool,
    pub refactor_architecture: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogsAgentConfig {
    pub enabled: bool,
    pub detection_interval: String,
    pub sources: Vec<LogSource>,
    pub error_threshold: ErrorThreshold,
    pub auto_fix: bool,
    pub escalate_on_unknown: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSource {
    pub r#type: String,
    pub path: String,
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorThreshold {
    pub count: u32,
    pub time_window: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsAgentConfig {
    pub enabled: bool,
    pub detection_interval: String,
    pub providers: Vec<String>,
    pub prometheus: Option<PrometheusConfig>,
    pub thresholds: MetricThresholds,
    pub auto_scale: bool,
    pub optimization_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusConfig {
    pub endpoint: String,
    pub retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricThresholds {
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub disk_percent: f64,
    pub latency_ms: u64,
    pub error_rate_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAgentConfig {
    pub enabled: bool,
    pub detection_interval: String,
    pub endpoints: Vec<String>,
    pub auto_restart: bool,
    pub max_restart_attempts: u32,
    pub restart_backoff_seconds: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAgentConfig {
    pub enabled: bool,
    pub detection_interval: String,
    pub auto_execute: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceAgentConfig {
    pub enabled: bool,
    pub detection_interval: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub mode: String,
    pub local_only: bool,
    pub max_concurrent_actions: u32,
    pub approval_required_for: Vec<String>,
    pub rollback_enabled: bool,
    pub rollback_timeout_minutes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConfig {
    pub enabled: bool,
    pub storage: String,
    pub db_path: String,
    pub prediction_enabled: bool,
    pub prediction_confidence_threshold: f64,
    pub memory_retention_days: u64,
    pub auto_learn: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub channels: Vec<String>,
    pub slack: Option<SlackConfig>,
    pub email: Option<EmailConfig>,
    pub triggers: NotificationTriggers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    pub workspace_url: Option<String>,
    pub bot_token: Option<String>,
    pub channel: Option<String>,
    pub thread_replies: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub address: Option<String>,
    pub smtp: Option<SmtpConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub use_tls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTriggers {
    pub fixed_issues: bool,
    pub escalations: bool,
    pub predictions: bool,
    pub errors: bool,
    pub all_decisions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedConfig {
    pub experimental_features: bool,
    pub debug_level: String,
    pub max_agent_memory_mb: u64,
    pub log_retention_days: u64,
    pub telemetry_enabled: bool,
    pub db_path: Option<String>,
    pub config_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalAgentsConfig {
    #[serde(default)]
    pub claude_code: Option<ClaudeCodeConfig>,
    #[serde(default)]
    pub openclaw: Option<OpenClawConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeCodeConfig {
    pub enabled: bool,
    pub binary_path: Option<String>,
    pub working_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenClawConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSIConfig {
    pub enabled: bool,
    pub min_confidence_threshold: f64,
    pub max_confidence_threshold: f64,
    pub min_interval_secs: u64,
    pub max_interval_secs: u64,
    pub evaluation_window: usize,
    pub hyperfocus_enabled: bool,
    pub cross_agent_reactions: bool,
}

impl Default for Config {
    fn default() -> Self {
        let config_dir = dirs::home_dir()
            .map(|p| p.join(".aas"))
            .unwrap_or_else(|| PathBuf::from(".aas"));

        Config {
            version: "1.0".to_string(),
            metadata: Metadata {
                created_at: None,
                user: None,
                description: None,
            },
            llm: LLMConfig {
                provider: "hermes".to_string(),
                endpoint: "http://localhost:5000".to_string(),
                api_key: None,
                model_name: None,
                timeout_seconds: 30,
                fallback_provider: None,
                fallback_endpoint: None,
            },
            agents: AgentConfigs {
                repository: Some(RepositoryAgentConfig::default()),
                logs: Some(LogsAgentConfig::default()),
                metrics: Some(MetricsAgentConfig::default()),
                health: Some(HealthAgentConfig::default()),
                task: Some(TaskAgentConfig::default()),
                trace: Some(TraceAgentConfig::default()),
            },
            execution: ExecutionConfig {
                mode: "staged_rollout".to_string(),
                local_only: true,
                max_concurrent_actions: 3,
                approval_required_for: vec![
                    "architecture_changes".to_string(),
                    "data_deletion".to_string(),
                ],
                rollback_enabled: true,
                rollback_timeout_minutes: 30,
            },
            learning: LearningConfig {
                enabled: true,
                storage: "sqlite".to_string(),
                db_path: config_dir.join("aas.db").to_string_lossy().to_string(),
                prediction_enabled: true,
                prediction_confidence_threshold: 0.85,
                memory_retention_days: 365,
                auto_learn: true,
            },
            notifications: NotificationConfig {
                channels: vec![],
                slack: None,
                email: None,
                triggers: NotificationTriggers {
                    fixed_issues: true,
                    escalations: true,
                    predictions: true,
                    errors: true,
                    all_decisions: false,
                },
            },
            advanced: AdvancedConfig {
                experimental_features: false,
                debug_level: "info".to_string(),
                max_agent_memory_mb: 500,
                log_retention_days: 30,
                telemetry_enabled: false,
                db_path: None,
                config_dir: None,
            },
            external_agents: Some(ExternalAgentsConfig {
                claude_code: Some(ClaudeCodeConfig {
                    enabled: true,
                    binary_path: None,
                    working_dir: None,
                }),
                openclaw: Some(OpenClawConfig {
                    enabled: false,
                    endpoint: "http://localhost:3001".to_string(),
                    api_key: None,
                }),
            }),
            rsi: Some(RSIConfig {
                enabled: true,
                min_confidence_threshold: 0.3,
                max_confidence_threshold: 0.95,
                min_interval_secs: 5,
                max_interval_secs: 3600,
                evaluation_window: 20,
                hyperfocus_enabled: true,
                cross_agent_reactions: true,
            }),
        }
    }
}

impl RepositoryAgentConfig {
    fn default() -> Self {
        RepositoryAgentConfig {
            enabled: true,
            detection_interval: "5m".to_string(),
            platforms: vec!["github".to_string()],
            github: None,
            fixes: FixesConfig {
                failing_tests: true,
                performance_regressions: true,
                security_vulnerabilities: true,
                code_quality: true,
                dependency_updates: true,
                refactor_architecture: true,
            },
            max_complexity: "deep".to_string(),
            auto_commit: true,
            require_pr_approval: false,
            max_actions_per_run: 3,
            local_repos: vec!["/tmp/aas-test-repo".to_string()],
        }
    }
}

impl LogsAgentConfig {
    fn default() -> Self {
        LogsAgentConfig {
            enabled: true,
            detection_interval: "continuous".to_string(),
            sources: vec![LogSource {
                r#type: "file".to_string(),
                path: "/tmp/aas-test-logs/error.log".to_string(),
                format: None,
            }],
            error_threshold: ErrorThreshold {
                count: 5,
                time_window: "1m".to_string(),
            },
            auto_fix: true,
            escalate_on_unknown: true,
        }
    }
}

impl MetricsAgentConfig {
    fn default() -> Self {
        MetricsAgentConfig {
            enabled: true,
            detection_interval: "1m".to_string(),
            providers: vec!["prometheus".to_string()],
            prometheus: None,
            thresholds: MetricThresholds {
                cpu_percent: 80.0,
                memory_percent: 85.0,
                disk_percent: 90.0,
                latency_ms: 500,
                error_rate_percent: 5.0,
            },
            auto_scale: true,
            optimization_enabled: true,
        }
    }
}

impl HealthAgentConfig {
    fn default() -> Self {
        HealthAgentConfig {
            enabled: true,
            detection_interval: "30s".to_string(),
            endpoints: vec![],
            auto_restart: true,
            max_restart_attempts: 3,
            restart_backoff_seconds: vec![5, 10, 30],
        }
    }
}

impl TaskAgentConfig {
    fn default() -> Self {
        TaskAgentConfig {
            enabled: true,
            detection_interval: "10m".to_string(),
            auto_execute: true,
        }
    }
}

impl TraceAgentConfig {
    fn default() -> Self {
        TraceAgentConfig {
            enabled: false,
            detection_interval: "5m".to_string(),
        }
    }
}

impl Config {
    pub fn config_dir() -> PathBuf {
        dirs::home_dir()
            .map(|p| p.join(".aas"))
            .unwrap_or_else(|| PathBuf::from(".aas"))
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    pub fn default_db_path() -> PathBuf {
        Self::config_dir().join("aas.db")
    }

    pub fn load() -> Result<Self, String> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Config::default());
        }
        let contents = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read config at {}: {}", path.display(), e))?;
        serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse config: {}", e))
    }

    pub fn save(&self) -> Result<(), String> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(Self::config_path(), &contents)
            .map_err(|e| format!("Failed to write config: {}", e))?;
        tracing::info!("Configuration saved to {}", Self::config_path().display());
        Ok(())
    }

    pub fn get_enabled_agents(&self) -> Vec<&str> {
        let mut agents = Vec::new();
        if let Some(ref c) = self.agents.repository {
            if c.enabled { agents.push("repository"); }
        }
        if let Some(ref c) = self.agents.logs {
            if c.enabled { agents.push("logs"); }
        }
        if let Some(ref c) = self.agents.metrics {
            if c.enabled { agents.push("metrics"); }
        }
        if let Some(ref c) = self.agents.health {
            if c.enabled { agents.push("health"); }
        }
        if let Some(ref c) = self.agents.task {
            if c.enabled { agents.push("task"); }
        }
        if let Some(ref c) = self.agents.trace {
            if c.enabled { agents.push("trace"); }
        }
        agents
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.llm.endpoint.is_empty() {
            errors.push("LLM endpoint must not be empty".to_string());
        }
        if self.llm.timeout_seconds == 0 {
            errors.push("LLM timeout must be > 0".to_string());
        }

        if let Some(ref repo) = self.agents.repository {
            if repo.enabled {
                if repo.detection_interval.is_empty() {
                    errors.push("Repository agent detection interval must not be empty".to_string());
                }
            }
        }
        if let Some(ref logs) = self.agents.logs {
            if logs.enabled {
                if logs.error_threshold.count == 0 {
                    errors.push("Logs agent error threshold count must be > 0".to_string());
                }
            }
        }
        if let Some(ref metrics) = self.agents.metrics {
            if metrics.enabled {
                if metrics.thresholds.cpu_percent <= 0.0 || metrics.thresholds.cpu_percent > 100.0 {
                    errors.push("CPU threshold must be between 1 and 100".to_string());
                }
                if metrics.thresholds.memory_percent <= 0.0 || metrics.thresholds.memory_percent > 100.0 {
                    errors.push("Memory threshold must be between 1 and 100".to_string());
                }
            }
        }

        if self.execution.mode != "staged_rollout" && self.execution.mode != "manual_approval" && self.execution.mode != "auto" {
            errors.push(format!("Unknown execution mode: {}. Must be staged_rollout, manual_approval, or auto", self.execution.mode));
        }
        if self.execution.max_concurrent_actions == 0 {
            errors.push("max_concurrent_actions must be > 0".to_string());
        }

        if self.learning.prediction_confidence_threshold <= 0.0 || self.learning.prediction_confidence_threshold > 1.0 {
            errors.push("Prediction confidence threshold must be between 0.0 and 1.0".to_string());
        }

        errors
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}
