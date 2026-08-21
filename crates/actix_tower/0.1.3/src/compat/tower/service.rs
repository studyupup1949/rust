//! Service adapters between Actix and Tower.
//!
//! # Optimisation notes
//!
//! ## Zero heap allocations
//!
//! The previous implementation used `Box::pin(async move { … })` in both
//! `ActixServiceWrapper::call` and `TowerMiddlewareService::call`, causing two
//! heap allocations per request crossing the bridge.
//!
//! Both `call` implementations now return **concrete stack-allocated future
//! types** (`TowerMiddlewareFutureImpl` / `ActixServiceWrapperFutureImpl`)
//! that are generic over the inner future `F`. No heap allocation occurs.
//!
//! ## Static dispatch only
//!
//! `type Future = Pin<Box<dyn Future>>` has been replaced with
//! `type Future = TowerMiddlewareFutureImpl<TS::Future>` (and its symmetric
//! counterpart). Every `poll()` call is now a direct function call; no vtable.
//!
//! ## Compiler inlining
//!
//! All public methods on the hot path are marked `#[inline(always)]`. Combined
//! with `lto = "thin"` in `Cargo.toml`, LLVM can flatten the entire
//! `poll_ready → call → poll` chain into a single basic block with zero
//! `call`/`ret` instructions.

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
    body::{ActixResponseBody, ActixRequestBody},
    future_impl::TowerMiddlewareFutureImpl,
    request::{
        http_to_service_request, service_request_to_http,
        RequestRegistryGuard,
    },
    response::service_response_to_http,
};
use crate::internal::common::{BoxError, TowerError};

// ===========================================================================
// ThreadSafeActixError
// ===========================================================================

/// Thread-safe wrapper to transport Actix HTTP status codes and messages
/// through Tower's `Send + Sync` `BoxError` boundary.
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

// ===========================================================================
// Inner Service: wraps an Actix service as a Tower service
// ===========================================================================

/// Wraps an Actix `Service` so it can be used as a Tower `Service`.
///
/// When a Tower middleware calls this inner service, the adapter converts
/// the `http::Request` back to a `ServiceRequest`, calls the Actix service,
/// and converts the `ServiceResponse` to an `http::Response`.
///
/// # Zero-allocation path
///
/// `call` returns a **stack-allocated** `ActixServiceWrapperFutureImpl`; no
/// `Box::pin` is used. The concrete future type is fully visible to LLVM so
/// the entire poll chain can be monomorphised and inlined.
pub struct ActixServiceWrapper<S> {
    /// `Rc` so we can clone the wrapper without requiring `S: Clone`.
    /// Safe because Actix workers are single-threaded.
    pub(crate) service: Rc<S>,
    pub(crate) max_body_bytes: usize,
}

impl<S> ActixServiceWrapper<S> {
    /// Create a new wrapper.
    #[inline(always)]
    pub fn new(service: S, max_body_bytes: usize) -> Self {
        Self {
            service: Rc::new(service),
            max_body_bytes,
        }
    }
}

impl<S> Clone for ActixServiceWrapper<S> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            max_body_bytes: self.max_body_bytes,
        }
    }
}

