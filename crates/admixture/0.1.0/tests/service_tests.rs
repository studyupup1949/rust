//! Integration tests for the service! macro.

use admixture::service;
use admixture::service::{ServiceRunning, ServiceSetup};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TestServiceError {
    #[error("Test service failed")]
    #[allow(dead_code)]
    Failed,
}

service! {
    TestService {
        error: TestServiceError,
        client: String,

        setup {
            name: String,
        }

        running {
            name: String,
        }

        async fn start(self) -> Result<TestServiceRunning, TestServiceError> {
            Ok(TestServiceRunning { name: self.name })
        }

        async fn client(&self) -> Result<String, TestServiceError> {
            Ok(self.name.clone())
        }

        async fn healthy(&self) -> Result<(), TestServiceError> {
            Ok(())
        }

        async fn stop(&mut self) -> Result<(), TestServiceError> {
            Ok(())
        }
    }
}

#[tokio::test]
async fn test_service_macro_basic() -> eyre::Result<()> {
    let setup = TestServiceSetup {
        name: "test".to_string(),
    };

    let running = setup.start().await?;

    let client = running.client().await?;
    assert_eq!(client, "test");

    running.healthy().await?;

    Ok(())
}

#[tokio::test]
async fn test_service_macro_lifecycle() -> eyre::Result<()> {
    let setup = TestServiceSetup {
        name: "lifecycle_test".to_string(),
    };

    let mut running = setup.start().await?;

    // Verify client works
    let client = running.client().await?;
    assert_eq!(client, "lifecycle_test");

    // Verify health check works
    running.healthy().await?;

    // Verify stop works
    running.stop().await?;

    Ok(())
}
