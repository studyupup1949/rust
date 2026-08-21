use std::collections::HashMap;
use std::future::Future;

use crate::error::RouterError;
use crate::model::LanguageModel;
use crate::provider::Provider;
use crate::stream::ChatStream;
use crate::types::message::Message;
use crate::types::request::ChatRequest;
use crate::types::response::ChatResponse;
use crate::types::tool::{ToolCall, ToolResult};

/// The main LLM Gateway that routes requests to providers.
pub struct Gateway {
    providers: HashMap<String, Box<dyn Provider>>,
    model_routes: HashMap<String, String>,
    default_provider: Option<String>,
}

impl Gateway {
    pub fn builder() -> GatewayBuilder {
        GatewayBuilder::default()
    }

    /// Resolve which provider handles a given model name.
    fn resolve_provider(&self, model: &str) -> Result<&dyn Provider, RouterError> {
        // First check explicit model routes
        if let Some(provider_id) = self.model_routes.get(model) {
            if let Some(provider) = self.providers.get(provider_id) {
                return Ok(provider.as_ref());
            }
        }

        // Then try to match by provider prefix patterns
        let provider_id = Self::infer_provider(model)
            .or(self.default_provider.as_deref())
            .ok_or_else(|| RouterError::ModelNotFound {
                model: model.to_string(),
            })?;

        self.providers
            .get(provider_id)
            .map(|p| p.as_ref())
            .ok_or_else(|| RouterError::ProviderNotConfigured {
                provider: provider_id.to_string(),
            })
    }

    /// Infer provider from model name patterns.
    fn infer_provider(model: &str) -> Option<&str> {
        let model_lower = model.to_lowercase();
        if model_lower.starts_with("gpt-") || model_lower.starts_with("o1") || model_lower.starts_with("o3") || model_lower.starts_with("o4") {
            Some("openai")
        } else if model_lower.starts_with("claude") {
            Some("anthropic")
        } else if model_lower.starts_with("gemini") {
            Some("gemini")
        } else if model_lower.starts_with("grok") {
            Some("grok")
        } else if model_lower.starts_with("deepseek") {
            Some("deepseek")
        } else if model_lower.starts_with("minimax") || model_lower.starts_with("abab") {
            Some("minimax")
        } else if model_lower.starts_with("glm") {
            Some("glm")
        } else if model_lower.starts_with("qwen") {
            Some("qwen")
        } else {
            None
        }
    }

    /// Send a non-streaming request, routing to the correct provider.
    pub async fn generate(&self, request: ChatRequest) -> Result<ChatResponse, RouterError> {
        let provider = self.resolve_provider(&request.model)?;
        let model = provider.language_model(&request.model);
        model.generate(request).await
    }

    /// Send a streaming request, routing to the correct provider.
    pub async fn stream(&self, request: ChatRequest) -> Result<ChatStream, RouterError> {
        let provider = self.resolve_provider(&request.model)?;
        let model = provider.language_model(&request.model);
        model.stream(request).await
    }

    /// Get a language model instance by name.
    pub fn model(&self, model_id: &str) -> Result<Box<dyn LanguageModel>, RouterError> {
        let provider = self.resolve_provider(model_id)?;
        Ok(provider.language_model(model_id))
    }
}

/// Builder for constructing a Gateway.
#[derive(Default)]
pub struct GatewayBuilder {
    providers: HashMap<String, Box<dyn Provider>>,
    model_routes: HashMap<String, String>,
    default_provider: Option<String>,
}

impl GatewayBuilder {
    /// Register a provider.
    pub fn with_provider(mut self, provider: impl Provider + 'static) -> Self {
        let id = provider.id().to_string();
        self.providers.insert(id, Box::new(provider));
        self
    }

    /// Map a specific model name to a provider.
    pub fn with_route(mut self, model: impl Into<String>, provider: impl Into<String>) -> Self {
        self.model_routes.insert(model.into(), provider.into());
        self
    }

    /// Set a default provider for unrecognized model names.
    pub fn with_default_provider(mut self, provider: impl Into<String>) -> Self {
        self.default_provider = Some(provider.into());
        self
    }

    /// Build the Gateway.
    pub fn build(self) -> Gateway {
        Gateway {
            providers: self.providers,
            model_routes: self.model_routes,
            default_provider: self.default_provider,
        }
    }
}

/// Run an agent loop that automatically handles tool calls.
///
/// Sends the request, executes any tool calls via `tool_executor`,
/// appends results, and repeats until the model stops calling tools
/// or `max_rounds` is reached.
pub async fn agent_loop<F, Fut>(
    model: &dyn LanguageModel,
    mut request: ChatRequest,
    tool_executor: F,
    max_rounds: usize,
) -> Result<ChatResponse, RouterError>
where
    F: Fn(ToolCall) -> Fut,
    Fut: Future<Output = ToolResult>,
{
    for round in 0..max_rounds {
        let response = model.generate(request.clone()).await?;

        let tool_calls = response.tool_calls();
        if tool_calls.is_empty() {
            return Ok(response);
        }

        // Add assistant message with tool use
        request.messages.push(response.to_assistant_message());

        // Execute each tool and add results
        for tc in tool_calls {
            let result = tool_executor(tc).await;
            request.messages.push(Message::tool_result(
                result.tool_call_id,
                result.content,
                result.is_error,
            ));
        }

        if round == max_rounds - 1 {
            return Err(RouterError::MaxRoundsExceeded { rounds: max_rounds });
        }
    }

    Err(RouterError::MaxRoundsExceeded { rounds: max_rounds })
}
