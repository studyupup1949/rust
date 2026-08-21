//! `AutoState<T>` — ergonomic state extractor (same as `AutoData` but for `web::ServiceConfig` state).

use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::task::{Context, Poll};

use actix_web::{web::Data, Error, FromRequest, HttpRequest};
use pin_project_lite::pin_project;

/// Ergonomic state extractor — alias for `AutoData<T>`.
///
/// Use this when you want to emphasize that the data is application state
/// rather than injected data.
///
/// # Example
///
/// ```no_run
/// use actix_tower::prelude::*;
/// use std::sync::Arc;
///
/// struct AppState { db_url: String }
///
/// async fn handler(AutoState(state): AutoState<Arc<AppState>>) -> impl Responder {
///     HttpResponse::Ok().body(state.db_url.clone())
/// }
/// ```
pub struct AutoState<T>(pub T);

impl<T> AutoState<T> {
    /// Create a new AutoState instance.
    pub fn new(value: T) -> Self {
        Self(value)
    }
    /// Unwrap into the inner type.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for AutoState<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

pin_project! {
    /// Future for AutoState extraction.
    pub struct AutoStateFuture<F> {
        #[pin]
        inner: F,
    }
}

impl<T: Clone + 'static> FromRequest for AutoState<T> {
    type Error = Error;
    type Future = AutoStateFuture<<Data<T> as FromRequest>::Future>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        AutoStateFuture {
            inner: Data::<T>::from_request(req, &mut actix_web::dev::Payload::None),
        }
    }
}

impl<F, T> Future for AutoStateFuture<F>
where
    F: Future<Output = Result<Data<T>, Error>>,
    T: Clone,
{
    type Output = Result<AutoState<T>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match std::task::ready!(this.inner.poll(cx)) {
            Ok(data) => Poll::Ready(Ok(AutoState(data.get_ref().clone()))),
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for AutoState<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AutoState").field(&self.0).finish()
    }
}

impl<T: Clone> Clone for AutoState<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
