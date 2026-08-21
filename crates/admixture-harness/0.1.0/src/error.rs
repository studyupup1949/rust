//! Error types for the test harness.

use thiserror::Error;

/// Errors that can occur during test execution.
#[derive(Debug, Error)]
pub enum TestError {
    /// Context failed to start.
    #[error("Failed to start test context: {0}")]
    ContextStartFailed(#[source] Box<dyn std::error::Error + Send + Sync>),

    /// Context failed to stop.
    #[error("Failed to stop test context: {0}")]
    ContextStopFailed(#[source] Box<dyn std::error::Error + Send + Sync>),

    /// Hook execution failed.
    #[error("Hook '{hook_name}' failed: {source}")]
    HookFailed {
        hook_name: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Test execution failed.
    #[error("Test failed: {0}")]
    TestFailed(String),

    /// Generic error.
    #[error("{0}")]
    Other(String),
}

impl TestError {
    /// Create a new context start error.
    pub fn context_start_failed<E: std::error::Error + Send + Sync + 'static>(error: E) -> Self {
        TestError::ContextStartFailed(Box::new(error))
    }

    /// Create a new context stop error.
    pub fn context_stop_failed<E: std::error::Error + Send + Sync + 'static>(error: E) -> Self {
        TestError::ContextStopFailed(Box::new(error))
    }

    /// Create a new hook failed error.
    pub fn hook_failed<E: std::error::Error + Send + Sync + 'static>(
        hook_name: impl Into<String>,
        error: E,
    ) -> Self {
        TestError::HookFailed {
            hook_name: hook_name.into(),
            source: Box::new(error),
        }
    }
}
