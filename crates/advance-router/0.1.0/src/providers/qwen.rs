use crate::model::LanguageModel;
use crate::provider::{Provider, ProviderConfig};
use crate::providers::openai_compat::config::OpenAICompatConfig;
use crate::providers::openai_compat::OpenAICompatModel;

/// Qwen / Alibaba Cloud provider.
pub struct QwenProvider {
    config: ProviderConfig,
}

impl QwenProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self { config }
    }

    fn compat_config(&self, model_id: &str) -> OpenAICompatConfig {
        let base_url = self
            .config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://dashscope.aliyuncs.com/compatible-mode".to_string());

        let mut compat = OpenAICompatConfig::new(base_url, &self.config.api_key);
        compat.default_model = model_id.to_string();

        for (k, v) in &self.config.extra_headers {
            compat.extra_headers.insert(k.clone(), v.clone());
        }

        compat
    }
}

impl Provider for QwenProvider {
    fn id(&self) -> &str {
        "qwen"
    }

    fn language_model(&self, model_id: &str) -> Box<dyn LanguageModel> {
        Box::new(OpenAICompatModel::new(
            self.compat_config(model_id),
            model_id,
        ))
    }

    fn models(&self) -> Vec<&str> {
        vec!["qwen-turbo", "qwen-plus", "qwen-max", "qwen-long"]
    }
}
