mod support;

use admixture::context::{
    ContextConfig, ContextError, ContextRunning, ContextSetup, TestContext,
    wait_until_healthy_with_config,
};
use admixture::service::{ServiceRunning, ServiceSetup};
use assert2::let_assert;
use std::time::Duration;
use support::mock_service::{MockServiceConfig, MockServiceRunning, MockServiceSetup};
struct SingleServiceContextSetup {
    service: MockServiceSetup,
}

struct SingleServiceContextRunning {
    service: MockServiceRunning,
}

impl ContextSetup for SingleServiceContextSetup {
    type Running = SingleServiceContextRunning;
    type Error = ContextError;
    type Config = MockServiceConfig;

    fn construct(config: Self::Config) -> Self {
        Self {
            service: MockServiceSetup::construct(config),
        }
    }

    async fn start_all(self, config: &ContextConfig) -> Result<Self::Running, ContextError> {
        let service = self
            .service
            .start()
            .await
            .map_err(|e| ContextError::ServiceStartFailed {
                source: Box::new(e),
            })?;
        wait_until_healthy_with_config(&service, config).await?;

        Ok(SingleServiceContextRunning { service })
    }
}

impl ContextRunning for SingleServiceContextRunning {
    type Error = ContextError;

    async fn stop_all(&mut self) -> Result<(), ContextError> {
        self.service
            .stop()
            .await
            .map_err(|e| ContextError::ShutdownFailed {
                source: Box::new(e),
            })?;
        Ok(())
    }
}

#[tokio::test]
async fn test_single_service_context() -> eyre::Result<()> {
    let setup = SingleServiceContextSetup {
        service: MockServiceSetup::healthy("test_service"),
    };

    let ctx = TestContext::builder(setup).build().await?;

    let client = ctx.service.client().await?;
    assert_eq!(client.name, "test_service");

    ctx.stop().await?;

    Ok(())
}

struct MultiServiceContextConfig {
    db: MockServiceConfig,
    cache: MockServiceConfig,
}

struct MultiServiceContextSetup {
    db: MockServiceSetup,
    cache: MockServiceSetup,
}

struct MultiServiceContextRunning {
    db: MockServiceRunning,
    cache: MockServiceRunning,
}

impl ContextSetup for MultiServiceContextSetup {
    type Running = MultiServiceContextRunning;
    type Error = ContextError;
    type Config = MultiServiceContextConfig;

    fn construct(config: Self::Config) -> Self {
        Self {
            db: MockServiceSetup::construct(config.db),
            cache: MockServiceSetup::construct(config.cache),
        }
    }

    async fn start_all(self, config: &ContextConfig) -> Result<Self::Running, ContextError> {
        let db = self
            .db
            .start()
            .await
            .map_err(|e| ContextError::ServiceStartFailed {
                source: Box::new(e),
            })?;
        wait_until_healthy_with_config(&db, config).await?;

        let cache = self
            .cache
            .start()
            .await
            .map_err(|e| ContextError::ServiceStartFailed {
                source: Box::new(e),
            })?;
        wait_until_healthy_with_config(&cache, config).await?;

        Ok(MultiServiceContextRunning { db, cache })
    }
}

impl ContextRunning for MultiServiceContextRunning {
    type Error = ContextError;

    async fn stop_all(&mut self) -> Result<(), ContextError> {
        let _ = self.cache.stop().await;
        let _ = self.db.stop().await;
        Ok(())
    }
}

