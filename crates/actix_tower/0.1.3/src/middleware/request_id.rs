//! Request ID middleware — generates a unique ID for each request.

use std::{
    fmt,
    future::{ready, Ready},
    task::{Context, Poll},
};

use actix_service::{Service, Transform};
use actix_web::{
    dev::{forward_ready, ServiceRequest, ServiceResponse},
    http::header,
    Error, HttpMessage,
};
use uuid::Uuid;

/// Header name for the request ID.
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Middleware that generates a unique request ID for each incoming request.
///
/// If the client provides an `x-request-id` header, it is used.
/// Otherwise, a new UUID v4 is generated.
///
/// # Example
///
/// ```no_run
/// use actix_tower::prelude::*;
/// use actix_web::App;
///
/// let app = App::new()
///     .wrap(RequestId::new());
/// ```
#[derive(Clone, Debug)]
pub struct RequestId;

impl RequestId {
    /// Create a new `RequestId` middleware.
    pub fn new() -> Self {
        Self
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequestId
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestIdMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestIdMiddleware { service }))
    }
}

/// The actual middleware service.
pub struct RequestIdMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestIdMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = RequestIdFuture<S::Future, B>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        // Check if the client already provided a request ID
        let request_id = req
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        // Set the request ID header
        if let Ok(value) = header::HeaderValue::from_str(&request_id) {
            req.headers_mut()
                .insert(header::HeaderName::from_static(REQUEST_ID_HEADER), value);
        }

        // Store in extensions for access in handlers
        req.extensions_mut().insert(RequestIdExt(request_id));

        RequestIdFuture {
            fut: self.service.call(req),
            _phantom: std::marker::PhantomData,
        }
    }
}

/// Extension that stores the request ID in request extensions.
#[derive(Clone, Debug)]
pub struct RequestIdExt(pub String);

impl RequestIdExt {
    /// Get the request ID as a string reference.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RequestIdExt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

pin_project_lite::pin_project! {
    /// Future for `RequestIdMiddleware`.
    pub struct RequestIdFuture<F, B> {
        #[pin]
        fut: F,
        _phantom: std::marker::PhantomData<B>,
    }
}

impl<F, B, E> std::future::Future for RequestIdFuture<F, B>
where
    F: std::future::Future<Output = Result<ServiceResponse<B>, E>>,
{
    type Output = Result<ServiceResponse<B>, E>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let mut res = std::task::ready!(this.fut.poll(cx))?;

        // Add request ID to response headers
        let req_id = res
            .request()
            .extensions()
            .get::<RequestIdExt>()
            .map(|ext| ext.0.clone());
        if let Some(req_id) = req_id {
            if let Ok(value) = header::HeaderValue::from_str(&req_id) {
                res.headers_mut()
                    .insert(header::HeaderName::from_static(REQUEST_ID_HEADER), value);
            }
        }

        Poll::Ready(Ok(res))
    }
}
