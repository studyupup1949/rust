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

pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    api_base: String,
    model: String,
    timeout: Duration,
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
    usage: OpenAIUsage,
    model: String,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

impl OpenAIProvider {
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let api_key = config
            .api_key
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .ok_or_else(|| {
                AdversariaError::Provider(
                    "OpenAI API key not found in config or OPENAI_API_KEY env var".to_string(),
                )
            })?;

        let api_base = config
            .api_base
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());

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
impl Provider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn generate(&self, prompt: &str) -> Result<ModelResponse> {
        let url = format!("{}/chat/completions", self.api_base);

        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: 0.7,
            max_tokens: 1000,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AdversariaError::Provider(format!(
                "OpenAI API error ({}): {}",
                status, error_text
            )));
        }

        let openai_response: OpenAIResponse = response.json().await?;

        let content = openai_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| AdversariaError::Provider("No response from OpenAI".to_string()))?;

        Ok(ModelResponse {
            content,
            model: openai_response.model,
            usage: Some(Usage {
                prompt_tokens: openai_response.usage.prompt_tokens,
                completion_tokens: openai_response.usage.completion_tokens,
                total_tokens: openai_response.usage.total_tokens,
            }),
        })
    }

    async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/models", self.api_base);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}
