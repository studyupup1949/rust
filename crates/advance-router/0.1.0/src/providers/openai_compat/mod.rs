pub mod config;
pub mod convert;
pub mod stream;

use async_trait::async_trait;
use futures::StreamExt;

use crate::error::RouterError;
use crate::http;
use crate::model::{LanguageModel, ModelCapabilities};
use crate::stream::ChatStream;
use crate::types::request::ChatRequest;
use crate::types::response::ChatResponse;

use self::config::OpenAICompatConfig;

/// A language model backed by any OpenAI-compatible API.
pub struct OpenAICompatModel {
    config: OpenAICompatConfig,
    model_id: String,
    client: reqwest::Client,
}

impl OpenAICompatModel {
    pub fn new(config: OpenAICompatConfig, model_id: impl Into<String>) -> Self {
        let client = http::create_client(std::time::Duration::from_secs(300))
            .expect("Failed to create HTTP client");
        Self {
            config,
            model_id: model_id.into(),
            client,
        }
    }

    fn build_request(&self, body: serde_json::Value) -> reqwest::RequestBuilder {
        let url = format!("{}/v1/chat/completions", self.config.base_url.trim_end_matches('/'));

        let mut req = self
            .client
            .post(&url)
            .header(&self.config.auth_header_name, &self.config.auth_header_value)
            .json(&body);

        for (key, value) in &self.config.extra_headers {
            req = req.header(key, value);
        }

        req
    }
}

#[async_trait]
impl LanguageModel for OpenAICompatModel {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn provider_id(&self) -> &str {
        "openai_compat"
    }

    async fn generate(&self, request: ChatRequest) -> Result<ChatResponse, RouterError> {
        let body = convert::to_openai_request(&request, &self.config);
        let response = self
            .build_request(body)
            .send()
            .await
            .map_err(RouterError::Http)?;

        let json = http::handle_json_response(response, self.provider_id()).await?;
        convert::from_openai_response(json, &self.config)
    }

    async fn stream(&self, mut request: ChatRequest) -> Result<ChatStream, RouterError> {
        request.stream = true;
        let body = convert::to_openai_request(&request, &self.config);

        let response = self
            .build_request(body)
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
            return Err(RouterError::api_error(self.provider_id(), status, message, body));
        }

        let config = self.config.clone();
        let raw_stream = http::sse_stream(response);

        let event_stream = raw_stream.flat_map(move |result| {
            let config = config.clone();
            let events: Vec<Result<crate::types::response::StreamEvent, RouterError>> =
                match result {
                    Ok(data) => match serde_json::from_str::<serde_json::Value>(&data) {
                        Ok(json) => stream::parse_stream_chunk(&json, &config)
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
            tool_calling: self.config.supports_tools,
            vision: self.config.supports_vision,
            json_mode: self.config.supports_json_mode,
            json_schema: self.config.supports_json_mode,
            extended_thinking: self.config.reasoning_field.is_some(),
            embeddings: false,
        }
    }
}
