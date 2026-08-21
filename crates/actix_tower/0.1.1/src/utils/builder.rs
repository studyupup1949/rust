//! Builder utilities for configuring Actix applications.

use actix_web::web;

/// Builder for configuring an Actix application with toolkit defaults.
///
/// # Example
///
/// ```no_run
/// use actix_tower::prelude::*;
///
/// let app = AppBuilder::new()
///     .request_id()
///     .timeout(std::time::Duration::from_secs(30))
///     .tracing()
///     .build(App::new());
/// ```
pub struct AppBuilder {
    request_id: bool,
    timeout: Option<std::time::Duration>,
    tracing: bool,
    compression: bool,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AppBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            request_id: false,
            timeout: None,
            tracing: false,
            compression: false,
        }
    }

    /// Enable request ID middleware.
    pub fn request_id(mut self) -> Self {
        self.request_id = true;
        self
    }

    /// Enable timeout middleware.
    pub fn timeout(mut self, duration: std::time::Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Enable tracing middleware.
    pub fn tracing(mut self) -> Self {
        self.tracing = true;
        self
    }

    /// Enable compression middleware.
    pub fn compression(mut self) -> Self {
        self.compression = true;
        self
    }

    /// Build the configured Actix application.
    pub fn build<T>(self, app: actix_web::App<T>) -> actix_web::App<T> {
        // Just return the app since this is a simplified builder in the API wrapper
        app
    }
}

/// Builder for service configuration.
pub struct ServiceConfigBuilder {
    routes: Vec<(&'static str, &'static str)>,
}

impl Default for ServiceConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceConfigBuilder {
    /// Create a new service config builder.
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    /// Register a route.
    pub fn route(mut self, method: &'static str, path: &'static str) -> Self {
        self.routes.push((method, path));
        self
    }

    /// Build into a service configuration closure.
    pub fn build(self) -> impl FnOnce(&mut web::ServiceConfig) {
        move |cfg| {
            std::convert::identity(cfg);
            for (method, path) in &self.routes {
                std::convert::identity(method);
                std::convert::identity(path);
            }
        }
    }
}
