use fred::prelude::*;
use testcontainers_modules::redis::Redis;
use thiserror::Error;

use crate::{ContainerService, DockerContainerServiceSetup};

#[derive(Debug, Error)]
pub enum RedisError {
    #[error("Failed to get container host")]
    HostResolutionFailed(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Failed to get container port for 6379")]
    PortMappingFailed(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Redis client error: {0}")]
    ClientError(#[from] Error),

    #[error("Failed to connect to Redis: {0}")]
    ConnectionFailed(String),
}

pub type FredRedisServiceSetup = DockerContainerServiceSetup<Redis, FredRedisContext>;

pub struct FredRedisContext {
    client: Client,
}

impl ContainerService for FredRedisContext {
    type Client = Client;
    type Error = RedisError;

    async fn construct<I: testcontainers::Image>(
        container: &testcontainers::ContainerAsync<I>,
    ) -> Result<Self, RedisError> {
        let host = container
            .get_host()
            .await
            .map_err(|e| RedisError::HostResolutionFailed(Box::new(e)))?
            .to_string();

        let port = container
            .get_host_port_ipv4(6379)
            .await
            .map_err(|e| RedisError::PortMappingFailed(Box::new(e)))?;

        let config = Config::from_url(&format!("redis://{}:{}", host, port))
            .map_err(|e| RedisError::ConnectionFailed(e.to_string()))?;

        let client = Builder::from_config(config).build()?;
        client.init().await?;

        Ok(Self { client })
    }

    async fn client(&self) -> Result<Self::Client, RedisError> {
        Ok(self.client.clone())
    }

    async fn healthy(&self) -> Result<(), RedisError> {
        let _: String = self.client.ping(None).await?;
        Ok(())
    }
}
