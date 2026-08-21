use async_trait::async_trait;

use crate::error::RouterError;
use crate::stream::ChatStream;
use crate::types::embedding::{EmbeddingRequest, EmbeddingResponse};
use crate::types::request::ChatRequest;
use crate::types::response::ChatResponse;

/// Capabilities that a model may support.
#[derive(Debug, Clone, Default)]
pub struct ModelCapabilities {
    pub streaming: bool,
    pub tool_calling: bool,
    pub vision: bool,
    pub json_mode: bool,
    pub json_schema: bool,
    pub extended_thinking: bool,
    pub embeddings: bool,
}

/// Core trait for a language model that supports text generation.
#[async_trait]
pub trait LanguageModel: Send + Sync {
    /// Model identifier, e.g. "claude-sonnet-4-20250514" or "gpt-4o".
    fn model_id(&self) -> &str;

    /// Provider name, e.g. "anthropic", "openai".
    fn provider_id(&self) -> &str;

    /// Non-streaming completion.
    async fn generate(&self, request: ChatRequest) -> Result<ChatResponse, RouterError>;

    /// Streaming completion -- returns an async stream of events.
    async fn stream(&self, request: ChatRequest) -> Result<ChatStream, RouterError>;

    /// What capabilities does this model support?
    fn capabilities(&self) -> ModelCapabilities;
}

/// Trait for models that support embedding generation.
#[async_trait]
pub trait EmbeddingModel: Send + Sync {
    fn model_id(&self) -> &str;
    fn provider_id(&self) -> &str;
    fn dimensions(&self) -> Option<usize>;
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, RouterError>;
}
