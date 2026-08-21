//! Integration tests for the context! macro.

use admixture::context;
use admixture::{context::ContextSetup, service::ServiceRunning};

// Mock service for testing
mod mock {
    use admixture::service::{ServiceRunning, ServiceSetup};
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum MockError {
        #[error("Mock service failed")]
        #[allow(dead_code)]
        Failed,
    }

    pub struct MockServiceSetup {
        pub name: String,
    }

    impl ServiceSetup for MockServiceSetup {
        type Running = MockServiceRunning;
        type Error = MockError;
        type Config = String;

        fn construct(name: String) -> Self {
            Self { name }
        }

        async fn start(self) -> Result<Self::Running, MockError> {
            Ok(MockServiceRunning { name: self.name })
        }
    }

    pub struct MockServiceRunning {
        name: String,
    }

    impl ServiceRunning for MockServiceRunning {
        type Client = String;
        type Error = MockError;

        async fn client(&self) -> Result<Self::Client, MockError> {
            Ok(self.name.clone())
        }

        async fn healthy(&self) -> Result<(), MockError> {
            Ok(())
        }

        async fn stop(&mut self) -> Result<(), MockError> {
            Ok(())
        }
    }
}

#[tokio::test]
async fn test_single_service_macro() -> eyre::Result<()> {
    // Define a context using the macro
    context! {
        SingleServiceTestCtx {
            service: mock::MockServiceSetup,
        }
    }

    // Create and use the context using the new API with config
    let config = SingleServiceTestCtxConfig {
        service: "test_service".to_string(),
    };
    let setup = SingleServiceTestCtxSetup::construct(config);
    let ctx = SingleServiceTestCtx::new(setup).build().await?;

    // Verify service is accessible
    let client = ctx.service().client().await?;
    assert_eq!(client, "test_service");

    // Stop the context
    ctx.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_multi_service_macro() -> eyre::Result<()> {
    // Define a context with multiple services
    context! {
        MultiServiceTestCtx {
            db: mock::MockServiceSetup,
            cache: mock::MockServiceSetup,
            api: mock::MockServiceSetup,
        }
    }

    // Create and use the context using the new API with config
    let config = MultiServiceTestCtxConfig {
        db: "database".to_string(),
        cache: "cache".to_string(),
        api: "api".to_string(),
    };
    let setup = MultiServiceTestCtxSetup::construct(config);
    let ctx = MultiServiceTestCtx::new(setup).build().await?;

    // Verify all services are accessible
    assert_eq!(ctx.db().client().await?, "database");
    assert_eq!(ctx.cache().client().await?, "cache");
    assert_eq!(ctx.api().client().await?, "api");

    // Stop the context
    ctx.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_macro_with_builder_config() -> eyre::Result<()> {
    use std::time::Duration;

    context! {
        ConfiguredTestCtx {
            service: mock::MockServiceSetup,
        }
    }

    // Use builder configuration with the new API and config
    let config = ConfiguredTestCtxConfig {
        service: "test".to_string(),
    };
    let setup = ConfiguredTestCtxSetup::construct(config);
    let ctx = ConfiguredTestCtx::new(setup)
        .with_startup_timeout(Duration::from_secs(30))
        .with_health_check_interval(Duration::from_millis(100))
        .build()
        .await?;

    let client = ctx.service().client().await?;
    assert_eq!(client, "test");

    ctx.stop().await?;

    Ok(())
}
