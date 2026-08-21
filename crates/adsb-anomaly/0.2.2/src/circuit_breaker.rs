// ABOUTME: Circuit breaker pattern implementation for external service resilience
// ABOUTME: Prevents resource exhaustion during prolonged PiAware outages with state management

use std::sync::atomic::{AtomicI64, AtomicU32, AtomicU8, Ordering};
use tracing::{debug, error, info, warn};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed - all requests allowed
    Closed = 0,
    /// Circuit is open - requests blocked until timeout
    Open = 1,
    /// Circuit is half-open - single test request allowed
    HalfOpen = 2,
}

impl From<u8> for CircuitState {
    fn from(value: u8) -> Self {
        match value {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            2 => CircuitState::HalfOpen,
            _ => CircuitState::Closed, // Default fallback
        }
    }
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening circuit
    pub failure_threshold: u32,
    /// Timeout in milliseconds before attempting recovery
    pub timeout_ms: i64,
    /// Success threshold to close circuit from half-open state
    pub success_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout_ms: 30_000, // 30 seconds
            success_threshold: 2,
        }
    }
}

/// Thread-safe circuit breaker implementation
pub struct CircuitBreaker {
    failure_count: AtomicU32,
    success_count: AtomicU32,
    last_failure: AtomicI64,
    state: AtomicU8,
    config: CircuitBreakerConfig,
    name: String,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given configuration
    pub fn new(name: String, config: CircuitBreakerConfig) -> Self {
        info!(
            name = %name,
            failure_threshold = config.failure_threshold,
            timeout_ms = config.timeout_ms,
            success_threshold = config.success_threshold,
            "Creating circuit breaker"
        );

        Self {
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            last_failure: AtomicI64::new(0),
            state: AtomicU8::new(CircuitState::Closed as u8),
            config,
            name,
        }
    }

    /// Check if a request should be allowed through the circuit breaker
    pub fn should_allow_request(&self) -> bool {
        let current_state = CircuitState::from(self.state.load(Ordering::Relaxed));
        let now_ms = chrono::Utc::now().timestamp_millis();

        match current_state {
            CircuitState::Closed => {
                debug!(name = %self.name, state = "closed", "Request allowed");
                true
            }
            CircuitState::Open => {
                let last_failure = self.last_failure.load(Ordering::Relaxed);
                let time_since_failure = now_ms - last_failure;

                if time_since_failure > self.config.timeout_ms {
                    // Transition to half-open
                    self.state
                        .store(CircuitState::HalfOpen as u8, Ordering::Relaxed);
                    self.success_count.store(0, Ordering::Relaxed);

                    info!(
                        name = %self.name,
                        timeout_ms = self.config.timeout_ms,
                        time_since_failure_ms = time_since_failure,
                        "Circuit breaker transitioning to half-open state"
                    );
                    true
                } else {
                    debug!(
                        name = %self.name,
                        state = "open",
                        remaining_timeout_ms = self.config.timeout_ms - time_since_failure,
                        "Request blocked by open circuit"
                    );
                    false
                }
            }
            CircuitState::HalfOpen => {
                debug!(name = %self.name, state = "half_open", "Test request allowed");
                true
            }
        }
    }

