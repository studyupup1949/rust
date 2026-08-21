//! `route!` macro for concise route definitions.

/// Define a route with method and path.
///
/// # Example
///
/// ```no_run
/// use actix_tower::prelude::*;
///
/// route!(get_user, GET, "/users/{id}");
/// ```
#[macro_export]
macro_rules! route {
    ($name:ident, GET, $path:expr) => {
        #[actix_web::get($path)]
        pub async fn $name() -> impl actix_web::Responder {
            actix_web::HttpResponse::Ok().finish()
        }
    };
    ($name:ident, POST, $path:expr) => {
        #[actix_web::post($path)]
        pub async fn $name() -> impl actix_web::Responder {
            actix_web::HttpResponse::Ok().finish()
        }
    };
    ($name:ident, PUT, $path:expr) => {
        #[actix_web::put($path)]
        pub async fn $name() -> impl actix_web::Responder {
            actix_web::HttpResponse::Ok().finish()
        }
    };
    ($name:ident, DELETE, $path:expr) => {
        #[actix_web::delete($path)]
        pub async fn $name() -> impl actix_web::Responder {
            actix_web::HttpResponse::Ok().finish()
        }
    };
    ($name:ident, PATCH, $path:expr) => {
        #[actix_web::patch($path)]
        pub async fn $name() -> impl actix_web::Responder {
            actix_web::HttpResponse::Ok().finish()
        }
    };
}

pub use route;
