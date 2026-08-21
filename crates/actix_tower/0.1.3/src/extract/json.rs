//! `AutoJson<T>` — ergonomic JSON extractor.

use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};

use actix_web::FromRequest;
use actix_web::{dev::Payload, web::Json, Error, HttpRequest};
use pin_project_lite::pin_project;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Ergonomic JSON extractor that automatically unwraps the inner value.
///
/// # Example
///
/// ```no_run
/// use actix_tower::prelude::*;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Deserialize, Serialize)]
/// struct User { name: String }
///
/// async fn create_user(AutoJson(user): AutoJson<User>) -> impl Responder {
///     // `user` is `User`, not `Json<User>`
///     HttpResponse::Ok().json(user)
/// }
/// ```
pub struct AutoJson<T>(pub T);

// ---- Smart constructors --------------------------------------------------

impl<T> AutoJson<T> {
    /// Create a new `AutoJson` wrapping the given value.
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// Consume and return the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

// ---- Deref / DerefMut ----------------------------------------------------

impl<T> Deref for AutoJson<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for AutoJson<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

// ---- Responder -----------------------------------------------------------

impl<T: Serialize> actix_web::Responder for AutoJson<T> {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        actix_web::HttpResponse::Ok().json(self.0)
    }
}

// ---- FromRequest ---------------------------------------------------------

pin_project! {
    /// Future for `AutoJson<T>`.
    pub struct AutoJsonFuture<F> {
        #[pin]
        inner: F,
    }
}

impl<T> FromRequest for AutoJson<T>
where
    T: DeserializeOwned + 'static,
{
    type Error = Error;
    type Future = AutoJsonFuture<<Json<T> as FromRequest>::Future>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        AutoJsonFuture {
            inner: Json::<T>::from_request(req, payload),
        }
    }
}

impl<F, T> Future for AutoJsonFuture<F>
where
    F: Future<Output = Result<Json<T>, Error>>,
{
    type Output = Result<AutoJson<T>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match std::task::ready!(this.inner.poll(cx)) {
            Ok(json) => Poll::Ready(Ok(AutoJson(json.into_inner()))),
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

// ---- Debug / Clone -------------------------------------------------------

impl<T: std::fmt::Debug> std::fmt::Debug for AutoJson<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AutoJson").field(&self.0).finish()
    }
}

impl<T: Clone> Clone for AutoJson<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