    /// Record a successful request
    pub fn record_success(&self) {
        let current_state = CircuitState::from(self.state.load(Ordering::Relaxed));

        match current_state {
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::Relaxed);
                debug!(name = %self.name, "Success recorded in closed state");
            }
            CircuitState::HalfOpen => {
                let successes = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;

                if successes >= self.config.success_threshold {
                    // Transition back to closed
                    self.state
                        .store(CircuitState::Closed as u8, Ordering::Relaxed);
                    self.failure_count.store(0, Ordering::Relaxed);
                    self.success_count.store(0, Ordering::Relaxed);

                    info!(
                        name = %self.name,
                        success_threshold = self.config.success_threshold,
                        "Circuit breaker closed after successful recovery"
                    );
                } else {
                    debug!(
                        name = %self.name,
                        successes = successes,
                        threshold = self.config.success_threshold,
                        "Recovery in progress"
                    );
                }
            }
            CircuitState::Open => {
                warn!(name = %self.name, "Success recorded while circuit is open - unexpected state");
            }
        }
    }

    /// Record a failed request
    pub fn record_failure(&self, error: &str) {
        let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        let now_ms = chrono::Utc::now().timestamp_millis();
        self.last_failure.store(now_ms, Ordering::Relaxed);

        let current_state = CircuitState::from(self.state.load(Ordering::Relaxed));

        match current_state {
            CircuitState::Closed => {
                if failures >= self.config.failure_threshold {
                    // Transition to open
                    self.state
                        .store(CircuitState::Open as u8, Ordering::Relaxed);

                    error!(
                        name = %self.name,
                        failure_count = failures,
                        threshold = self.config.failure_threshold,
                        timeout_ms = self.config.timeout_ms,
                        error = %error,
                        "Circuit breaker opened due to failure threshold"
                    );
                } else {
                    warn!(
                        name = %self.name,
                        failure_count = failures,
                        threshold = self.config.failure_threshold,
                        error = %error,
                        "Failure recorded in closed state"
                    );
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open immediately goes back to open
                self.state
                    .store(CircuitState::Open as u8, Ordering::Relaxed);

                error!(
                    name = %self.name,
                    error = %error,
                    "Circuit breaker reopened after failed recovery attempt"
                );
            }
            CircuitState::Open => {
                debug!(
                    name = %self.name,
                    failure_count = failures,
                    error = %error,
                    "Failure recorded while circuit is already open"
                );
            }
        }
    }

    /// Get current circuit breaker state
    pub fn get_state(&self) -> CircuitState {
        CircuitState::from(self.state.load(Ordering::Relaxed))
    }

    /// Get current failure count
    pub fn get_failure_count(&self) -> u32 {
        self.failure_count.load(Ordering::Relaxed)
    }

    /// Get current success count (relevant in half-open state)
    pub fn get_success_count(&self) -> u32 {
        self.success_count.load(Ordering::Relaxed)
    }

    /// Get time since last failure in milliseconds
    #[cfg(test)]
    pub fn time_since_last_failure_ms(&self) -> i64 {
        let now_ms = chrono::Utc::now().timestamp_millis();
        let last_failure = self.last_failure.load(Ordering::Relaxed);
        now_ms - last_failure
    }

    /// Force circuit to closed state (for testing/admin purposes)
    #[cfg(test)]
    pub fn force_closed(&self) {
        self.state
            .store(CircuitState::Closed as u8, Ordering::Relaxed);
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
    }

    /// Force circuit to open state (for testing purposes)
    #[cfg(test)]
    pub fn force_open(&self) {
        self.state
            .store(CircuitState::Open as u8, Ordering::Relaxed);
        self.last_failure
            .store(chrono::Utc::now().timestamp_millis(), Ordering::Relaxed);
    }
}

impl CircuitBreaker {
    /// Execute a future through the circuit breaker
    pub async fn execute<F, Fut, T>(&self, operation: F) -> Result<T, CircuitBreakerError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
    {
        if !self.should_allow_request() {
            return Err(CircuitBreakerError::CircuitOpen);
        }

        match operation().await {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(error) => {
                self.record_failure(&error.to_string());
                Err(CircuitBreakerError::OperationFailed(error))
            }
        }
    }
}

/// Circuit breaker wrapper that automatically handles success/failure recording
#[cfg(test)]
pub struct CircuitBreakerWrapper<T> {
    circuit_breaker: CircuitBreaker,
    _phantom: std::marker::PhantomData<T>,
}

#[cfg(test)]
impl<T> CircuitBreakerWrapper<T> {
    pub fn new(name: String, config: CircuitBreakerConfig) -> Self {
        Self {
            circuit_breaker: CircuitBreaker::new(name, config),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Execute a future through the circuit breaker
    pub async fn execute<F, Fut>(&self, operation: F) -> Result<T, CircuitBreakerError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
    {
        self.circuit_breaker.execute(operation).await
    }

    pub fn get_state(&self) -> CircuitState {
        self.circuit_breaker.get_state()
    }

    pub fn get_failure_count(&self) -> u32 {
        self.circuit_breaker.get_failure_count()
    }

    pub fn force_closed(&self) {
        self.circuit_breaker.force_closed();
    }

    pub fn force_open(&self) {
        self.circuit_breaker.force_open();
    }
}

/// Circuit breaker specific errors
#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError {
    #[error("Circuit breaker is open, requests are blocked")]
    CircuitOpen,
    #[error("Operation failed: {0}")]
    OperationFailed(#[from] Box<dyn std::error::Error + Send + Sync>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_circuit_breaker_closed_state() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            timeout_ms: 1000,
            success_threshold: 2,
        };
        let cb = CircuitBreaker::new("test".to_string(), config);

        // Initial state should be closed
        assert_eq!(cb.get_state(), CircuitState::Closed);
        assert!(cb.should_allow_request());

        // Record success - should stay closed
        cb.record_success();
        assert_eq!(cb.get_state(), CircuitState::Closed);
        assert_eq!(cb.get_failure_count(), 0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_open_on_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            timeout_ms: 1000,
            success_threshold: 2,
        };
        let cb = CircuitBreaker::new("test".to_string(), config);

        // Record failures below threshold
        cb.record_failure("error 1");
        assert_eq!(cb.get_state(), CircuitState::Closed);
        assert!(cb.should_allow_request());

        cb.record_failure("error 2");
        assert_eq!(cb.get_state(), CircuitState::Closed);
        assert!(cb.should_allow_request());

        // Record failure that should open circuit
        cb.record_failure("error 3");
        assert_eq!(cb.get_state(), CircuitState::Open);
        assert!(!cb.should_allow_request());
        assert_eq!(cb.get_failure_count(), 3);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_transition() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout_ms: 100, // Short timeout for test
            success_threshold: 2,
        };
        let cb = CircuitBreaker::new("test".to_string(), config);

