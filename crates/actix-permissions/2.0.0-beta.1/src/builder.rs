//! [`Builder`] struct and function implementations.
use std::future::ready;
use std::marker::PhantomData;

use actix_web::http::StatusCode;
use actix_web::{FromRequest, Handler, HttpRequest, HttpResponse, Responder, Route};

use crate::{fn_factory, Permission, PermissionService};

/// Permission builder, combines route, handler and permission check, so
/// you can write something more fluent like:
/// ```
/// use actix_web::web;
/// use actix_web::HttpRequest;
/// use actix_permissions::builder::Builder;
///
/// async fn permission_check(_req: HttpRequest) -> actix_web::Result<bool>{
///     Ok(true)
/// }
/// async fn index() -> actix_web::Result<String> {
///     Ok("".to_string())
/// }
///
/// Builder::default().check(web::get()).is(permission_check).to(index);
/// ```
pub struct Builder<F, Args, P1, P1Args> {
    route: Option<Route>,
    permission: Option<P1>,
    handler: Option<F>,
    pd_handler: PhantomData<Args>,
    deny_handler: fn(HttpRequest) -> HttpResponse,
    pd_permission1: PhantomData<P1Args>,
}

impl<F, Args, P1, P1Args> Builder<F, Args, P1, P1Args>
where
    F: Handler<Args>,
    Args: FromRequest + 'static,
    P1: Permission<P1Args>,
    P1Args: FromRequest + 'static,
    F::Output: Responder,
{
    /// Appends route to builder.
    pub fn check<'a>(&'a mut self, route: Route) -> &'a mut Self {
        self.route = Some(route);
        self
    }

    /// Appends permission to builder.
    pub fn is<'a>(&'a mut self, permission: P1) -> &'a mut Self {
        self.permission = Some(permission);
        self
    }

    /// Appends deny handler to builder.
    pub fn when_denied<'a>(&'a mut self, handler: fn(HttpRequest) -> HttpResponse) -> &'a mut Self {
        self.deny_handler = handler;
        self
    }

    /// Appends handler to builder.
    pub fn to<'a>(&'a mut self, handler: F) -> &'a mut Self {
        self.handler = Some(handler);
        self
    }

    /// Builds a `Route` from permission `Builder` properties.
    /// `Route` that checks a `route` if passes a `permission`, then responds with `handler`.
    /// Otherwise a `deny_handler` is called.
    pub fn build<'a>(&mut self) -> Route {
        let permission = self.permission.take().unwrap();
        let handler = self.handler.take().unwrap();
        let deny_handler = self.deny_handler;

        self.route.take().unwrap().service(fn_factory(move || {
            ready(Ok(PermissionService::new(
                permission.clone(),
                handler.clone(),
                deny_handler,
            )))
        }))
    }
}

impl<F, Args, P1, P1Args> Default for Builder<F, Args, P1, P1Args> {
    fn default() -> Self {
        Self {
            handler: None,
            deny_handler: default_deny_handler,
            permission: None,
            pd_handler: PhantomData,
            pd_permission1: PhantomData,
            route: None,
        }
    }
}

/// Default deny handler, returns Forbidden.
pub fn default_deny_handler(_req: HttpRequest) -> HttpResponse {
    HttpResponse::new(StatusCode::FORBIDDEN)
}
