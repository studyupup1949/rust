use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMOptions {
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub model: Option<String>,
}

impl Default for LLMOptions {
    fn default() -> Self {
        LLMOptions {
            temperature: Some(0.3),
            max_tokens: Some(4096),
            model: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<LLMUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[async_trait]
pub trait LLMProvider: Send + Sync + Debug {
    async fn chat(&self, messages: &[Message], options: &LLMOptions) -> Result<LLMResponse, String>;
    async fn analyze(&self, prompt: &str, context: &str) -> Result<String, String> {
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: "You are an expert systems analyst. Analyze the following context and provide a detailed analysis.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: format!("Context:\n{}\n\nPrompt:\n{}", context, prompt),
            },
        ];
        let resp = self.chat(&messages, &LLMOptions::default()).await?;
        Ok(resp.content)
    }

    async fn decide(&self, options: &str, context: &str) -> Result<String, String> {
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: "You are a decision-making AI. Choose the best option from the available choices and explain your reasoning.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: format!("Context:\n{}\n\nAvailable options:\n{}\n\nWhich option do you choose and why?", context, options),
            },
        ];
        let resp = self.chat(&messages, &LLMOptions::default()).await?;
        Ok(resp.content)
    }
}
