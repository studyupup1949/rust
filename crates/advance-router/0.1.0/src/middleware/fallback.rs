use async_trait::async_trait;
use tracing::warn;

use crate::error::RouterError;
use crate::model::LanguageModel;
use crate::stream::ChatStream;
use crate::types::request::ChatRequest;
use crate::types::response::ChatResponse;

use super::Middleware;

/// Middleware that tries fallback models when the primary model fails.
pub struct FallbackMiddleware {
    /// Ordered list of (model_id, model) pairs to try as fallbacks.
    pub fallbacks: Vec<(String, Box<dyn LanguageModel>)>,
}

impl FallbackMiddleware {
    pub fn new(fallbacks: Vec<(String, Box<dyn LanguageModel>)>) -> Self {
        Self { fallbacks }
    }
}

#[async_trait]
impl Middleware for FallbackMiddleware {
    async fn process(
        &self,
        request: ChatRequest,
        next: &dyn LanguageModel,
    ) -> Result<ChatResponse, RouterError> {
        match next.generate(request.clone()).await {
            Ok(resp) => return Ok(resp),
            Err(e) if matches!(e, RouterError::Auth { .. }) => return Err(e),
            Err(e) => {
                warn!(
                    primary = next.model_id(),
                    error = %e,
                    "Primary model failed, trying fallbacks"
                );

                let mut errors = vec![e];

                for (model_id, fallback) in &self.fallbacks {
                    let mut fallback_req = request.clone();
                    fallback_req.model = model_id.clone();

                    match fallback.generate(fallback_req).await {
                        Ok(resp) => return Ok(resp),
                        Err(e) => {
                            warn!(fallback = model_id, error = %e, "Fallback model failed");
                            errors.push(e);
                        }
                    }
                }

                Err(RouterError::AllFallbacksFailed { errors })
            }
        }
    }

    async fn process_stream(
        &self,
        request: ChatRequest,
        next: &dyn LanguageModel,
    ) -> Result<ChatStream, RouterError> {
        match next.stream(request.clone()).await {
            Ok(stream) => return Ok(stream),
            Err(e) if matches!(e, RouterError::Auth { .. }) => return Err(e),
            Err(e) => {
                warn!(
                    primary = next.model_id(),
                    error = %e,
                    "Primary model stream failed, trying fallbacks"
                );

                let mut errors = vec![e];

                for (model_id, fallback) in &self.fallbacks {
                    let mut fallback_req = request.clone();
                    fallback_req.model = model_id.clone();

                    match fallback.stream(fallback_req).await {
                        Ok(stream) => return Ok(stream),
                        Err(e) => {
                            warn!(fallback = model_id, error = %e, "Fallback stream failed");
                            errors.push(e);
                        }
                    }
                }

                Err(RouterError::AllFallbacksFailed { errors })
            }
        }
    }
}
