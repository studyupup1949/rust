use std::collections::HashMap;
use std::time::Duration;

use crate::model::{EmbeddingModel, LanguageModel};

/// Configuration for connecting to a provider.
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub api_key: String,
    pub base_url: Option<String>,
    pub org_id: Option<String>,
    pub extra_headers: HashMap<String, String>,
    pub timeout: Duration,
}

impl ProviderConfig {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: None,
            org_id: None,
            extra_headers: HashMap::new(),
            timeout: Duration::from_secs(300),
        }
    }

    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    pub fn with_org_id(mut self, org_id: impl Into<String>) -> Self {
        self.org_id = Some(org_id.into());
        self
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra_headers.insert(key.into(), value.into());
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// Trait for an LLM provider that can create model instances.
pub trait Provider: Send + Sync {
    /// Provider identifier (e.g. "openai", "anthropic").
    fn id(&self) -> &str;

    /// Create a language model instance for the given model ID.
    fn language_model(&self, model_id: &str) -> Box<dyn LanguageModel>;

    /// Create an embedding model instance (not all providers support this).
    fn embedding_model(&self, _model_id: &str) -> Option<Box<dyn EmbeddingModel>> {
        None
    }

    /// List available model IDs for this provider.
    fn models(&self) -> Vec<&str>;
}
