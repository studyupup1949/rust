use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// LLM 提供商 (openai/azure/anthropic)
    pub provider: String,
    /// API Key
    pub api_key: String,
    /// API Base URL
    pub api_base: Option<String>,
    /// 模型名称
    pub model: String,
    /// 最大 Token 数
    pub max_tokens: u32,
    /// 温度系数
    pub temperature: f32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            api_key: "".to_string(),
            api_base: None,
            model: "gpt-3.5-turbo".to_string(),
            max_tokens: 2048,
            temperature: 0.7,
        }
    }
}
