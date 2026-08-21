//! Docker container service abstraction for Admixture
//!
//! This crate provides the `ContainerService` trait and implementations
//! for running services inside Docker containers via testcontainers-rs.
//!
//! ## Features
//!
//! - `sqlx-postgres`: PostgreSQL containers with sqlx PgPool support
//! - `fred-redis`: Redis containers with fred client support

pub mod container;
pub mod error;
pub mod service;

#[cfg(feature = "sqlx-postgres")]
pub mod sqlx_postgres;

#[cfg(feature = "fred-redis")]
pub mod fred_redis;

pub use container::ContainerService;
pub use error::DockerError;
pub use service::{DockerContainerServiceRunning, DockerContainerServiceSetup};

#[cfg(feature = "sqlx-postgres")]
pub use sqlx_postgres::{PostgresError, SqlxPostgresContext, SqlxPostgresServiceSetup};

#[cfg(feature = "fred-redis")]
pub use fred_redis::{FredRedisContext, FredRedisServiceSetup, RedisError};

// Re-export the docker_service! macro
pub use admixture_docker_macros::docker_service;
