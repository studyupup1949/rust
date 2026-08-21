//! Extension traits for Actix types.

use actix_web::{dev::ServiceRequest, HttpMessage, HttpRequest};

/// Extension trait for `HttpRequest` with additional utility methods.
pub trait RequestExt {
    /// Get the request ID from extensions, if set.
    fn request_id(&self) -> Option<String>;

    /// Get a typed value from request extensions.
    fn get_extension<T: Clone + 'static>(&self) -> Option<T>;
}

impl RequestExt for HttpRequest {
    fn request_id(&self) -> Option<String> {
        self.extensions()
            .get::<crate::middleware::request_id::RequestIdExt>()
            .map(|ext| ext.0.clone())
    }

    fn get_extension<T: Clone + 'static>(&self) -> Option<T> {
        self.extensions().get::<T>().cloned()
    }
}

impl RequestExt for ServiceRequest {
    fn request_id(&self) -> Option<String> {
        self.extensions()
            .get::<crate::middleware::request_id::RequestIdExt>()
            .map(|ext| ext.0.clone())
    }

    fn get_extension<T: Clone + 'static>(&self) -> Option<T> {
        self.extensions().get::<T>().cloned()
    }
}
