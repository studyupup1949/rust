use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::Semaphore;

use crate::error::RouterError;
use crate::model::LanguageModel;
use crate::stream::ChatStream;
use crate::types::request::ChatRequest;
use crate::types::response::ChatResponse;

use super::Middleware;

/// Simple concurrency-based rate limiter using a semaphore.
pub struct RateLimitMiddleware {
    semaphore: Arc<Semaphore>,
    provider: String,
    timeout: Duration,
}

impl RateLimitMiddleware {
    /// Create a rate limiter that allows `max_concurrent` simultaneous requests.
    pub fn new(provider: impl Into<String>, max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            provider: provider.into(),
            timeout: Duration::from_secs(60),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

#[async_trait]
impl Middleware for RateLimitMiddleware {
    async fn process(
        &self,
        request: ChatRequest,
        next: &dyn LanguageModel,
    ) -> Result<ChatResponse, RouterError> {
        let _permit = tokio::time::timeout(self.timeout, self.semaphore.acquire())
            .await
            .map_err(|_| RouterError::RateLimited {
                provider: self.provider.clone(),
                retry_after: Some(self.timeout),
            })?
            .map_err(|_| RouterError::Stream("Semaphore closed".into()))?;

        next.generate(request).await
    }

    async fn process_stream(
        &self,
        request: ChatRequest,
        next: &dyn LanguageModel,
    ) -> Result<ChatStream, RouterError> {
        let _permit = tokio::time::timeout(self.timeout, self.semaphore.acquire())
            .await
            .map_err(|_| RouterError::RateLimited {
                provider: self.provider.clone(),
                retry_after: Some(self.timeout),
            })?
            .map_err(|_| RouterError::Stream("Semaphore closed".into()))?;

        next.stream(request).await
    }
}
