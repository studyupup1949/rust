//! `handler!` macro for simplified handler definitions.

/// Define a handler with automatic extractor wrapping.
///
/// This macro creates an async function that accepts `AutoJson<T>` and
/// extracts the inner value automatically.
///
/// # Example
///
/// ```no_run
/// use actix_tower::prelude::*;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Deserialize, Serialize)]
/// struct CreateUser { name: String }
///
/// handler!(create_user, json: CreateUser => {
///     HttpResponse::Ok().json(json.into_inner())
/// });
/// ```
#[macro_export]
macro_rules! handler {
    ($name:ident, $extractor:ident : $ty:ty => $body:block) => {
        pub async fn $name(
            $extractor: $crate::extract::AutoJson<$ty>,
        ) -> impl actix_web::Responder {
            $body
        }
    };
    ($name:ident, $extractor:ident : $ty:ty => $body:expr) => {
        pub async fn $name(
            $extractor: $crate::extract::AutoJson<$ty>,
        ) -> impl actix_web::Responder {
            $body
        }
    };
}

pub use handler;