        // Open the circuit
        cb.record_failure("error 1");
        cb.record_failure("error 2");
        assert_eq!(cb.get_state(), CircuitState::Open);

        // Should not allow requests immediately
        assert!(!cb.should_allow_request());

        // Wait for timeout
        sleep(Duration::from_millis(150)).await;

        // Should transition to half-open and allow request
        assert!(cb.should_allow_request());
        assert_eq!(cb.get_state(), CircuitState::HalfOpen);
    }

    #[tokio::test]
    async fn test_circuit_breaker_recovery_to_closed() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout_ms: 50,
            success_threshold: 2,
        };
        let cb = CircuitBreaker::new("test".to_string(), config);

        // Open the circuit
        cb.record_failure("error 1");
        cb.record_failure("error 2");
        assert_eq!(cb.get_state(), CircuitState::Open);

        // Wait and transition to half-open
        sleep(Duration::from_millis(60)).await;
        assert!(cb.should_allow_request());
        assert_eq!(cb.get_state(), CircuitState::HalfOpen);

        // Record first success
        cb.record_success();
        assert_eq!(cb.get_state(), CircuitState::HalfOpen);

        // Record second success - should close circuit
        cb.record_success();
        assert_eq!(cb.get_state(), CircuitState::Closed);
        assert_eq!(cb.get_failure_count(), 0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_failure() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout_ms: 50,
            success_threshold: 2,
        };
        let cb = CircuitBreaker::new("test".to_string(), config);

        // Open the circuit
        cb.record_failure("error 1");
        cb.record_failure("error 2");

        // Wait and transition to half-open
        sleep(Duration::from_millis(60)).await;
        assert!(cb.should_allow_request());
        assert_eq!(cb.get_state(), CircuitState::HalfOpen);

        // Failure in half-open should immediately go back to open
        cb.record_failure("recovery failed");
        assert_eq!(cb.get_state(), CircuitState::Open);
        assert!(!cb.should_allow_request());
    }

    #[tokio::test]
    async fn test_circuit_breaker_execute_method() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout_ms: 100,
            success_threshold: 1,
        };
        let cb = CircuitBreaker::new("test".to_string(), config);

        // Successful operation
        let result = cb.execute(|| async { Ok("success".to_string()) }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");

        // Failed operations
        let result: Result<String, _> = cb.execute(|| async { Err("error 1".into()) }).await;
        assert!(result.is_err());

        let result: Result<String, _> = cb.execute(|| async { Err("error 2".into()) }).await;
        assert!(result.is_err());

        // Circuit should be open now
        let result: Result<String, _> = cb
            .execute(|| async { Ok("should not execute".to_string()) })
            .await;

        match result {
            Err(CircuitBreakerError::CircuitOpen) => {} // Expected
            other => panic!("Expected CircuitOpen error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker_wrapper() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout_ms: 100,
            success_threshold: 1,
        };
        let wrapper: CircuitBreakerWrapper<String> =
            CircuitBreakerWrapper::new("test".to_string(), config);

        // Successful operation
        let result = wrapper
            .execute(|| async { Ok("success".to_string()) })
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");

        // Failed operations
        let result = wrapper.execute(|| async { Err("error 1".into()) }).await;
        assert!(result.is_err());

        let result = wrapper.execute(|| async { Err("error 2".into()) }).await;
        assert!(result.is_err());

        // Circuit should be open now
        let result = wrapper
            .execute(|| async { Ok("should not execute".to_string()) })
            .await;

        match result {
            Err(CircuitBreakerError::CircuitOpen) => {} // Expected
            other => panic!("Expected CircuitOpen error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_time_since_last_failure() {
        let config = CircuitBreakerConfig::default();
        let cb = CircuitBreaker::new("test".to_string(), config);

        // Record a failure
        cb.record_failure("test error");

        // Should have recent failure time
        let time_since = cb.time_since_last_failure_ms();
        assert!(time_since >= 0);
        assert!(time_since < 1000); // Should be very recent
    }

    #[test]
    fn test_circuit_state_conversion() {
        assert_eq!(CircuitState::from(0), CircuitState::Closed);
        assert_eq!(CircuitState::from(1), CircuitState::Open);
        assert_eq!(CircuitState::from(2), CircuitState::HalfOpen);
        assert_eq!(CircuitState::from(99), CircuitState::Closed); // Invalid values default to Closed
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let config = CircuitBreakerConfig {
            failure_threshold: 5,
            timeout_ms: 100,
            success_threshold: 2,
        };
        let cb = Arc::new(CircuitBreaker::new("concurrent_test".to_string(), config));

        // Spawn multiple tasks that record failures concurrently
        let mut handles = Vec::new();
        for i in 0..10 {
            let cb_clone = cb.clone();
            let handle = tokio::spawn(async move {
                cb_clone.record_failure(&format!("concurrent error {}", i));
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Circuit should be open after 5 failures
        assert_eq!(cb.get_state(), CircuitState::Open);
        assert!(cb.get_failure_count() >= 5);
    }
}
