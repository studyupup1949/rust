use std::time::{Duration, Instant};
use tracing::{debug, trace, warn};

use crate::service::ServiceRunning;

use super::config::ContextConfig;
use super::error::ContextError;

/// Wait for a service to become healthy with custom timeout and interval.
///
/// This function polls the service's `healthy()` method at regular intervals
/// until either:
/// - The service reports healthy (returns `Ok`)
/// - The timeout is reached (returns `Err`)
///
/// # Arguments
///
/// * `service` - The service to check
/// * `timeout` - Maximum time to wait for the service to become healthy
/// * `interval` - Time to wait between health check attempts
///
/// # Example
///
/// ```rust,ignore
/// let service = postgres.start().await?;
/// wait_until_healthy(&service, Duration::from_secs(30), Duration::from_millis(500)).await?;
/// ```
pub async fn wait_until_healthy<S: ServiceRunning>(
    service: &S,
    timeout: Duration,
    interval: Duration,
) -> Result<(), S::Error> {
    let start = Instant::now();
    let mut attempt = 0;

    debug!(
        timeout_secs = timeout.as_secs(),
        interval_ms = interval.as_millis(),
        "Starting health check polling"
    );

    loop {
        attempt += 1;
        trace!(
            attempt,
            elapsed_ms = start.elapsed().as_millis(),
            "Checking service health"
        );

        match service.healthy().await {
            Ok(()) => {
                debug!(
                    attempt,
                    elapsed_ms = start.elapsed().as_millis(),
                    "Service became healthy"
                );
                return Ok(());
            }
            Err(e) if start.elapsed() < timeout => {
                trace!(
                    attempt,
                    elapsed_ms = start.elapsed().as_millis(),
                    error = %e,
                    "Service not healthy yet, will retry"
                );
                tokio::time::sleep(interval).await;
                continue;
            }
            Err(e) => {
                warn!(
                    attempt,
                    elapsed_ms = start.elapsed().as_millis(),
                    timeout_secs = timeout.as_secs(),
                    "Service failed to become healthy within timeout"
                );
                return Err(e);
            }
        }
    }
}

/// Wait for a service to become healthy using config defaults.
///
/// This is a convenience wrapper around `wait_until_healthy` that uses
/// the timeout and interval from a `ContextConfig`.
///
/// Returns `ContextError::HealthCheckTimeout` on timeout, wrapping the service error.
///
/// # Example
///
/// ```rust,ignore
/// let service = postgres.start().await?;
/// wait_until_healthy_with_config(&service, &config).await?;
/// ```
pub async fn wait_until_healthy_with_config<S: ServiceRunning>(
    service: &S,
    config: &ContextConfig,
) -> Result<(), ContextError> {
    let start = Instant::now();
    let mut attempt = 0;

    let timeout = config.health_check_timeout;
    let interval = config.health_check_interval;

    debug!(
        timeout_secs = timeout.as_secs(),
        interval_ms = interval.as_millis(),
        "Starting health check polling"
    );

    loop {
        attempt += 1;
        trace!(
            attempt,
            elapsed_ms = start.elapsed().as_millis(),
            "Checking service health"
        );

        match service.healthy().await {
            Ok(()) => {
                debug!(
                    attempt,
                    elapsed_ms = start.elapsed().as_millis(),
                    "Service became healthy"
                );
                return Ok(());
            }
            Err(e) if start.elapsed() < timeout => {
                trace!(
                    attempt,
                    elapsed_ms = start.elapsed().as_millis(),
                    error = %e,
                    "Service not healthy yet, will retry"
                );
                tokio::time::sleep(interval).await;
                continue;
            }
            Err(e) => {
                warn!(
                    attempt,
                    elapsed_ms = start.elapsed().as_millis(),
                    timeout_secs = timeout.as_secs(),
                    "Service failed to become healthy within timeout"
                );
                return Err(ContextError::HealthCheckTimeout {
                    elapsed: start.elapsed(),
                    attempts: attempt,
                    source: Box::new(e),
                });
            }
        }
    }
}
