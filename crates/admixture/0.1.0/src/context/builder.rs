use super::{StoppableContext, config::ContextConfig, setup::ContextSetup};

/// Builder for creating and configuring test contexts.
///
/// Provides a fluent API for customizing context behavior before
/// starting services.
///
/// # Example
///
/// ```rust,ignore
/// let ctx = TestContext::builder(setup)
///     .with_startup_timeout(Duration::from_secs(60))
///     .with_health_check_interval(Duration::from_millis(500))
///     .build()
///     .await?;
/// ```
pub struct ContextBuilder<S> {
    setup: S,
    config: ContextConfig,
}

impl<S> ContextBuilder<S> {
    /// Create a new context builder with default configuration.
    pub fn new(setup: S) -> Self {
        Self {
            setup,
            config: ContextConfig::default(),
        }
    }

    /// Set the maximum time to wait for all services to start.
    ///
    /// Default: 60 seconds
    pub fn with_startup_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.config.startup_timeout = timeout;
        self
    }

    /// Set the interval between health check polls.
    ///
    /// Default: 500 milliseconds
    pub fn with_health_check_interval(mut self, interval: std::time::Duration) -> Self {
        self.config.health_check_interval = interval;
        self
    }

    /// Set the maximum time to wait for a single service to become healthy.
    ///
    /// Default: 30 seconds
    pub fn with_health_check_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.config.health_check_timeout = timeout;
        self
    }

    /// Build and start the test context.
    ///
    /// This method:
    /// 1. Calls `ContextSetup::start_all()` to start all services
    /// 2. Wraps the running context in a `StoppableContext` for lifecycle management
    /// 3. Returns the ready-to-use context with automatic cleanup on drop
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Any service fails to start
    /// - Any service fails to become healthy within the configured timeout
    pub async fn build<R>(self) -> Result<StoppableContext<R>, S::Error>
    where
        S: ContextSetup<Running = R>,
        R: super::setup::ContextRunning + Send + 'static,
    {
        tracing::info!("Starting test context");

        let running = self.setup.start_all(&self.config).await?;

        tracing::info!("Test context started successfully");

        Ok(StoppableContext { inner: Some(running) })
    }
}
