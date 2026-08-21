use crate::llm::traits::{LLMProvider, LLMResponse, Message, LLMOptions};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

#[derive(Debug)]
pub struct ClaudeProvider {
    api_key: String,
    model: String,
    client: Client,
}

impl ClaudeProvider {
    pub fn new(api_key: &str, model: &str) -> Self {
        ClaudeProvider {
            api_key: api_key.to_string(),
            model: model.to_string(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LLMProvider for ClaudeProvider {
    async fn chat(&self, messages: &[Message], options: &LLMOptions) -> Result<LLMResponse, String> {
        let url = "https://api.anthropic.com/v1/messages";

        let system = messages
            .iter()
            .find(|m| m.role == "system")
            .map(|m| m.content.clone());

        let user_messages: Vec<serde_json::Value> = messages
            .iter()
            .filter(|m| m.role != "system")
            .map(|m| json!({"role": m.role, "content": m.content}))
            .collect();

        let mut body = json!({
            "model": options.model.as_deref().unwrap_or(&self.model),
            "messages": user_messages,
            "max_tokens": options.max_tokens.unwrap_or(4096),
        });

        if let Some(ref s) = system {
            body["system"] = json!(s);
        }

        let resp = self
            .client
            .post(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Claude request failed: {}", e))?;

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Claude response parse failed: {}", e))?;

        let content = data["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let model = data["model"].as_str().unwrap_or(&self.model).to_string();

        let usage = data["usage"].as_object().map(|u| crate::llm::traits::LLMUsage {
            prompt_tokens: u["input_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: u["output_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: 0,
        });

        Ok(LLMResponse {
            content,
            model,
            usage,
        })
    }
}
