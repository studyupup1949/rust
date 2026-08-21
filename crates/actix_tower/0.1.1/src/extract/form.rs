//! `AutoForm<T>` — ergonomic form extractor.

use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};

use actix_web::{dev::Payload, web::Form, Error, FromRequest, HttpRequest};
use pin_project_lite::pin_project;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Ergonomic form extractor that automatically unwraps the inner value.
///
/// # Example
///
/// ```no_run
/// use actix_tower::prelude::*;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Login { username: String, password: String }
///
/// async fn login(AutoForm(form): AutoForm<Login>) -> impl Responder {
///     HttpResponse::Ok().body(format!("Welcome {}", form.username))
/// }
/// ```
pub struct AutoForm<T>(pub T);

impl<T> AutoForm<T> {
    /// Create a new AutoForm instance.
    pub fn new(value: T) -> Self {
        Self(value)
    }
    /// Unwrap into the inner type.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for AutoForm<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for AutoForm<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: Serialize> actix_web::Responder for AutoForm<T> {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        match serde_urlencoded::to_string(&self.0) {
            Ok(body) => actix_web::HttpResponse::Ok()
                .content_type("application/x-www-form-urlencoded")
                .body(body),
            Err(_) => actix_web::HttpResponse::InternalServerError().finish(),
        }
    }
}

pin_project! {
    /// Future for AutoForm extraction.
    pub struct AutoFormFuture<F> {
        #[pin]
        inner: F,
    }
}

impl<T> FromRequest for AutoForm<T>
where
    T: DeserializeOwned + 'static,
{
    type Error = Error;
    type Future = AutoFormFuture<<Form<T> as FromRequest>::Future>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        AutoFormFuture {
            inner: Form::<T>::from_request(req, payload),
        }
    }
}

impl<F, T> Future for AutoFormFuture<F>
where
    F: Future<Output = Result<Form<T>, Error>>,
{
    type Output = Result<AutoForm<T>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match std::task::ready!(this.inner.poll(cx)) {
            Ok(form) => Poll::Ready(Ok(AutoForm(form.into_inner()))),
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for AutoForm<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AutoForm").field(&self.0).finish()
    }
}

impl<T: Clone> Clone for AutoForm<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
