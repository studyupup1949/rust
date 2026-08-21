use crate::core::error::{AdversariaError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub default_provider: String,
    pub providers: HashMap<String, ProviderConfig>,
    pub suites: SuitesConfig,
    pub reporting: ReportingConfig,
    pub plugins: PluginsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub api_base: Option<String>,
    pub model: String,
    pub timeout_seconds: Option<u64>,
    pub max_retries: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuitesConfig {
    pub directory: PathBuf,
    pub enabled_suites: Vec<String>,
    #[serde(default)]
    pub custom_suites: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingConfig {
    pub output_directory: PathBuf,
    pub format: ReportFormat,
    #[serde(default = "default_keep_reports")]
    pub keep_reports: usize,
}

fn default_keep_reports() -> usize {
    100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportFormat {
    Json,
    Yaml,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginsConfig {
    pub directory: PathBuf,
    #[serde(default)]
    pub enabled: bool,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| AdversariaError::Config(format!("Failed to read config file: {}", e)))?;

        serde_yaml::from_str(&content)
            .map_err(|e| AdversariaError::Config(format!("Failed to parse config file: {}", e)))
    }

    pub fn default_config_path() -> PathBuf {
        PathBuf::from("adversaria.config.yaml")
    }

    pub fn create_default<P: AsRef<Path>>(path: P) -> Result<()> {
        let default_config = Self::default();
        let yaml = serde_yaml::to_string(&default_config)?;
        std::fs::write(path, yaml)?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut providers = HashMap::new();

        providers.insert(
            "openai".to_string(),
            ProviderConfig {
                api_key: None,
                api_base: Some("https://api.openai.com/v1".to_string()),
                model: "gpt-4".to_string(),
                timeout_seconds: Some(30),
                max_retries: Some(3),
            },
        );

        providers.insert(
            "anthropic".to_string(),
            ProviderConfig {
                api_key: None,
                api_base: Some("https://api.anthropic.com/v1".to_string()),
                model: "claude-3-opus-20240229".to_string(),
                timeout_seconds: Some(30),
                max_retries: Some(3),
            },
        );

        providers.insert(
            "ollama".to_string(),
            ProviderConfig {
                api_key: None,
                api_base: Some("http://localhost:11434".to_string()),
                model: "llama2".to_string(),
                timeout_seconds: Some(60),
                max_retries: Some(2),
            },
        );

        Self {
            version: "1.0".to_string(),
            default_provider: "openai".to_string(),
            providers,
            suites: SuitesConfig {
                directory: PathBuf::from("./suites"),
                enabled_suites: vec![
                    "prompt_injection".to_string(),
                    "jailbreak".to_string(),
                    "role_confusion".to_string(),
                    "data_exfiltration".to_string(),
                ],
                custom_suites: vec![],
            },
            reporting: ReportingConfig {
                output_directory: PathBuf::from("./reports"),
                format: ReportFormat::Json,
                keep_reports: 100,
            },
            plugins: PluginsConfig {
                directory: PathBuf::from("./plugins"),
                enabled: true,
            },
        }
    }
}
