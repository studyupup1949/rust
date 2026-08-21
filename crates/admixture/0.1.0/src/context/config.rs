use std::time::Duration;

/// Configuration for test context behavior.
///
/// Controls timeouts, retry intervals, and other operational parameters
/// for context lifecycle management.
#[derive(Debug, Clone)]
pub struct ContextConfig {
    /// Maximum time to wait for all services to start
    pub startup_timeout: Duration,

    /// Interval between health check polls
    pub health_check_interval: Duration,

    /// Maximum time to wait for a single service to become healthy
    pub health_check_timeout: Duration,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            startup_timeout: Duration::from_secs(60),
            health_check_interval: Duration::from_millis(500),
            health_check_timeout: Duration::from_secs(30),
        }
    }
}

impl ContextConfig {
    /// Create a new context config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the startup timeout
    pub fn with_startup_timeout(mut self, timeout: Duration) -> Self {
        self.startup_timeout = timeout;
        self
    }

    /// Set the health check interval
    pub fn with_health_check_interval(mut self, interval: Duration) -> Self {
        self.health_check_interval = interval;
        self
    }

    /// Set the health check timeout
    pub fn with_health_check_timeout(mut self, timeout: Duration) -> Self {
        self.health_check_timeout = timeout;
        self
    }
}
