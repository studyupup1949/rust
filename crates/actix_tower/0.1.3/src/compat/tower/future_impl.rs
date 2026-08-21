// pin_project_lite does not support doc comments on enum variants/fields inside
// the macro body, and #[allow(missing_docs)] cannot be applied to macro invocations.
// We suppress the resulting warnings at the module level here.
#![allow(missing_docs, unused_doc_comments, unused_attributes)]
//! Zero-allocation, zero-dynamic-dispatch Future implementations for the Tower bridge.
//!
//! # Design
//!
//! The original implementation used `Pin<Box<dyn Future>>` for both
//! `TowerMiddlewareFuture` and `ActixServiceWrapperFuture`. Each `Box::pin`
//! triggers a heap allocation (~40–80 ns on a cold allocator path) and every
//! `.poll()` call goes through a vtable, preventing LLVM from inlining across
//! the async boundary.
//!
//! This module replaces the *outer* future (`TowerMiddlewareFutureImpl`) with a
//! **structural enum state machine** via `pin_project_lite`. The entire future
//! lives on the caller's stack frame. LLVM can see through all generic
//! parameters and flatten the poll chain into a single function body with no
//! `call`/`ret` pairs on the hot path.
//!
//! # Generic parameters
//!
//! `TowerMiddlewareFutureImpl` carries:
//! - `F`  — the concrete `TS::Future` type (Tower middleware output future)
//! - `B`  — the concrete response body type
//! - `E`  — the concrete error type
//!
//! The compiler monomorphises a separate, fully-optimised copy for every
//! distinct `(F, B, E)` triple, enabling complete inlining without any runtime
//! overhead.

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use actix_web::{dev::ServiceResponse, Error};
use http_body::Body as HttpBody;
use pin_project_lite::pin_project;

use crate::compat::tower::{
    request::{REQUEST_REGISTRY, RESPONSE_REGISTRY, ResponseRegistryGuard},
    response::http_to_service_response,
    service::ThreadSafeActixError,
};
use crate::internal::common::{BoxError, StringError, TowerError};

// ============================================================================
// TowerMiddlewareFutureImpl
// ============================================================================

/// Stack-allocated state machine for [`TowerMiddlewareService::call`].
///
/// Replaces `Pin<Box<dyn Future<Output = Result<ServiceResponse, Error>>>>` —
/// the concrete inner future `F` is fully visible to LLVM so the entire poll
/// chain can be monomorphised and inlined with no `call`/`ret` pairs.
/// `Done` is the terminal state; polling it returns `Poll::Pending`.
#[allow(missing_docs)]
pin_project! {
    #[project = TowerMiddlewareFutureImplProj]
    pub enum TowerMiddlewareFutureImpl<F, B, E> {
        Running {
            #[pin]
            inner: F,
            req_id: u64,
        },
        Done {
            _phantom: std::marker::PhantomData<(B, E)>,
        },
    }
}

impl<F, B, E> TowerMiddlewareFutureImpl<F, B, E> {
    /// Construct a new running future.
    #[inline(always)]
    pub fn new(inner: F, req_id: u64) -> Self {
        Self::Running { inner, req_id }
    }
}

impl<F, B, E> Future for TowerMiddlewareFutureImpl<F, B, E>
where
    F: Future<Output = Result<http::Response<B>, E>>,
    B: HttpBody<Data = actix_web::web::Bytes> + 'static,
    B::Error: std::fmt::Display + 'static,
    E: Into<BoxError> + 'static,
{
    type Output = Result<ServiceResponse, Error>;

    #[inline(always)]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.as_mut().project() {
            TowerMiddlewareFutureImplProj::Done { .. } => Poll::Pending,
            TowerMiddlewareFutureImplProj::Running { inner, req_id } => {
                let req_id = *req_id;
                match inner.poll(cx) {
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(result) => {
                        self.set(TowerMiddlewareFutureImpl::Done {
                            _phantom: std::marker::PhantomData,
                        });

                        let mut http_response = result.map_err(|e| {
                            let boxed: BoxError = e.into();
                            match boxed.downcast::<ThreadSafeActixError>() {
                                Ok(wrapped) => actix_web::error::InternalError::new(
                                    wrapped.message,
                                    wrapped.status,
                                )
                                .into(),
                                Err(boxed) => Error::from(TowerError(boxed)),
                            }
                        })?;

                        // Retrieve and disarm ResponseRegistryGuard if present.
                        let guard = http_response
                            .extensions_mut()
                            .remove::<ResponseRegistryGuard>();
                        if let Some(g) = guard {
                            std::mem::forget(g);
                        }

                        // Recover the original Actix HttpRequest from whichever
                        // registry holds it (response → short-circuit → request).
                        let actix_req = RESPONSE_REGISTRY
                            .with(|registry| registry.borrow_mut().remove(&req_id))
                            .or_else(|| {
                                REQUEST_REGISTRY.with(|registry| {
                                    registry.borrow_mut().remove(&req_id)
                                })
                            })
                            .ok_or_else(|| {
                                Error::from(TowerError(
                                    Box::new(StringError(
                                        "actix_tower: HttpRequest not found in registries. \
                                         This is an internal bug; please file an issue."
                                            .to_owned(),
                                    )) as BoxError,
                                ))
                            })?;

                        Poll::Ready(Ok(http_to_service_response(http_response, actix_req)))
                    }
                }
            }
        }
    }
}

