//! Authentication and authorization middleware.

use std::future::{ready, Ready};

use actix_service::{Service, Transform};
use actix_web::{
    body::MessageBody,
    dev::{forward_ready, ServiceRequest, ServiceResponse},
    Error, HttpMessage, HttpRequest, HttpResponse,
};

/// Trait for extracting authentication information from a request.
pub trait AuthExtractor: Clone + Send + Sync + 'static {
    /// The principal type extracted from the request.
    type Principal: Clone + Send + Sync + 'static;

    /// Extract the principal from the request, if present.
    fn extract(&self, req: &HttpRequest) -> Option<Self::Principal>;
}

/// Authentication middleware that requires a valid principal.
///
/// # Example
///
/// ```no_run
/// use actix_tower::prelude::*;
/// use actix_web::{App, HttpRequest};
///
/// #[derive(Clone)]
/// struct BearerAuth;
///
/// impl AuthExtractor for BearerAuth {
///     type Principal = String;
///     fn extract(&self, req: &HttpRequest) -> Option<String> {
///         req.headers().get("authorization")
///             .and_then(|v| v.to_str().ok())
///             .and_then(|s| s.strip_prefix("Bearer "))
///             .map(|s| s.to_string())
///     }
/// }
///
/// let app = App::new()
///     .wrap(Authentication::new(BearerAuth));
/// ```
pub struct Authentication<E: AuthExtractor> {
    extractor: E,
}

impl<E: AuthExtractor> Authentication<E> {
    /// Create a new authentication middleware.
    pub fn new(extractor: E) -> Self {
        Self { extractor }
    }
}

impl<E: AuthExtractor> Clone for Authentication<E> {
    fn clone(&self) -> Self {
        Self {
            extractor: self.extractor.clone(),
        }
    }
}

impl<S, E, B> Transform<S, ServiceRequest> for Authentication<E>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    E: AuthExtractor,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = AuthMiddleware<S, E>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service,
            extractor: self.extractor.clone(),
        }))
    }
}

/// Authentication middleware service.
pub struct AuthMiddleware<S, E: AuthExtractor> {
    service: S,
    extractor: E,
}

impl<S, E, B> Service<ServiceRequest> for AuthMiddleware<S, E>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    E: AuthExtractor,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future =
        std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Extract principal
        if let Some(principal) = self.extractor.extract(req.request()) {
            // Store principal in extensions
            req.extensions_mut().insert(principal);
            let fut = self.service.call(req);
            Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_boxed_body())
            })
        } else {
            let (req, _) = req.into_parts();
            let response = HttpResponse::Unauthorized()
                .insert_header(("www-authenticate", "Bearer"))
                .body("Unauthorized");
            Box::pin(ready(Ok(ServiceResponse::new(req, response))))
        }
    }
}

/// Authorization middleware that checks permissions after authentication.
pub struct Authorization<F>
where
    F: Fn(&HttpRequest) -> bool + Clone + Send + Sync + 'static,
{
    check: F,
}

impl<F> Authorization<F>
where
    F: Fn(&HttpRequest) -> bool + Clone + Send + Sync + 'static,
{
    /// Create a new authorization middleware with a custom check function.
    pub fn new(check: F) -> Self {
        Self { check }
    }
}

impl<F> Clone for Authorization<F>
where
    F: Fn(&HttpRequest) -> bool + Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            check: self.check.clone(),
        }
    }
}

impl<S, F, B> Transform<S, ServiceRequest> for Authorization<F>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    F: Fn(&HttpRequest) -> bool + Clone + Send + Sync + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = AuthzMiddleware<S, F>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthzMiddleware {
            service,
            check: self.check.clone(),
        }))
    }
}

/// Authorization middleware service.
pub struct AuthzMiddleware<S, F>
where
    F: Fn(&HttpRequest) -> bool + Clone + Send + Sync + 'static,
{
    service: S,
    check: F,
}

impl<S, F, B> Service<ServiceRequest> for AuthzMiddleware<S, F>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    F: Fn(&HttpRequest) -> bool + Clone + Send + Sync + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future =
        std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if (self.check)(req.request()) {
            let fut = self.service.call(req);
            Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_boxed_body())
            })
        } else {
            let (req, _) = req.into_parts();
            let response = HttpResponse::Forbidden().body("Forbidden");
            Box::pin(ready(Ok(ServiceResponse::new(req, response))))
        }
    }
}
