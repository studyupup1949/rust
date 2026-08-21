use crate::llm::traits::{LLMProvider, LLMResponse, Message, LLMOptions};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

#[derive(Debug)]
pub struct HermesProvider {
    endpoint: String,
    api_key: Option<String>,
    model: String,
    client: Client,
}

impl HermesProvider {
    pub fn new(endpoint: &str, api_key: Option<String>, model: &str) -> Self {
        HermesProvider {
            endpoint: endpoint.trim_end_matches('/').to_string(),
            api_key,
            model: model.to_string(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LLMProvider for HermesProvider {
    async fn chat(&self, messages: &[Message], options: &LLMOptions) -> Result<LLMResponse, String> {
        let url = format!("{}/v1/chat/completions", self.endpoint);

        let body = json!({
            "model": options.model.as_deref().unwrap_or(&self.model),
            "messages": messages.iter().map(|m| {
                json!({"role": m.role, "content": m.content})
            }).collect::<Vec<_>>(),
            "temperature": options.temperature.unwrap_or(0.3),
            "max_tokens": options.max_tokens.unwrap_or(4096),
        });

        let mut req = self.client.post(&url).json(&body);

        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let resp = req
            .send()
            .await
            .map_err(|e| format!("Hermes request failed: {}", e))?;

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Hermes response parse failed: {}", e))?;

        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let model = data["model"].as_str().unwrap_or(&self.model).to_string();

        let usage = data["usage"].as_object().map(|u| crate::llm::traits::LLMUsage {
            prompt_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: u["total_tokens"].as_u64().unwrap_or(0) as u32,
        });

        Ok(LLMResponse {
            content,
            model,
            usage,
        })
    }
}
