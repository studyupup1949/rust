//! Allow clean configuration of Actix Web routes.

use actix_web::web::ServiceConfig;

/// Trait indicating that the object can route web requests.
pub trait Routable {
    /// Configure the service with the information from the this router.
    fn configure(_config: &mut ServiceConfig) {}

    /// Configure the service with access to `self`.
    fn configure_non_static(&self, _config: &mut ServiceConfig) {}
}