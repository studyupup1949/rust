//! Service lifecycle management.
//!
//! A service can have 3 states:
//!
//! 1. Setup: The service is ready for takeoff
//! 2. Running: The service is actively running
//! 3. Stopped: The service used to run, but does not anymore

use std::future::Future;

pub trait ServiceSetup {
    type Running: ServiceRunning;
    type Error: std::error::Error + Send + Sync + 'static;

    /// Configuration needed to construct this service.
    /// Use `()` for services that don't need configuration.
    type Config;

    /// Construct a service setup from configuration.
    fn construct(config: Self::Config) -> Self;

    fn start(self) -> impl Future<Output = Result<Self::Running, Self::Error>>;
}

pub trait ServiceRunning {
    type Client;
    type Error: std::error::Error + Send + Sync + 'static;

    fn client(&self) -> impl Future<Output = Result<Self::Client, Self::Error>>;

    fn healthy(&self) -> impl Future<Output = Result<(), Self::Error>>;

    fn stop(&mut self) -> impl Future<Output = Result<(), Self::Error>>;
}
