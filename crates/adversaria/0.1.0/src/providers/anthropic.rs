use crate::core::{
    config::ProviderConfig,
    error::{AdversariaError, Result},
    ModelResponse, Usage,
};
use crate::providers::Provider;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    api_base: String,
    model: String,
    timeout: Duration,
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
    model: String,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    text: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: usize,
    output_tokens: usize,
}

impl AnthropicProvider {
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let api_key = config
            .api_key
            .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
            .ok_or_else(|| {
                AdversariaError::Provider(
                    "Anthropic API key not found in config or ANTHROPIC_API_KEY env var"
                        .to_string(),
                )
            })?;

        let api_base = config
            .api_base
            .unwrap_or_else(|| "https://api.anthropic.com/v1".to_string());

        let timeout = Duration::from_secs(config.timeout_seconds.unwrap_or(30));

        let client = Client::builder().timeout(timeout).build().map_err(|e| {
            AdversariaError::Provider(format!("Failed to create HTTP client: {}", e))
        })?;

        Ok(Self {
            client,
            api_key,
            api_base,
            model: config.model,
            timeout,
        })
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn generate(&self, prompt: &str) -> Result<ModelResponse> {
        let url = format!("{}/messages", self.api_base);

        let request = AnthropicRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: 1000,
            temperature: 0.7,
        };

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AdversariaError::Provider(format!(
                "Anthropic API error ({}): {}",
                status, error_text
            )));
        }

        let anthropic_response: AnthropicResponse = response.json().await?;

        let content = anthropic_response
            .content
            .first()
            .map(|c| c.text.clone())
            .ok_or_else(|| AdversariaError::Provider("No response from Anthropic".to_string()))?;

        Ok(ModelResponse {
            content,
            model: anthropic_response.model,
            usage: Some(Usage {
                prompt_tokens: anthropic_response.usage.input_tokens,
                completion_tokens: anthropic_response.usage.output_tokens,
                total_tokens: anthropic_response.usage.input_tokens
                    + anthropic_response.usage.output_tokens,
            }),
        })
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}
