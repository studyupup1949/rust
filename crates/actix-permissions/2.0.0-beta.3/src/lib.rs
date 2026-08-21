//! Action Permissions Permission and input validation extension for Actix Web.
//! With access to app data injections, HttpRequest and Payload.
#![deny(missing_docs, rust_2018_idioms, elided_lifetimes_in_paths)]

use actix_web::dev::fn_factory;
use actix_web::{FromRequest, Handler, HttpRequest, HttpResponse, Responder, Route};

use crate::builder::Builder;
use crate::permission::Permission;
use crate::service::PermissionService;

pub mod builder;
pub mod permission;
pub mod service;
mod tests;

/// Shorthand for instantiating permissions [`Builder`]
/// ```
/// use actix_web::web;
/// use actix_web::HttpRequest;
/// use actix_permissions::permission;
///
/// async fn permission_check(_req: HttpRequest)->actix_web::Result<bool>{
///     Ok(true)
/// }
/// async fn index() -> actix_web::Result<String> {
///     Ok("".to_string())
/// }
/// permission().check(web::get()).validate(permission_check).to(index).build();
/// ```
pub fn permission<F, Args, P1, P1Args>() -> Builder<F, Args, P1, P1Args> {
    Builder::default()
}

/// Creates a route which:
/// - intercepts requests and validates inputs.
/// - if permission check is true, passes through to handler.
/// - if permission check is false, `FORBIDDEN` is returned.
#[deprecated(since = "2.0.0-beta.1", note = "please use `permission()` instead")]
pub fn check<F, Args, P1, P1Args>(route: Route, perm: P1, handler: F) -> Route
where
    F: Handler<Args>,
    Args: FromRequest + 'static,
    P1: Permission<P1Args>,
    P1Args: FromRequest + 'static,
    F::Output: Responder,
{
    permission().check(route).validate(perm).to(handler).build()
}

/// Creates a more flexible route than `check`, which:
/// - intercepts requests and validates inputs.
/// - if permission checks are all true, passes through to handler.
/// - if any of the permissions is false, `deny_handler` is called.
#[deprecated(since = "2.0.0-beta.1", note = "please use `permission()` instead")]
pub fn check_with_custom_deny<F, Args, P1, P1Args>(
    route: Route,
    perm: P1,
    handler: F,
    deny_handler: fn(HttpRequest) -> HttpResponse,
) -> Route
where
    F: Handler<Args>,
    Args: FromRequest + 'static,
    P1: Permission<P1Args>,
    P1Args: FromRequest + 'static,
    F::Output: Responder,
{
    permission()
        .with_deny_handler(deny_handler)
        .check(route)
        .validate(perm)
        .to(handler)
        .build()
}
