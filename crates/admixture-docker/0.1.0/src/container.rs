use std::future::Future;
use testcontainers::ContainerAsync;

/// Trait for services running inside Docker containers.
///
/// This trait provides a bridge between testcontainers and Admixture's
/// service abstraction, allowing container-based services to provide
/// typed clients and health checks.
pub trait ContainerService: Sized {
    type Client;
    type Error: std::error::Error + Send + Sync + 'static;

    fn construct<I: testcontainers::Image>(
        container: &ContainerAsync<I>,
    ) -> impl Future<Output = Result<Self, Self::Error>>;

    fn client(&self) -> impl Future<Output = Result<Self::Client, Self::Error>>;

    fn healthy(&self) -> impl Future<Output = Result<(), Self::Error>>;
}
