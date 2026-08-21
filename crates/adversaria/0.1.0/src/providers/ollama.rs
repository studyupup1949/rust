use crate::core::{
    config::ProviderConfig,
    error::{AdversariaError, Result},
    ModelResponse,
};
use crate::providers::Provider;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct OllamaProvider {
    client: Client,
    api_base: String,
    model: String,
    timeout: Duration,
}

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let api_base = config
            .api_base
            .unwrap_or_else(|| "http://localhost:11434".to_string());

        let timeout = Duration::from_secs(config.timeout_seconds.unwrap_or(60));

        let client = Client::builder().timeout(timeout).build().map_err(|e| {
            AdversariaError::Provider(format!("Failed to create HTTP client: {}", e))
        })?;

        Ok(Self {
            client,
            api_base,
            model: config.model,
            timeout,
        })
    }
}

#[async_trait]
impl Provider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn generate(&self, prompt: &str) -> Result<ModelResponse> {
        let url = format!("{}/api/generate", self.api_base);

        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AdversariaError::Provider(format!(
                "Ollama API error ({}): {}",
                status, error_text
            )));
        }

        let ollama_response: OllamaResponse = response.json().await?;

        Ok(ModelResponse {
            content: ollama_response.response,
            model: ollama_response.model,
            usage: None,
        })
    }

    async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/api/tags", self.api_base);

        let response = self.client.get(&url).send().await?;

        Ok(response.status().is_success())
    }
}
