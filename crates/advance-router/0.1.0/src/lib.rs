pub mod error;
pub mod gateway;
pub mod http;
pub mod middleware;
pub mod model;
pub mod provider;
pub mod providers;
pub mod stream;
pub mod types;

// Re-export core types for convenience
pub use error::RouterError;
pub use gateway::{agent_loop, Gateway, GatewayBuilder};
pub use model::{EmbeddingModel, LanguageModel, ModelCapabilities};
pub use provider::{Provider, ProviderConfig};
pub use stream::ChatStream;
pub use types::embedding::{EmbeddingRequest, EmbeddingResponse, EmbeddingUsage};
pub use types::message::{ContentPart, Message, MessageContent, Role};
pub use types::request::{ChatRequest, ExtendedThinking};
pub use types::response::{ChatResponse, FinishReason, StreamEvent, Usage};
pub use types::tool::{ToolCall, ToolChoice, ToolDefinition, ToolResult};

// Re-export providers
#[cfg(feature = "openai")]
pub use providers::openai::OpenAIProvider;

#[cfg(feature = "anthropic")]
pub use providers::anthropic::AnthropicProvider;

#[cfg(feature = "gemini")]
pub use providers::gemini::GeminiProvider;

#[cfg(feature = "deepseek")]
pub use providers::deepseek::DeepSeekProvider;

#[cfg(feature = "grok")]
pub use providers::grok::GrokProvider;

#[cfg(feature = "minimax")]
pub use providers::minimax::MinimaxProvider;

#[cfg(feature = "glm")]
pub use providers::glm::GLMProvider;

#[cfg(feature = "qwen")]
pub use providers::qwen::QwenProvider;
