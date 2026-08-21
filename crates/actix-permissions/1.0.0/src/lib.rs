//! Action Permissions Permission and input validation extension for Actix Web.
//! With access to app data injections, HttpRequest and Payload.
#![deny(missing_docs, rust_2018_idioms, elided_lifetimes_in_paths)]

use std::future::ready;

use actix_web::dev::fn_factory;
use actix_web::http::StatusCode;
use actix_web::{FromRequest, Handler, HttpRequest, HttpResponse, Responder, Route};

use crate::permission::Permission;
use crate::service::PermissionService;

pub mod permission;
pub mod service;
mod tests;

fn default_deny_handler(_req: HttpRequest) -> HttpResponse {
    HttpResponse::new(StatusCode::FORBIDDEN)
}

/// Creates a route which:
/// - intercepts requests and validates inputs.
/// - if permission check is true, passes through to handler.
/// - if permission check is false, `FORBIDDEN` is returned.
pub fn check<F, Args, P1, P1Args>(route: Route, permission: P1, handler: F) -> Route
where
    F: Handler<Args>,
    Args: FromRequest + 'static + Clone,
    P1: Permission<P1Args>,
    P1Args: FromRequest + 'static + Clone,
    F::Output: Responder,
{
    check_with_custom_deny(route, permission, handler, default_deny_handler)
}

/// Creates a more flexible route than `check`, which:
/// - intercepts requests and validates inputs.
/// - if permission checks are all true, passes through to handler.
/// - if any of the permissions is false, `deny_handler` is called.
pub fn check_with_custom_deny<F, Args, P1, P1Args>(
    route: Route,
    permission: P1,
    handler: F,
    deny_handler: fn(HttpRequest) -> HttpResponse,
) -> Route
where
    F: Handler<Args>,
    Args: FromRequest + 'static + Clone,
    P1: Permission<P1Args>,
    P1Args: FromRequest + 'static + Clone,
    F::Output: Responder,
{
    route.service(fn_factory(move || {
        ready(Ok(PermissionService::new(
            permission.clone(),
            handler.clone(),
            deny_handler,
        )))
    }))
}
