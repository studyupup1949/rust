//! `AutoPath<T>` — ergonomic path parameter extractor.

use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};

use actix_web::{web::Path, Error, FromRequest, HttpRequest};
use pin_project_lite::pin_project;
use serde::de::DeserializeOwned;

/// Ergonomic path extractor that automatically unwraps the inner value.
///
/// # Example
///
/// ```no_run
/// use actix_tower::prelude::*;
///
/// async fn get_user(AutoPath(id): AutoPath<u32>) -> impl Responder {
///     HttpResponse::Ok().body(format!("User {}", id))
/// }
/// ```
pub struct AutoPath<T>(pub T);

impl<T> AutoPath<T> {
    /// Create a new AutoPath instance.
    pub fn new(value: T) -> Self {
        Self(value)
    }
    /// Unwrap into the inner type.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for AutoPath<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for AutoPath<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

pin_project! {
    /// Future for AutoPath extraction.
    pub struct AutoPathFuture<F> {
        #[pin]
        inner: F,
    }
}

impl<T> FromRequest for AutoPath<T>
where
    T: DeserializeOwned + 'static,
{
    type Error = Error;
    type Future = AutoPathFuture<<Path<T> as FromRequest>::Future>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        AutoPathFuture {
            inner: Path::<T>::from_request(req, &mut actix_web::dev::Payload::None),
        }
    }
}

impl<F, T> Future for AutoPathFuture<F>
where
    F: Future<Output = Result<Path<T>, Error>>,
{
    type Output = Result<AutoPath<T>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match std::task::ready!(this.inner.poll(cx)) {
            Ok(p) => Poll::Ready(Ok(AutoPath(p.into_inner()))),
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for AutoPath<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AutoPath").field(&self.0).finish()
    }
}

impl<T: Clone> Clone for AutoPath<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