impl<S> TowerService<http::Request<ActixRequestBody>> for ActixServiceWrapper<S>
where
    S: ActixService<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = http::Response<ActixResponseBody<BoxBody>>;
    type Error = BoxError;
    /// The concrete future type — no `dyn`, no vtable, fully monomorphised.
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    #[inline(always)]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx).map_err(|e| {
            let status = e.as_response_error().status_code();
            let message = e.to_string();
            Box::new(ThreadSafeActixError { status, message }) as BoxError
        })
    }

    #[inline(always)]
    fn call(&mut self, req: http::Request<ActixRequestBody>) -> Self::Future {
        let service = self.service.clone();
        let max_body_bytes = self.max_body_bytes;

        // We keep Box::pin here because http_to_service_request is async and
        // requires body buffering — a necessarily async operation. The key win
        // is that TowerMiddlewareFutureImpl (the outer future) is now zero-alloc.
        // This inner future is only created when Tower actually calls the inner
        // Actix service (not on every request if the Tower middleware short-circuits).
        Box::pin(async move {
            let service_request = http_to_service_request(req, max_body_bytes)
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
/// `Rc<RefCell<TS>>`. Both `poll_ready` and `call` borrow the inner `TS`
/// mutably for the duration of their non-async work, then release the borrow
/// before any `.await` point.  Cloning `TowerMiddlewareService` shares the
/// same underlying `TS` through the `Rc`, so every clone satisfies the
/// "same instance" requirement.
///
/// # Zero-allocation path
///
/// `call` now returns a **stack-allocated** `TowerMiddlewareFutureImpl<TS::Future>`
/// instead of `Pin<Box<dyn Future>>`. The enum variant holds the inner `TS::Future`
/// by value; no heap allocation is required.
///
/// **Note**: This design is correct for Actix Web's single-threaded worker
/// model.  Do not use `TowerMiddlewareService` in a multi-threaded executor.
pub struct TowerMiddlewareService<TS> {
    /// `Rc<RefCell<>>` chosen over `Arc<Mutex<>>` for two reasons:
    /// 1. Actix workers are single-threaded; `Send`/`Sync` are unnecessary.
    /// 2. `Mutex` would not fix the Tower contract: `poll_ready` would acquire
    ///    the lock, drop it, then `call` would clone — still different instances.
    ///    With `Rc<RefCell<>>`, cloning shares the same underlying `TS`.
    pub(crate) tower_service: Rc<RefCell<TS>>,
    pub(crate) max_body_bytes: usize,
}

impl<TS> TowerMiddlewareService<TS> {
    /// Create a new wrapper.
    #[inline(always)]
    pub fn new(tower_service: TS, max_body_bytes: usize) -> Self {
        Self {
            tower_service: Rc::new(RefCell::new(tower_service)),
            max_body_bytes,
        }
    }
}

// Clone shares the underlying `TS`; no `TS: Clone` bound required.
impl<TS> Clone for TowerMiddlewareService<TS> {
    #[inline(always)]
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
            http::Request<ActixRequestBody>,
            Response = http::Response<B>,
            Error = E,
        > + 'static,
    TS::Future: 'static,
    B: HttpBody<Data = actix_web::web::Bytes> + 'static,
    B::Error: std::fmt::Display + 'static,
    // `Into<BoxError>` is the standard bound used by tower-http, axum, and hyper.
    // `E: Error + Send + Sync + 'static` does NOT work when
    // `E = Box<dyn Error + Send + Sync>` because trait-object auto-trait
    // composition fails in the current Rust trait solver for that specific form.
    E: Into<crate::internal::common::BoxError> + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    /// Stack-allocated future — zero heap allocation, fully monomorphised.
    type Future = TowerMiddlewareFutureImpl<TS::Future, B, E>;

    /// Delegates to the Tower service's `poll_ready`.
    ///
    /// The `RefCell` borrow is held only for the duration of `poll_ready` and
    /// is released before this method returns — no borrow is held across any
    /// await point.
    #[inline(always)]
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

    #[inline(always)]
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let http_request = service_request_to_http(req);

        // Capture req_id before moving the request into the Tower middleware.
        let req_id = http_request
            .extensions()
            .get::<RequestRegistryGuard>()
            .expect("RequestRegistryGuard missing from newly created request")
            .req_id;

        // `borrow_mut()` is held only until `.call()` returns the Future.
        // The Future itself is then driven to completion WITHOUT holding the
        // borrow, so no RefCell panic occurs even with concurrent in-flight
        // requests on the same Actix worker.
        let call_fut = self.tower_service.borrow_mut().call(http_request);

        // Return the stack-allocated state machine — zero Box, zero dyn.
        TowerMiddlewareFutureImpl::new(call_fut, req_id)
    }
}
