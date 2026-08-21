pub mod convert;
pub mod stream;

use async_trait::async_trait;
use futures::StreamExt;

use crate::error::RouterError;
use crate::http;
use crate::model::{LanguageModel, ModelCapabilities};
use crate::provider::{Provider, ProviderConfig};
use crate::stream::ChatStream;
use crate::types::request::ChatRequest;
use crate::types::response::ChatResponse;

use self::stream::AnthropicStreamParser;

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
const API_VERSION: &str = "2023-06-01";

/// Anthropic provider (Claude models).
pub struct AnthropicProvider {
    config: ProviderConfig,
}

impl AnthropicProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self { config }
    }
}

impl Provider for AnthropicProvider {
    fn id(&self) -> &str {
        "anthropic"
    }

    fn language_model(&self, model_id: &str) -> Box<dyn LanguageModel> {
        Box::new(AnthropicModel {
            config: self.config.clone(),
            model_id: model_id.to_string(),
            client: http::create_client(self.config.timeout).expect("Failed to create HTTP client"),
        })
    }

    fn models(&self) -> Vec<&str> {
        vec![
            "claude-sonnet-4-20250514",
            "claude-opus-4-20250514",
            "claude-haiku-4-20250414",
        ]
    }
}

struct AnthropicModel {
    config: ProviderConfig,
    model_id: String,
    client: reqwest::Client,
}

impl AnthropicModel {
    fn base_url(&self) -> &str {
        self.config
            .base_url
            .as_deref()
            .unwrap_or(DEFAULT_BASE_URL)
    }

    fn build_request(&self, body: serde_json::Value) -> reqwest::RequestBuilder {
        let url = format!("{}/v1/messages", self.base_url().trim_end_matches('/'));

        let mut req = self
            .client
            .post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .json(&body);

        for (key, value) in &self.config.extra_headers {
            req = req.header(key, value);
        }

        req
    }
}

#[async_trait]
impl LanguageModel for AnthropicModel {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn provider_id(&self) -> &str {
        "anthropic"
    }

    async fn generate(&self, request: ChatRequest) -> Result<ChatResponse, RouterError> {
        let body = convert::to_anthropic_request(&request);
        let response = self
            .build_request(body)
            .send()
            .await
            .map_err(RouterError::Http)?;

        let json = http::handle_json_response(response, "anthropic").await?;
        convert::from_anthropic_response(json)
    }

    async fn stream(&self, mut request: ChatRequest) -> Result<ChatStream, RouterError> {
        request.stream = true;
        let body = convert::to_anthropic_request(&request);

        let response = self
            .build_request(body)
            .send()
            .await
            .map_err(RouterError::Http)?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            if status == 401 || status == 403 {
                let body = response.text().await.unwrap_or_default();
                return Err(RouterError::Auth {
                    provider: "anthropic".to_string(),
                    message: body,
                });
            }
            let body: serde_json::Value = response.json().await.unwrap_or_default();
            let message = body["error"]["message"]
                .as_str()
                .unwrap_or("Unknown error")
                .to_string();
            return Err(RouterError::api_error("anthropic", status, message, body));
        }

        let raw_stream = http::sse_stream(response);

        let event_stream = async_stream::stream! {
            let mut parser = AnthropicStreamParser::new();
            let mut raw_stream = std::pin::pin!(raw_stream);

            while let Some(result) = raw_stream.next().await {
                match result {
                    Ok(data) => {
                        match serde_json::from_str::<serde_json::Value>(&data) {
                            Ok(json) => {
                                for event in parser.parse_event(&json) {
                                    yield Ok(event);
                                }
                            }
                            Err(e) => {
                                yield Err(RouterError::Serialization(e));
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(e);
                    }
                }
            }
        };

        Ok(ChatStream::new(event_stream))
    }

    fn capabilities(&self) -> ModelCapabilities {
        ModelCapabilities {
            streaming: true,
            tool_calling: true,
            vision: true,
            json_mode: false,
            json_schema: false,
            extended_thinking: true,
            embeddings: false,
        }
    }
}
