//! Service adapters between Actix and Tower.

use std::{
    cell::RefCell,
    rc::Rc,
    task::{Context, Poll},
};

use actix_service::Service as ActixService;
use actix_web::{
    body::BoxBody,
    dev::{ServiceRequest, ServiceResponse},
    Error,
};
use http_body::Body as HttpBody;
use tower_service::Service as TowerService;

use crate::compat::tower::{
    body::ActixResponseBody,
    future::{ActixServiceWrapperFuture, TowerMiddlewareFuture},
    request::{http_to_service_request, service_request_to_http},
    response::{http_to_service_response, service_response_to_http},
};
use crate::internal::common::{BoxError, StringError, TowerError};

// ===========================================================================
// Inner Service: wraps an Actix service as a Tower service
// ===========================================================================

/// Wraps an Actix `Service` so it can be used as a Tower `Service`.
///
/// When a Tower middleware calls this inner service, the adapter converts
/// the `http::Request` back to a `ServiceRequest`, calls the Actix service,
/// and converts the `ServiceResponse` to an `http::Response`.
pub struct ActixServiceWrapper<S> {
    /// Rc so we can clone the wrapper without requiring `S: Clone`.
    /// Safe because Actix workers are single-threaded.
    pub(crate) service: Rc<S>,
    pub(crate) max_body_bytes: usize,
}

impl<S> ActixServiceWrapper<S> {
    /// Create a new wrapper.
    pub fn new(service: S, max_body_bytes: usize) -> Self {
        Self {
            service: Rc::new(service),
            max_body_bytes,
        }
    }
}

impl<S> Clone for ActixServiceWrapper<S> {
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            max_body_bytes: self.max_body_bytes,
        }
    }
}

/// Thread-safe wrapper to transport Actix HTTP status codes and messages
/// through Tower's Send+Sync BoxError boundary.
#[derive(Debug)]
pub struct ThreadSafeActixError {
    /// The HTTP status code of the original Actix error.
    pub status: actix_web::http::StatusCode,
    /// The original Actix error message.
    pub message: String,
}

impl std::fmt::Display for ThreadSafeActixError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ThreadSafeActixError {}

impl<S> TowerService<http::Request<crate::compat::tower::body::ActixRequestBody>>
    for ActixServiceWrapper<S>
where
    S: ActixService<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = http::Response<ActixResponseBody<BoxBody>>;
    type Error = BoxError;
    type Future = ActixServiceWrapperFuture;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx).map_err(|e| {
            let status = e.as_response_error().status_code();
            let message = e.to_string();
            Box::new(ThreadSafeActixError { status, message }) as BoxError
        })
    }

    fn call(
        &mut self,
        req: http::Request<crate::compat::tower::body::ActixRequestBody>,
    ) -> Self::Future {
        let service = self.service.clone();
        let max_body_bytes = self.max_body_bytes;

        Box::pin(async move {
            // Convert http::Request → ServiceRequest.
            // Returns Err if the RequestRegistryGuard is missing (Tower middleware
            // stripped extensions) or if the body exceeds max_body_bytes.
            let service_request =
                http_to_service_request(req, max_body_bytes)
                    .await
                    .map_err(|e| {
                        let status = e.as_response_error().status_code();
                        let message = e.to_string();
                        Box::new(ThreadSafeActixError { status, message }) as BoxError
                    })?;

            let service_response = service.call(service_request).await.map_err(|e| {
                let status = e.as_response_error().status_code();
                let message = e.to_string();
                Box::new(ThreadSafeActixError { status, message }) as BoxError
            })?;

            Ok(service_response_to_http(service_response))
        })
    }
}

// ===========================================================================
// Outer Service: wraps a Tower service as an Actix service
// ===========================================================================

/// Wraps a Tower `Service` so it can be used as an Actix `Service`.
///
/// # Tower `poll_ready` / `call` contract
///
/// Tower requires that `poll_ready` and `call` are invoked on the **same**
/// service instance.  This wrapper stores the Tower service in an
/// `Rc<RefCell<TS>>`.  Both `poll_ready` and `call` borrow the inner `TS`
/// mutably for the duration of their non-async work, then release the borrow
/// before any `.await` point.  Cloning `TowerMiddlewareService` shares the
/// same underlying `TS` through the `Rc`, so every clone satisfies the
/// "same instance" requirement.
///
/// **Note**: This design is correct for Actix Web's single-threaded worker
/// model.  Do not use `TowerMiddlewareService` in a multi-threaded executor.
pub struct TowerMiddlewareService<TS> {
    /// Rc<RefCell<>> chosen over Arc<Mutex<>> for two reasons:
    /// 1. Actix workers are single-threaded, so Send/Sync are unnecessary.
    /// 2. Mutex would not fix the Tower contract: poll_ready would acquire
    ///    the lock, drop it, then call() would clone — still different instances.
    ///    With Rc<RefCell<>>, cloning shares the same underlying TS.
    pub(crate) tower_service: Rc<RefCell<TS>>,
    pub(crate) max_body_bytes: usize,
}

