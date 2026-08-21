//! `AutoMultipart<T>` — ergonomic multipart extractor.

use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;

use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use pin_project_lite::pin_project;
use serde::de::DeserializeOwned;

/// Ergonomic multipart extractor that automatically unwraps the inner value.
///
/// This is a placeholder that delegates to `actix_multipart` when available.
/// In the current version, it provides a structure for future multipart support.
pub struct AutoMultipart<T>(pub T);

impl<T> AutoMultipart<T> {
    /// Create a new AutoMultipart instance.
    pub fn new(value: T) -> Self {
        Self(value)
    }
    /// Unwrap into the inner type.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for AutoMultipart<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for AutoMultipart<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

pin_project! {
    /// Future for AutoMultipart extraction.
    pub struct AutoMultipartFuture<F> {
        #[pin]
        inner: F,
    }
}

/// Placeholder implementation — full multipart support requires
/// `actix-multipart` integration.
impl<T: DeserializeOwned + 'static> FromRequest for AutoMultipart<T> {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        std::convert::identity(req);
        std::convert::identity(payload);
        Box::pin(async move {
            Err(actix_web::error::ErrorPreconditionFailed(
                "AutoMultipart is not yet implemented. Use actix-multipart directly.",
            ))
        })
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for AutoMultipart<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AutoMultipart").field(&self.0).finish()
    }
}
