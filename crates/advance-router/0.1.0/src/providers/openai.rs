use crate::model::LanguageModel;
use crate::provider::{Provider, ProviderConfig};
use crate::providers::openai_compat::config::OpenAICompatConfig;
use crate::providers::openai_compat::OpenAICompatModel;

/// OpenAI provider (GPT-4o, GPT-4, o1, o3, etc.)
pub struct OpenAIProvider {
    config: ProviderConfig,
}

impl OpenAIProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self { config }
    }

    fn compat_config(&self, model_id: &str) -> OpenAICompatConfig {
        let base_url = self
            .config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com".to_string());

        let mut compat = OpenAICompatConfig::new(base_url, &self.config.api_key);
        compat.default_model = model_id.to_string();
        compat.supports_tools = true;
        compat.supports_vision = true;
        compat.supports_json_mode = true;

        if let Some(org_id) = &self.config.org_id {
            compat
                .extra_headers
                .insert("OpenAI-Organization".to_string(), org_id.clone());
        }

        for (k, v) in &self.config.extra_headers {
            compat.extra_headers.insert(k.clone(), v.clone());
        }

        compat
    }
}

impl Provider for OpenAIProvider {
    fn id(&self) -> &str {
        "openai"
    }

    fn language_model(&self, model_id: &str) -> Box<dyn LanguageModel> {
        Box::new(OpenAICompatModel::new(
            self.compat_config(model_id),
            model_id,
        ))
    }

    fn models(&self) -> Vec<&str> {
        vec![
            "gpt-4o",
            "gpt-4o-mini",
            "gpt-4-turbo",
            "gpt-4",
            "o1",
            "o1-mini",
            "o3",
            "o3-mini",
            "o4-mini",
        ]
    }
}