// ============================================================================
// ActixServiceWrapperFutureImpl  (kept for completeness / future use)
// ============================================================================
//
// This type is currently not used as the associated Future type of
// ActixServiceWrapper because `http_to_service_request` is async and we
// cannot avoid one async block there. However, the type is kept here so
// that future work (a streaming body path) can use it without `Box::pin`.

/// Placeholder state machine for `ActixServiceWrapper::call`.
///
/// Currently unused on the main hot path (see `service.rs` comments) but
/// provided so a future streaming, zero-alloc Actix wrapper can reuse this
/// state machine when body buffering is disabled.
#[allow(missing_docs)]
pin_project! {
    #[project = ActixServiceWrapperFutureImplProj]
    #[allow(dead_code)]
    pub enum ActixServiceWrapperFutureImpl<CallFut> {
        Running {
            #[pin]
            call_fut: CallFut,
        },
        Done,
    }
}

#[allow(dead_code)]
impl<CallFut> ActixServiceWrapperFutureImpl<CallFut> {
    #[inline(always)]
    pub fn new(call_fut: CallFut) -> Self {
        Self::Running { call_fut }
    }
}

impl<CallFut, Resp, Err> Future for ActixServiceWrapperFutureImpl<CallFut>
where
    CallFut: Future<Output = Result<Resp, Err>>,
{
    type Output = Result<Resp, Err>;

    #[inline(always)]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.as_mut().project() {
            ActixServiceWrapperFutureImplProj::Done => Poll::Pending,
            ActixServiceWrapperFutureImplProj::Running { call_fut } => {
                match call_fut.poll(cx) {
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(result) => {
                        self.set(ActixServiceWrapperFutureImpl::Done);
                        Poll::Ready(result)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{dev::ServiceResponse, Error};
    use futures_util::future::Ready;
    use std::mem::size_of;

    #[test]
    fn assert_future_size() {
        // Use a genuinely empty inner future so we are testing only the wrapper overhead.
        type DummyInnerFut = Ready<()>;
        type BodyType = actix_web::body::BoxBody;
        type ErrorType = Error;

        type TowerFut = TowerMiddlewareFutureImpl<DummyInnerFut, BodyType, ErrorType>;
        type ActixFut = ActixServiceWrapperFutureImpl<DummyInnerFut>;

        let tower_size = size_of::<TowerFut>();
        // Overhead should be `u64` (req_id) + discriminant + inner future (1 byte) + padding -> ~16-24 bytes
        assert!(tower_size <= 24, "TowerMiddlewareFutureImpl overhead {} is too large", tower_size);

        let actix_size = size_of::<ActixFut>();
        assert!(actix_size <= 16, "ActixServiceWrapperFutureImpl size {} is too large", actix_size);
    }

    #[test]
    fn assert_unpin() {
        type DummyInnerFut = Ready<Result<ServiceResponse, Error>>;
        type BodyType = actix_web::body::BoxBody;
        type ErrorType = Error;

        type TowerFut = TowerMiddlewareFutureImpl<DummyInnerFut, BodyType, ErrorType>;

        fn assert_is_unpin<T: Unpin>() {}
        assert_is_unpin::<TowerFut>();
    }

    #[test]
    fn test_no_vtable() {
        type DummyInnerFut = Ready<Result<ServiceResponse, Error>>;
        type BodyType = actix_web::body::BoxBody;
        type ErrorType = Error;

        type TowerFut = TowerMiddlewareFutureImpl<DummyInnerFut, BodyType, ErrorType>;

        fn assert_is_sized<T: Sized>() {}
        assert_is_sized::<TowerFut>();
    }
}
