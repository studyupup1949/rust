use sqlx::{PgPool, postgres::PgConnectOptions};
use testcontainers_modules::postgres::Postgres;
use thiserror::Error;

use crate::{ContainerService, DockerContainerServiceSetup};

#[derive(Debug, Error)]
pub enum PostgresError {
    #[error("Failed to get container host")]
    HostResolutionFailed(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Failed to get container port for 5432")]
    PortMappingFailed(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Database query failed: {0}")]
    QueryFailed(#[from] sqlx::Error),
}

pub type SqlxPostgresServiceSetup = DockerContainerServiceSetup<Postgres, SqlxPostgresContext>;

pub struct SqlxPostgresContext {
    pool: PgPool,
}

impl ContainerService for SqlxPostgresContext {
    type Client = PgPool;
    type Error = PostgresError;

    async fn construct<I: testcontainers::Image>(
        container: &testcontainers::ContainerAsync<I>,
    ) -> Result<Self, PostgresError> {
        let host = container
            .get_host()
            .await
            .map_err(|e| PostgresError::HostResolutionFailed(Box::new(e)))?
            .to_string();

        let port = container
            .get_host_port_ipv4(5432)
            .await
            .map_err(|e| PostgresError::PortMappingFailed(Box::new(e)))?;

        let pool = PgPool::connect_lazy_with(
            PgConnectOptions::new()
                .username("postgres")
                .password("postgres")
                .host(&host)
                .port(port),
        );

        Ok(Self { pool })
    }

    async fn client(&self) -> Result<Self::Client, PostgresError> {
        Ok(self.pool.clone())
    }

    async fn healthy(&self) -> Result<(), PostgresError> {
        let _: i32 = sqlx::query_scalar("SELECT 1").fetch_one(&self.pool).await?;
        Ok(())
    }
}
