//! Integration tests for the docker_service! macro.

use admixture_docker::docker_service;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MockDockerError {
    #[error("Container host resolution failed")]
    HostResolutionFailed(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Container port mapping failed")]
    PortMappingFailed(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Service operation failed")]
    OperationFailed,
}

// Mock image type for testing
#[derive(Debug, Clone)]
pub struct MockImage;

impl testcontainers::Image for MockImage {
    fn name(&self) -> &str {
        "mock"
    }

    fn tag(&self) -> &str {
        "latest"
    }

    fn ready_conditions(&self) -> Vec<testcontainers::core::WaitFor> {
        vec![]
    }
}

docker_service! {
    MockDocker {
        image: MockImage,
        error: MockDockerError,
        client: String,

        context {
            connection: String,
        }

        async fn construct<I: testcontainers::Image>(
            container: &testcontainers::ContainerAsync<I>
        ) -> Result<Self, MockDockerError> {
            let host = container
                .get_host()
                .await
                .map_err(|e| MockDockerError::HostResolutionFailed(Box::new(e)))?
                .to_string();

            Ok(Self {
                connection: format!("mock://{}:1234", host),
            })
        }

        async fn client(&self) -> Result<String, MockDockerError> {
            Ok(self.connection.clone())
        }

        async fn healthy(&self) -> Result<(), MockDockerError> {
            // Mock health check always succeeds
            Ok(())
        }
    }
}

#[tokio::test]
async fn test_docker_service_macro_generates_types() {
    // This test just verifies that the macro generates the expected types
    // We can't actually start a container in this test without Docker running

    // Verify the type alias exists and construct works
    let _setup: MockDockerServiceSetup =
        admixture_docker::DockerContainerServiceSetup::construct(MockImage);

    // If we got here, the types were generated successfully
}

#[tokio::test]
async fn test_docker_service_context_struct() {
    // Test that we can construct the context struct directly
    let context = MockDockerContext {
        connection: "mock://localhost:1234".to_string(),
    };

    // Verify we can get the client
    let client = admixture_docker::ContainerService::client(&context).await;
    assert!(client.is_ok());
    assert_eq!(client.unwrap(), "mock://localhost:1234");

    // Verify health check works
    let health = admixture_docker::ContainerService::healthy(&context).await;
    assert!(health.is_ok());
}
