use std::marker::PhantomData;

use testcontainers::{ContainerAsync, Image, runners::AsyncRunner};

use admixture::service::{ServiceRunning, ServiceSetup};

use crate::container::ContainerService;
use crate::error::DockerError;

pub struct DockerContainerServiceSetup<I: Image + Send + Clone, Wrap: ContainerService> {
    image: I,
    _wrap: PhantomData<Wrap>,
}

impl<I: Image + Send + Clone, Wrap: ContainerService> DockerContainerServiceSetup<I, Wrap> {
    pub fn construct(image: I) -> Self {
        Self {
            image,
            _wrap: PhantomData,
        }
    }
}

impl<I: Image + Send + Clone, ServiceContext: ContainerService> ServiceSetup
    for DockerContainerServiceSetup<I, ServiceContext>
{
    type Running = DockerContainerServiceRunning<I, ServiceContext>;
    type Error = DockerError;
    type Config = I;

    fn construct(image: I) -> Self {
        Self {
            image,
            _wrap: PhantomData,
        }
    }

    async fn start(self) -> Result<Self::Running, DockerError> {
        let container = self
            .image
            .start()
            .await
            .map_err(|e| DockerError::ContainerStartFailed(Box::new(e)))?;

        let wrapper = ServiceContext::construct(&container)
            .await
            .map_err(|e| DockerError::WrapperConstructionFailed(Box::new(e)))?;

        Ok(DockerContainerServiceRunning { container, wrapper })
    }
}

pub struct DockerContainerServiceRunning<I: Image + Send + Clone, Wrap: ContainerService> {
    container: ContainerAsync<I>,
    wrapper: Wrap,
}

impl<I: Image + Send + Clone, Wrap: ContainerService> ServiceRunning
    for DockerContainerServiceRunning<I, Wrap>
{
    type Client = Wrap::Client;
    type Error = DockerError;

    async fn client(&self) -> Result<Self::Client, DockerError> {
        self.wrapper
            .client()
            .await
            .map_err(|e| DockerError::OperationFailed(format!("Failed to get client: {}", e)))
    }

    async fn healthy(&self) -> Result<(), DockerError> {
        self.wrapper
            .healthy()
            .await
            .map_err(|e| DockerError::OperationFailed(format!("Health check failed: {}", e)))
    }

    async fn stop(&mut self) -> Result<(), DockerError> {
        self.container
            .stop()
            .await
            .map_err(|e| DockerError::ContainerStopFailed(Box::new(e)))
    }
}

pub trait DockerContainerSetup {
    type Image: Image;
}
