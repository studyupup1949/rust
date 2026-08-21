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

const DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com";

/// Google Gemini provider.
pub struct GeminiProvider {
    config: ProviderConfig,
}

impl GeminiProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self { config }
    }
}

impl Provider for GeminiProvider {
    fn id(&self) -> &str {
        "gemini"
    }

    fn language_model(&self, model_id: &str) -> Box<dyn LanguageModel> {
        Box::new(GeminiModel {
            config: self.config.clone(),
            model_id: model_id.to_string(),
            client: http::create_client(self.config.timeout).expect("Failed to create HTTP client"),
        })
    }

    fn models(&self) -> Vec<&str> {
        vec![
            "gemini-2.5-pro",
            "gemini-2.5-flash",
            "gemini-2.0-flash",
        ]
    }
}

struct GeminiModel {
    config: ProviderConfig,
    model_id: String,
    client: reqwest::Client,
}

impl GeminiModel {
    fn base_url(&self) -> &str {
        self.config
            .base_url
            .as_deref()
            .unwrap_or(DEFAULT_BASE_URL)
    }

    fn generate_url(&self) -> String {
        format!(
            "{}/v1beta/models/{}:generateContent",
            self.base_url().trim_end_matches('/'),
            self.model_id,
        )
    }

    fn stream_url(&self) -> String {
        format!(
            "{}/v1beta/models/{}:streamGenerateContent?alt=sse",
            self.base_url().trim_end_matches('/'),
            self.model_id,
        )
    }

    fn build_request(&self, url: &str, body: serde_json::Value) -> reqwest::RequestBuilder {
        let mut req = self
            .client
            .post(url)
            .header("x-goog-api-key", &self.config.api_key)
            .header("content-type", "application/json")
            .json(&body);

        for (key, value) in &self.config.extra_headers {
            req = req.header(key, value);
        }

        req
    }
}

#[async_trait]
impl LanguageModel for GeminiModel {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn provider_id(&self) -> &str {
        "gemini"
    }

    async fn generate(&self, request: ChatRequest) -> Result<ChatResponse, RouterError> {
        let body = convert::to_gemini_request(&request);
        let url = self.generate_url();

        let response = self
            .build_request(&url, body)
            .send()
            .await
            .map_err(RouterError::Http)?;

        let json = http::handle_json_response(response, "gemini").await?;
        convert::from_gemini_response(json)
    }

    async fn stream(&self, request: ChatRequest) -> Result<ChatStream, RouterError> {
        let body = convert::to_gemini_request(&request);
        let url = self.stream_url();

        let response = self
            .build_request(&url, body)
            .send()
            .await
            .map_err(RouterError::Http)?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body: serde_json::Value = response.json().await.unwrap_or_default();
            let message = body["error"]["message"]
                .as_str()
                .unwrap_or("Unknown error")
                .to_string();
            return Err(RouterError::api_error("gemini", status, message, body));
        }

        let raw_stream = http::sse_stream(response);

        let event_stream = raw_stream.flat_map(move |result| {
            let events: Vec<Result<crate::types::response::StreamEvent, RouterError>> =
                match result {
                    Ok(data) => match serde_json::from_str::<serde_json::Value>(&data) {
                        Ok(json) => stream::parse_stream_chunk(&json)
                            .into_iter()
                            .map(Ok)
                            .collect(),
                        Err(e) => vec![Err(RouterError::Serialization(e))],
                    },
                    Err(e) => vec![Err(e)],
                };
            futures::stream::iter(events)
        });

        Ok(ChatStream::new(event_stream))
    }

    fn capabilities(&self) -> ModelCapabilities {
        ModelCapabilities {
            streaming: true,
            tool_calling: true,
            vision: true,
            json_mode: true,
            json_schema: true,
            extended_thinking: false,
            embeddings: false,
        }
    }
}
