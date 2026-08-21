use crate::model::LanguageModel;
use crate::provider::{Provider, ProviderConfig};
use crate::providers::openai_compat::config::OpenAICompatConfig;
use crate::providers::openai_compat::OpenAICompatModel;

/// Grok (xAI) provider.
pub struct GrokProvider {
    config: ProviderConfig,
}

impl GrokProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self { config }
    }

    fn compat_config(&self, model_id: &str) -> OpenAICompatConfig {
        let base_url = self
            .config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.x.ai".to_string());

        let mut compat = OpenAICompatConfig::new(base_url, &self.config.api_key);
        compat.default_model = model_id.to_string();

        for (k, v) in &self.config.extra_headers {
            compat.extra_headers.insert(k.clone(), v.clone());
        }

        compat
    }
}

impl Provider for GrokProvider {
    fn id(&self) -> &str {
        "grok"
    }

    fn language_model(&self, model_id: &str) -> Box<dyn LanguageModel> {
        Box::new(OpenAICompatModel::new(
            self.compat_config(model_id),
            model_id,
        ))
    }

    fn models(&self) -> Vec<&str> {
        vec!["grok-3", "grok-3-mini", "grok-2"]
    }
}
