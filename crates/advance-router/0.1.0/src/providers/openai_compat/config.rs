use std::collections::HashMap;

/// Configuration for an OpenAI-compatible provider.
#[derive(Debug, Clone)]
pub struct OpenAICompatConfig {
    pub base_url: String,
    pub api_key: String,
    pub auth_header_name: String,
    pub auth_header_value: String,
    pub default_model: String,
    pub extra_headers: HashMap<String, String>,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_json_mode: bool,
    /// Field name for reasoning/thinking content in streaming deltas (e.g. "reasoning_content" for DeepSeek).
    pub reasoning_field: Option<String>,
}

impl OpenAICompatConfig {
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        let api_key = api_key.into();
        Self {
            base_url: base_url.into(),
            auth_header_name: "Authorization".to_string(),
            auth_header_value: format!("Bearer {}", api_key),
            api_key,
            default_model: String::new(),
            extra_headers: HashMap::new(),
            supports_tools: true,
            supports_vision: true,
            supports_json_mode: true,
            reasoning_field: None,
        }
    }
}
