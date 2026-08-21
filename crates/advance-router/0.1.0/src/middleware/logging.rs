use std::time::Instant;

use async_trait::async_trait;
use tracing::info;

use crate::error::RouterError;
use crate::model::LanguageModel;

use crate::types::request::ChatRequest;
use crate::types::response::ChatResponse;

use super::Middleware;

/// Middleware that logs request/response metadata using `tracing`.
pub struct LoggingMiddleware;

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn process(
        &self,
        request: ChatRequest,
        next: &dyn LanguageModel,
    ) -> Result<ChatResponse, RouterError> {
        let model = request.model.clone();
        let msg_count = request.messages.len();
        let tool_count = request.tools.len();
        let start = Instant::now();

        info!(
            model = model,
            messages = msg_count,
            tools = tool_count,
            "Sending request"
        );

        let result = next.generate(request).await;
        let elapsed = start.elapsed();

        match &result {
            Ok(resp) => {
                info!(
                    model = model,
                    elapsed_ms = elapsed.as_millis() as u64,
                    prompt_tokens = resp.usage.prompt_tokens,
                    completion_tokens = resp.usage.completion_tokens,
                    finish_reason = ?resp.finish_reason,
                    "Request completed"
                );
            }
            Err(e) => {
                info!(
                    model = model,
                    elapsed_ms = elapsed.as_millis() as u64,
                    error = %e,
                    "Request failed"
                );
            }
        }

        result
    }
}
