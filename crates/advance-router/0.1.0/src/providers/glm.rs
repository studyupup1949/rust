use crate::model::LanguageModel;
use crate::provider::{Provider, ProviderConfig};
use crate::providers::openai_compat::config::OpenAICompatConfig;
use crate::providers::openai_compat::OpenAICompatModel;

/// GLM / Zhipu AI provider.
pub struct GLMProvider {
    config: ProviderConfig,
}

impl GLMProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self { config }
    }

    fn compat_config(&self, model_id: &str) -> OpenAICompatConfig {
        let base_url = self
            .config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://open.bigmodel.cn/api/paas".to_string());

        let mut compat = OpenAICompatConfig::new(base_url, &self.config.api_key);
        compat.default_model = model_id.to_string();

        for (k, v) in &self.config.extra_headers {
            compat.extra_headers.insert(k.clone(), v.clone());
        }

        compat
    }
}

impl Provider for GLMProvider {
    fn id(&self) -> &str {
        "glm"
    }

    fn language_model(&self, model_id: &str) -> Box<dyn LanguageModel> {
        Box::new(OpenAICompatModel::new(
            self.compat_config(model_id),
            model_id,
        ))
    }

    fn models(&self) -> Vec<&str> {
        vec!["glm-4", "glm-4-plus", "glm-4-flash"]
    }
}
