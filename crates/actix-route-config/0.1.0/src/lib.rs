//! Allow clean configuration of Actix Web routes.

use actix_web::web::ServiceConfig;

/// Trait indicating that the object can route web requests.
pub trait Routable {
    /// Configure the service with the information from the this router.
    fn configure(config: &mut ServiceConfig);
}