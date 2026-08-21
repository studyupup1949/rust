//! `AutoData<T>` — ergonomic app data extractor.

use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::task::{Context, Poll};

use actix_web::{web::Data, Error, FromRequest, HttpRequest};
use pin_project_lite::pin_project;

/// Ergonomic data extractor that automatically unwraps the inner value.
pub struct AutoData<T>(pub T);

impl<T> AutoData<T> {
    /// Create a new AutoData instance.
    pub fn new(value: T) -> Self {
        Self(value)
    }
    /// Unwrap into the inner type.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for AutoData<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

pin_project! {
    /// Future for AutoData extraction.
    pub struct AutoDataFuture<F> {
        #[pin]
        inner: F,
    }
}

impl<T: Clone + 'static> FromRequest for AutoData<T> {
    type Error = Error;
    type Future = AutoDataFuture<<Data<T> as FromRequest>::Future>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        AutoDataFuture {
            inner: Data::<T>::from_request(req, &mut actix_web::dev::Payload::None),
        }
    }
}

impl<F, T> Future for AutoDataFuture<F>
where
    F: Future<Output = Result<Data<T>, Error>>,
    T: Clone,
{
    type Output = Result<AutoData<T>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match std::task::ready!(this.inner.poll(cx)) {
            Ok(data) => Poll::Ready(Ok(AutoData(data.get_ref().clone()))),
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for AutoData<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AutoData").field(&self.0).finish()
    }
}

impl<T: Clone> Clone for AutoData<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
