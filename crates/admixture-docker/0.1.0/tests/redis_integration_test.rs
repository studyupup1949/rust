//! Integration tests using real Docker containers with Redis.
//!
//! This test demonstrates using the docker_service! macro to define a Redis service.

#![cfg(feature = "fred-redis")]

use admixture::context;
use admixture::context::ContextSetup;
use admixture::service::ServiceRunning;
use admixture_docker::docker_service;
use fred::prelude::*;
use testcontainers_modules::redis::Redis;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RedisMacroError {
    #[error("Failed to get container host")]
    HostResolutionFailed(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Failed to get container port for 6379")]
    PortMappingFailed(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Redis client error: {0}")]
    ClientError(#[from] fred::prelude::Error),

    #[error("Failed to connect to Redis: {0}")]
    ConnectionFailed(String),
}

// Define Redis service using the docker_service! macro
docker_service! {
    RedisMacro {
        image: Redis,
        error: RedisMacroError,
        client: Client,

        context {
            client: Client,
        }

        async fn construct<I: testcontainers::Image>(
            container: &testcontainers::ContainerAsync<I>
        ) -> Result<Self, RedisMacroError> {
            let host = container
                .get_host()
                .await
                .map_err(|e| RedisMacroError::HostResolutionFailed(Box::new(e)))?
                .to_string();

            let port = container
                .get_host_port_ipv4(6379)
                .await
                .map_err(|e| RedisMacroError::PortMappingFailed(Box::new(e)))?;

            let config = Config::from_url(&format!("redis://{}:{}", host, port))
                .map_err(|e| RedisMacroError::ConnectionFailed(e.to_string()))?;

            let client = Builder::from_config(config).build()?;
            client.init().await?;

            Ok(Self { client })
        }

        async fn client(&self) -> Result<Client, RedisMacroError> {
            Ok(self.client.clone())
        }

        async fn healthy(&self) -> Result<(), RedisMacroError> {
            let _: String = self.client.ping(None).await?;
            Ok(())
        }
    }
}

// Also test with the manual implementation from the library
use admixture_docker::FredRedisServiceSetup;

// Define test contexts - one using the macro, one using the library implementation
context! {
    RedisTestMacroContext {
        redis: RedisMacroServiceSetup = Redis::default(),
    }
}

context! {
    RedisContext {
        redis: FredRedisServiceSetup = Redis::default(),
    }
}


#[tokio::test]
async fn test_redis_macro_context_lifecycle() -> eyre::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();

    let config = RedisTestMacroContextConfig {
        redis: Redis::default(),
    };
    let setup = RedisTestMacroContextSetup::construct(config);
    let ctx = RedisTestMacroContext::new(setup).build().await?;

    let client = ctx.redis().client().await?;

    let result: String = client.ping(None).await?;

    assert_eq!(result, "PONG", "PING should return PONG");

    ctx.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_redis_context_lifecycle() -> eyre::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();

    let config = RedisContextConfig {
        redis: Redis::default(),
    };
    let setup = RedisContextSetup::construct(config);
    let ctx = RedisContext::new(setup).build().await?;

    let client = ctx.redis().client().await?;

    let result: String = client.ping(None).await?;

    assert_eq!(result, "PONG", "PING should return PONG");

    ctx.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_redis_context_with_data_operations() -> eyre::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();

    let config = RedisContextConfig {
        redis: Redis::default(),
    };
    let setup = RedisContextSetup::construct(config);
    let ctx = RedisContext::new(setup).build().await?;
    let client = ctx.redis().client().await?;

    // Set a key-value pair
    let _: () = client.set("test_key", "test_value", None, None, false).await?;

    // Get the value
    let value: String = client.get("test_key").await?;

    assert_eq!(value, "test_value", "Should retrieve the set value");

    // Test key expiration
    let _: () = client.set("expiring_key", "temp_value", Some(Expiration::EX(1)), None, false).await?;
    let exists: bool = client.exists("expiring_key").await?;
    assert!(exists, "Key should exist immediately after setting");

    ctx.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_redis_context_health_check() -> eyre::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();

    let config = RedisContextConfig {
        redis: Redis::default(),
    };
    let setup = RedisContextSetup::construct(config);
    let ctx = RedisContext::new(setup).build().await?;

    ctx.redis().healthy().await?;

    ctx.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_redis_context_with_lists() -> eyre::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();

    let config = RedisContextConfig {
        redis: Redis::default(),
    };
    let setup = RedisContextSetup::construct(config);
    let ctx = RedisContext::new(setup).build().await?;
    let client = ctx.redis().client().await?;

    // Push items to a list
    let _: () = client.lpush("test_list", vec!["item1", "item2", "item3"]).await?;

    // Get list length
    let length: i64 = client.llen("test_list").await?;
    assert_eq!(length, 3, "List should have 3 items");

    // Pop an item
    let item: String = client.lpop("test_list", None).await?;
    assert_eq!(item, "item3", "Should pop the most recently pushed item");

    ctx.stop().await?;

    Ok(())
}
