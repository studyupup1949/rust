use std::time::Duration;

use async_trait::async_trait;
use tracing::warn;

use crate::error::RouterError;
use crate::model::LanguageModel;
use crate::stream::ChatStream;
use crate::types::request::ChatRequest;
use crate::types::response::ChatResponse;

use super::Middleware;

/// Middleware that retries requests on transient failures with exponential backoff.
pub struct RetryMiddleware {
    pub max_retries: u32,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
}

impl Default for RetryMiddleware {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
        }
    }
}

impl RetryMiddleware {
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            ..Default::default()
        }
    }

    fn backoff_duration(&self, attempt: u32, retry_after: Option<Duration>) -> Duration {
        if let Some(retry_after) = retry_after {
            return retry_after;
        }
        let backoff = self.initial_backoff.as_millis() as u64 * 2u64.pow(attempt);
        Duration::from_millis(backoff.min(self.max_backoff.as_millis() as u64))
    }
}

#[async_trait]
impl Middleware for RetryMiddleware {
    async fn process(
        &self,
        request: ChatRequest,
        next: &dyn LanguageModel,
    ) -> Result<ChatResponse, RouterError> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match next.generate(request.clone()).await {
                Ok(response) => return Ok(response),
                Err(e) if e.is_retryable() && attempt < self.max_retries => {
                    let retry_after = if let RouterError::RateLimited { retry_after, .. } = &e {
                        *retry_after
                    } else {
                        None
                    };
                    let backoff = self.backoff_duration(attempt, retry_after);
                    warn!(
                        attempt = attempt + 1,
                        max_retries = self.max_retries,
                        backoff_ms = backoff.as_millis() as u64,
                        error = %e,
                        "Retrying request"
                    );
                    tokio::time::sleep(backoff).await;
                    last_error = Some(e);
                }
                Err(e) => return Err(e),
            }
        }

        Err(last_error.unwrap())
    }

    async fn process_stream(
        &self,
        request: ChatRequest,
        next: &dyn LanguageModel,
    ) -> Result<ChatStream, RouterError> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match next.stream(request.clone()).await {
                Ok(stream) => return Ok(stream),
                Err(e) if e.is_retryable() && attempt < self.max_retries => {
                    let retry_after = if let RouterError::RateLimited { retry_after, .. } = &e {
                        *retry_after
                    } else {
                        None
                    };
                    let backoff = self.backoff_duration(attempt, retry_after);
                    warn!(
                        attempt = attempt + 1,
                        max_retries = self.max_retries,
                        backoff_ms = backoff.as_millis() as u64,
                        error = %e,
                        "Retrying stream request"
                    );
                    tokio::time::sleep(backoff).await;
                    last_error = Some(e);
                }
                Err(e) => return Err(e),
            }
        }

        Err(last_error.unwrap())
    }
}
