pub mod fallback;
pub mod logging;
pub mod rate_limit;
pub mod retry;

use async_trait::async_trait;

use crate::error::RouterError;
use crate::model::LanguageModel;
use crate::stream::ChatStream;
use crate::types::request::ChatRequest;
use crate::types::response::ChatResponse;

/// Middleware that can intercept and modify LLM requests and responses.
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Process a non-streaming request.
    async fn process(
        &self,
        request: ChatRequest,
        next: &dyn LanguageModel,
    ) -> Result<ChatResponse, RouterError>;

    /// Process a streaming request. Default implementation passes through.
    async fn process_stream(
        &self,
        request: ChatRequest,
        next: &dyn LanguageModel,
    ) -> Result<ChatStream, RouterError> {
        next.stream(request).await
    }
}
