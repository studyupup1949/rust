use admixture::service::{ServiceRunning, ServiceSetup};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MockError {
    #[error("Mock service '{0}' configured to fail start")]
    StartFailed(String),

    #[error("Mock service '{0}' is stopped")]
    ServiceStopped(String),

    #[error("Mock service '{name}' not healthy yet (waiting for {delay:?}, elapsed: {elapsed:?})")]
    NotHealthyYet {
        name: String,
        delay: Duration,
        elapsed: Duration,
    },

    #[error("Mock service '{0}' configured to fail health check")]
    HealthCheckFailed(String),
}

#[derive(Clone, Debug)]
pub struct MockClient {
    #[allow(dead_code)]
    pub name: String,
}

impl MockClient {
    fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Clone, Debug)]
pub struct MockServiceConfig {
    name: String,
    should_fail_start: bool,
    should_fail_health: bool,
    health_delay: Option<Duration>,
}

impl MockServiceConfig {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            should_fail_start: false,
            should_fail_health: false,
            health_delay: None,
        }
    }

    #[allow(dead_code)]
    fn with_start_failure(mut self) -> Self {
        self.should_fail_start = true;
        self
    }

    #[allow(dead_code)]
    fn with_health_failure(mut self) -> Self {
        self.should_fail_health = true;
        self
    }

    #[allow(dead_code)]
    fn with_health_delay(mut self, delay: Duration) -> Self {
        self.health_delay = Some(delay);
        self
    }
}

pub struct MockServiceSetup {
    config: MockServiceConfig,
}

impl MockServiceSetup {
    fn new(config: MockServiceConfig) -> Self {
        Self { config }
    }

    pub fn healthy(name: impl Into<String>) -> Self {
        Self::new(MockServiceConfig::new(name))
    }

    #[allow(dead_code)]
    pub fn fails_start(name: impl Into<String>) -> Self {
        Self::new(MockServiceConfig::new(name).with_start_failure())
    }

    #[allow(dead_code)]
    pub fn fails_health(name: impl Into<String>) -> Self {
        Self::new(MockServiceConfig::new(name).with_health_failure())
    }

    pub fn delayed_health(name: impl Into<String>, delay: Duration) -> Self {
        Self::new(MockServiceConfig::new(name).with_health_delay(delay))
    }
}

impl ServiceSetup for MockServiceSetup {
    type Running = MockServiceRunning;
    type Error = MockError;
    type Config = MockServiceConfig;

    fn construct(config: Self::Config) -> Self {
        Self::new(config)
    }

    async fn start(self) -> Result<Self::Running, MockError> {
        if self.config.should_fail_start {
            return Err(MockError::StartFailed(self.config.name.clone()));
        }

        Ok(MockServiceRunning {
            config: self.config,
            started_at: Instant::now(),
            stopped: Arc::new(Mutex::new(false)),
        })
    }
}

pub struct MockServiceRunning {
    config: MockServiceConfig,
    started_at: Instant,
    pub stopped: Arc<Mutex<bool>>,
}

impl MockServiceRunning {
    pub fn is_stopped(&self) -> bool {
        *self.stopped.lock().unwrap()
    }
}

impl ServiceRunning for MockServiceRunning {
    type Client = MockClient;
    type Error = MockError;

    async fn client(&self) -> Result<Self::Client, MockError> {
        if self.is_stopped() {
            return Err(MockError::ServiceStopped(self.config.name.clone()));
        }

        Ok(MockClient::new(self.config.name.clone()))
    }

    async fn healthy(&self) -> Result<(), MockError> {
        if self.is_stopped() {
            return Err(MockError::ServiceStopped(self.config.name.clone()));
        }

        // Check if we should wait for a delay
        if let Some(delay) = self.config.health_delay
            && self.started_at.elapsed() < delay
        {
            return Err(MockError::NotHealthyYet {
                name: self.config.name.clone(),
                delay,
                elapsed: self.started_at.elapsed(),
            });
        }

        // Check if we should always fail
        if self.config.should_fail_health {
            return Err(MockError::HealthCheckFailed(self.config.name.clone()));
        }

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), MockError> {
        *self.stopped.lock().unwrap() = true;
        Ok(())
    }
}
