//! Actix Permissions are extensions for permission and input validation for actix-web
//!
//! # Examples
//! ```no_run
//!
//! use actix_permissions::{check, with};
//! use actix_web::dev::*;
//! use actix_web::web::Data;
//! use actix_web::*;
//! use serde::Serialize;
//! use std::future::{ready, Ready};
//!
//! fn dummy_permission_check(
//!     req: &HttpRequest,
//!     _payload: &mut Payload,
//! ) -> Ready<actix_web::Result<bool, actix_web::Error>> {
//!     ready(Ok(true))
//! }
//!
//! async fn index() -> Result<String, Error> {
//!     Ok("Hi there!".to_string())
//! }
//!
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//!
//!    HttpServer::new(|| {
//!         App::new()
//!             .service(web::scope("").route(
//!                 "/",
//!                 check(
//!                     web::get(),
//!                     with(dummy_permission_check).and(dummy_permission_check),
//!                     index,
//!                 ),
//!             ))
//!     })
//!     .bind("127.0.0.1:8888")?
//!     .run()
//!     .await
//! }
//! ```
//!
mod builder;
mod tests;
pub mod permission;
pub(crate) mod service;

use crate::builder::Builder;
use crate::permission::Permission;
use crate::service::PermissionService;
use actix_web::dev::fn_factory;
use actix_web::{FromRequest, Handler, Responder, Route};
use std::future::ready;
use std::sync::Arc;

/// Creates a permission builder, initiated with single permission
///
/// # Arguments
/// * `permission` - permission
pub fn with<P>(permission: P) -> Builder
where
    P: Permission + 'static,
{
    Builder::new().and(permission)
}

/// Creates a permission builder, initiated with variable number of permissions

/// Creates a route which:
/// - intercepts requests and validates inputs
/// - if permission checks are all true, passes through to handler
pub fn check<F, Args>(route: Route, builder: Builder, handler: F) -> Route
where
    F: Handler<Args>,
    Args: FromRequest + 'static,
    F::Output: Responder,
{
    let new_perms = Arc::new(builder.permissions);
    route.service(fn_factory(move || {
        let new_perms_c = Arc::clone(&new_perms);
        let handler = handler.clone();
        ready(Ok(PermissionService::new(new_perms_c, handler)))
    }))
}
