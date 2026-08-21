use crate::llm::traits::{LLMProvider, LLMResponse};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

#[derive(Debug)]
pub struct OpenClawProvider {
    endpoint: String,
    api_key: Option<String>,
    client: Client,
}

impl OpenClawProvider {
    pub fn new(endpoint: &str, api_key: Option<String>) -> Self {
        OpenClawProvider {
            endpoint: endpoint.to_string(),
            api_key,
            client: Client::new(),
        }
    }

    pub async fn test_connection(&self) -> Result<String, String> {
        let url = format!("{}/health", self.endpoint);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("OpenClaw health check failed: {}", e))?;

        if resp.status().is_success() {
            Ok("OpenClaw connected".to_string())
        } else {
            Err(format!("OpenClaw health check returned {}", resp.status()))
        }
    }
}

#[async_trait]
impl LLMProvider for OpenClawProvider {
    async fn chat(
        &self,
        messages: &[crate::llm::traits::Message],
        _options: &crate::llm::traits::LLMOptions,
    ) -> Result<LLMResponse, String> {
        let task = messages
            .iter()
            .map(|m| format!("[{}]: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        let url = format!("{}/api/chat", self.endpoint);
        let mut body = json!({ "message": task, "stream": false });

        if let Some(ref key) = self.api_key {
            body["api_key"] = json!(key);
        }

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("OpenClaw request failed: {}", e))?;

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("OpenClaw response parse failed: {}", e))?;

        Ok(LLMResponse {
            content: data["response"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            model: "openclaw".to_string(),
            usage: None,
        })
    }
}
