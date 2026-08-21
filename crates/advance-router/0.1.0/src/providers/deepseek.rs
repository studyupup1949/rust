use crate::model::LanguageModel;
use crate::provider::{Provider, ProviderConfig};
use crate::providers::openai_compat::config::OpenAICompatConfig;
use crate::providers::openai_compat::OpenAICompatModel;

/// DeepSeek provider (DeepSeek-Chat, DeepSeek-Reasoner).
pub struct DeepSeekProvider {
    config: ProviderConfig,
}

impl DeepSeekProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self { config }
    }

    fn compat_config(&self, model_id: &str) -> OpenAICompatConfig {
        let base_url = self
            .config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.deepseek.com".to_string());

        let mut compat = OpenAICompatConfig::new(base_url, &self.config.api_key);
        compat.default_model = model_id.to_string();
        compat.supports_tools = true;
        compat.supports_vision = false;
        compat.supports_json_mode = true;

        // DeepSeek reasoner uses "reasoning_content" field for thinking
        if model_id.contains("reasoner") {
            compat.reasoning_field = Some("reasoning_content".to_string());
        }

        for (k, v) in &self.config.extra_headers {
            compat.extra_headers.insert(k.clone(), v.clone());
        }

        compat
    }
}

impl Provider for DeepSeekProvider {
    fn id(&self) -> &str {
        "deepseek"
    }

    fn language_model(&self, model_id: &str) -> Box<dyn LanguageModel> {
        Box::new(OpenAICompatModel::new(
            self.compat_config(model_id),
            model_id,
        ))
    }

    fn models(&self) -> Vec<&str> {
        vec!["deepseek-chat", "deepseek-reasoner"]
    }
}