#[tokio::test]
async fn test_multi_service_context() -> eyre::Result<()> {
    let setup = MultiServiceContextSetup {
        db: MockServiceSetup::healthy("postgres"),
        cache: MockServiceSetup::healthy("redis"),
    };

    let ctx = TestContext::builder(setup).build().await?;

    let db_client = ctx.db.client().await?;
    let cache_client = ctx.cache.client().await?;

    assert_eq!(db_client.name, "postgres");
    assert_eq!(cache_client.name, "redis");

    ctx.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_service_fails_to_start() {
    let setup = SingleServiceContextSetup {
        service: MockServiceSetup::fails_start("failing_service"),
    };

    let result = TestContext::builder(setup).build().await;

    let_assert!(Err(ContextError::ServiceStartFailed { .. }) = result);
}

#[tokio::test]
async fn test_service_fails_health_check() {
    let setup = SingleServiceContextSetup {
        service: MockServiceSetup::fails_health("unhealthy_service"),
    };

    let result = TestContext::builder(setup)
        .with_health_check_timeout(Duration::from_secs(2))
        .build()
        .await;

    let_assert!(Err(ContextError::HealthCheckTimeout { attempts, .. }) = result);
    assert!(attempts > 0, "Should have made multiple attempts");
}

struct OrderedContextConfig {
    first: MockServiceConfig,
    second: MockServiceConfig,
}

struct OrderedContextSetup {
    first: MockServiceSetup,
    second: MockServiceSetup,
}

struct OrderedContextRunning {
    first: MockServiceRunning,
    second: MockServiceRunning,
}

impl ContextSetup for OrderedContextSetup {
    type Running = OrderedContextRunning;
    type Error = ContextError;
    type Config = OrderedContextConfig;

    fn construct(config: Self::Config) -> Self {
        Self {
            first: MockServiceSetup::construct(config.first),
            second: MockServiceSetup::construct(config.second),
        }
    }

    async fn start_all(self, config: &ContextConfig) -> Result<Self::Running, ContextError> {
        let first = self
            .first
            .start()
            .await
            .map_err(|e| ContextError::ServiceStartFailed {
                source: Box::new(e),
            })?;
        wait_until_healthy_with_config(&first, config).await?;

        let second = self
            .second
            .start()
            .await
            .map_err(|e| ContextError::ServiceStartFailed {
                source: Box::new(e),
            })?;
        wait_until_healthy_with_config(&second, config).await?;

        Ok(OrderedContextRunning { first, second })
    }
}

impl ContextRunning for OrderedContextRunning {
    type Error = ContextError;

    async fn stop_all(&mut self) -> Result<(), ContextError> {
        let _ = self.second.stop().await;
        let _ = self.first.stop().await;
        Ok(())
    }
}

#[tokio::test]
async fn test_sequential_startup_order() -> eyre::Result<()> {
    let setup = OrderedContextSetup {
        first: MockServiceSetup::delayed_health("first", Duration::from_millis(500)),
        second: MockServiceSetup::healthy("second"),
    };

    let start = std::time::Instant::now();
    let ctx = TestContext::builder(setup).build().await?;

    assert!(start.elapsed() >= Duration::from_millis(500));

    let _ = ctx.first.client().await?;
    let _ = ctx.second.client().await?;

    ctx.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_builder_configuration() -> eyre::Result<()> {
    let setup = SingleServiceContextSetup {
        service: MockServiceSetup::healthy("test"),
    };

    let ctx = TestContext::builder(setup)
        .with_startup_timeout(Duration::from_secs(120))
        .with_health_check_interval(Duration::from_millis(200))
        .with_health_check_timeout(Duration::from_secs(60))
        .build()
        .await?;

    let _ = ctx.service.client().await?;

    ctx.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_delayed_health_with_retry() -> eyre::Result<()> {
    let setup = SingleServiceContextSetup {
        service: MockServiceSetup::delayed_health("delayed", Duration::from_secs(1)),
    };

    let start = std::time::Instant::now();

    let ctx = TestContext::builder(setup)
        .with_health_check_interval(Duration::from_millis(200))
        .build()
        .await?;

    assert!(start.elapsed() >= Duration::from_secs(1));

    let _ = ctx.service.client().await?;

    ctx.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_stop_with_multiple_services() -> eyre::Result<()> {
    let setup = MultiServiceContextSetup {
        db: MockServiceSetup::healthy("db"),
        cache: MockServiceSetup::healthy("cache"),
    };

    let ctx = TestContext::builder(setup).build().await?;

    assert!(!ctx.db.is_stopped());
    assert!(!ctx.cache.is_stopped());

    ctx.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_drop_cleanup_without_explicit_stop() -> eyre::Result<()> {
    use std::time::Duration;
    
    // Create a context but DON'T call .stop() explicitly
    // This tests that Drop handles cleanup
    let setup = SingleServiceContextSetup {
        service: MockServiceSetup::healthy("test_drop"),
    };

    let ctx = TestContext::builder(setup).build().await?;
    
    // Get a reference to check the stopped flag later
    // We need to clone the Arc<Mutex<bool>> to check it after ctx is dropped
    let stopped_flag = ctx.service.stopped.clone();
    
    // Verify service is not stopped yet
    assert!(!ctx.service.is_stopped());
    
    // Drop the context WITHOUT calling .stop()
    drop(ctx);
    
    // Give the spawned cleanup task a moment to complete
    // Drop spawns an async task that runs in the background
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Verify the service was stopped by Drop
    assert!(*stopped_flag.lock().unwrap(), "Service should be stopped by Drop");

    Ok(())
}