impl<TS> TowerMiddlewareService<TS> {
    /// Create a new wrapper.
    pub fn new(tower_service: TS, max_body_bytes: usize) -> Self {
        Self {
            tower_service: Rc::new(RefCell::new(tower_service)),
            max_body_bytes,
        }
    }
}

// Clone shares the underlying TS; no TS: Clone bound required.
impl<TS> Clone for TowerMiddlewareService<TS> {
    fn clone(&self) -> Self {
        Self {
            tower_service: self.tower_service.clone(),
            max_body_bytes: self.max_body_bytes,
        }
    }
}

impl<TS, B, E> ActixService<ServiceRequest> for TowerMiddlewareService<TS>
where
    TS: TowerService<
            http::Request<crate::compat::tower::body::ActixRequestBody>,
            Response = http::Response<B>,
            Error = E,
        > + 'static,
    TS::Future: 'static,
    B: HttpBody<Data = actix_web::web::Bytes> + 'static,
    B::Error: std::fmt::Display + 'static,
    // Into<BoxError> is the standard bound used by tower-http, axum, and hyper.
    // E: Error + Send + Sync + 'static does NOT work when E = Box<dyn Error +
    // Send + Sync> because trait-object auto-trait composition fails in the
    // current Rust trait solver for that specific form.
    // Into<BoxError> is satisfied by all tower middleware error types and
    // preserves the full error without stringification.
    E: Into<crate::internal::common::BoxError> + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = TowerMiddlewareFuture;

    /// Delegates to the Tower service's `poll_ready`.
    ///
    /// The `RefCell` borrow is held only for the duration of `poll_ready` and
    /// is released before this method returns — no borrow is held across any
    /// await point.
    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.tower_service.borrow_mut().poll_ready(cx).map_err(|e| {
            let boxed: BoxError = e.into();
            match boxed.downcast::<ThreadSafeActixError>() {
                Ok(wrapped) => {
                    actix_web::error::InternalError::new(wrapped.message, wrapped.status).into()
                }
                Err(boxed) => Error::from(TowerError(boxed)),
            }
        })
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Clone the Rc — both this call site and the async block share the
        // same underlying TS, satisfying the Tower poll_ready/call contract.
        let tower_service = self.tower_service.clone();

        let http_request = service_request_to_http(req);
        
        // Capture req_id to ensure we can always recover the HttpRequest
        // even if the Tower middleware short-circuits and drops the http::Request.
        let req_id = http_request
            .extensions()
            .get::<crate::compat::tower::request::RequestRegistryGuard>()
            .expect("RequestRegistryGuard missing from newly created request")
            .req_id;

        Box::pin(async move {
            // `borrow_mut()` is held only until `.call()` returns the Future.
            // The Future itself is then driven to completion WITHOUT holding
            // the borrow, so no RefCell panic occurs even with concurrent
            // in-flight requests on the same Actix worker.
            let call_fut = tower_service.borrow_mut().call(http_request);

            let mut http_response = call_fut.await.map_err(|e| {
                // Convert to BoxError without stringifying, preserving the error chain.
                let boxed: BoxError = e.into();
                match boxed.downcast::<ThreadSafeActixError>() {
                    Ok(wrapped) => actix_web::error::InternalError::new(wrapped.message, wrapped.status).into(),
                    Err(boxed) => Error::from(TowerError(boxed)),
                }
            })?;

            // Retrieve the ResponseRegistryGuard if it exists
            let guard = http_response
                .extensions_mut()
                .remove::<crate::compat::tower::request::ResponseRegistryGuard>();

            if let Some(g) = guard {
                // If present, the inner Actix service was called.
                // Disarm the guard so it doesn't try to remove the entry again on drop.
                std::mem::forget(g);
            }

            // Recover the original Actix HttpRequest.
            // If the inner service was called, it is in RESPONSE_REGISTRY.
            // If the middleware short-circuited and dropped the request, RequestRegistryGuard::drop 
            // moved it to RESPONSE_REGISTRY.
            // If the middleware short-circuited but kept the request alive, it is in REQUEST_REGISTRY.
            let actix_req = crate::compat::tower::request::RESPONSE_REGISTRY
                .with(|registry| registry.borrow_mut().remove(&req_id))
                .or_else(|| {
                    crate::compat::tower::request::REQUEST_REGISTRY
                        .with(|registry| registry.borrow_mut().remove(&req_id))
                })
                .ok_or_else(|| {
                    Error::from(TowerError(Box::new(StringError(
                        "actix_tower: HttpRequest not found in registries. \
                         This is an internal bug; please file an issue."
                            .to_owned(),
                    )) as BoxError))
                })?;

            Ok(http_to_service_response(http_response, actix_req))
        })
    }
}
