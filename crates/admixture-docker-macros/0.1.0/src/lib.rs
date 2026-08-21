//! Procedural macros for Admixture Docker services.
//!
//! This crate provides the `docker_service!` macro for declaratively defining Docker container services.

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod docker_service;

/// Declares a Docker container service.
///
/// # Example
///
/// ```ignore
/// use admixture_docker::docker_service;
/// use sqlx::PgPool;
/// use testcontainers_modules::postgres::Postgres;
///
/// docker_service! {
///     SqlxPostgres {
///         image: Postgres,
///         error: PostgresError,
///         client: PgPool,
///         
///         context {
///             pool: PgPool,
///         }
///         
///         async fn construct<I: testcontainers::Image>(
///             container: &testcontainers::ContainerAsync<I>
///         ) -> Result<Self, PostgresError> {
///             // Container setup logic
///             let host = container.get_host().await?.to_string();
///             let port = container.get_host_port_ipv4(5432).await?;
///             let pool = PgPool::connect_lazy_with(/* ... */);
///             Ok(Self { pool })
///         }
///         
///         async fn client(&self) -> Result<PgPool, PostgresError> {
///             Ok(self.pool.clone())
///         }
///         
///         async fn healthy(&self) -> Result<(), PostgresError> {
///             sqlx::query_scalar("SELECT 1").fetch_one(&self.pool).await?;
///             Ok(())
///         }
///     }
/// }
/// ```
///
/// This generates:
/// - A type alias `SqlxPostgresServiceSetup = DockerContainerServiceSetup<Postgres, SqlxPostgresContext>`
/// - A context struct `SqlxPostgresContext` with the specified fields
/// - An implementation of `ContainerService` for the context
#[proc_macro]
pub fn docker_service(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as docker_service::DockerServiceMacroInput);
    docker_service::generate(input).into()
}
