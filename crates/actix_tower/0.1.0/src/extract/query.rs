//! `AutoQuery<T>` — ergonomic query string extractor.

use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};

use actix_web::{web::Query, Error, FromRequest, HttpRequest};
use pin_project_lite::pin_project;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Ergonomic query extractor that automatically unwraps the inner value.
///
/// # Example
///
/// ```no_run
/// use actix_tower::prelude::*;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Pagination { page: u32, per_page: u32 }
///
/// async fn list(AutoQuery(pagination): AutoQuery<Pagination>) -> impl Responder {
///     HttpResponse::Ok().body(format!("Page {}", pagination.page))
/// }
/// ```
pub struct AutoQuery<T>(pub T);

impl<T> AutoQuery<T> {
    /// Create a new AutoQuery instance.
    pub fn new(value: T) -> Self {
        Self(value)
    }
    /// Unwrap into the inner type.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for AutoQuery<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for AutoQuery<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: Serialize> actix_web::Responder for AutoQuery<T> {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        match serde_urlencoded::to_string(&self.0) {
            Ok(qs) => actix_web::HttpResponse::Ok().body(qs),
            Err(_) => actix_web::HttpResponse::InternalServerError().finish(),
        }
    }
}

pin_project! {
    /// Future for AutoQuery extraction.
    pub struct AutoQueryFuture<F> {
        #[pin]
        inner: F,
    }
}

impl<T> FromRequest for AutoQuery<T>
where
    T: DeserializeOwned + 'static,
{
    type Error = Error;
    type Future = AutoQueryFuture<<Query<T> as FromRequest>::Future>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        AutoQueryFuture {
            inner: Query::<T>::from_request(req, &mut actix_web::dev::Payload::None),
        }
    }
}

impl<F, T> Future for AutoQueryFuture<F>
where
    F: Future<Output = Result<Query<T>, Error>>,
{
    type Output = Result<AutoQuery<T>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match std::task::ready!(this.inner.poll(cx)) {
            Ok(q) => Poll::Ready(Ok(AutoQuery(q.into_inner()))),
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for AutoQuery<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AutoQuery").field(&self.0).finish()
    }
}

impl<T: Clone> Clone for AutoQuery<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
