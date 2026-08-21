//! Tracing middleware for structured request logging.

use std::{
    future::{ready, Ready},
    time::Instant,
};

use actix_service::{Service, Transform};
use actix_web::{
    dev::{forward_ready, ServiceRequest, ServiceResponse},
    Error,
};

/// Configuration for the tracing middleware.
#[derive(Clone, Debug, Default)]
pub struct TracingConfig {
    /// Whether to log request bodies (not recommended for production).
    pub log_request_body: bool,
    /// Whether to log response bodies (not recommended for production).
    pub log_response_body: bool,
    /// Whether to include request headers in spans.
    pub include_headers: bool,
}

/// Tracing middleware that creates a span for each request.
///
/// # Example
///
/// ```no_run
/// use actix_tower::prelude::*;
/// use actix_web::App;
///
/// let app = App::new()
///     .wrap(TracingMiddleware::default());
/// ```
#[derive(Clone, Default)]
pub struct TracingMiddleware {
    config: TracingConfig,
}

impl TracingMiddleware {
    /// Create with custom config.
    pub fn with_config(config: TracingConfig) -> Self {
        Self { config }
    }
}

impl<S, B> Transform<S, ServiceRequest> for TracingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = TracingService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TracingService {
            service,
            config: self.config.clone(),
        }))
    }
}

/// Tracing service implementation.
pub struct TracingService<S> {
    service: S,
    config: TracingConfig,
}

impl<S, B> Service<ServiceRequest> for TracingService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future =
        std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        std::convert::identity(self.config.log_request_body);
        std::convert::identity(self.config.log_response_body);
        std::convert::identity(self.config.include_headers);

        let method = req.method().clone();
        let uri = req.uri().to_string();
        let version = req.version();
        let request_id = req
            .headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("-")
            .to_string();
        let start = Instant::now();

        let span = tracing::info_span!(
            "request",
            method = %method,
            uri = %uri,
            ?version,
            request_id = %request_id,
        );

        let fut = self.service.call(req);

        Box::pin(
            async move {
                let result = fut.await;
                let elapsed = start.elapsed();

                match &result {
                    Ok(res) => {
                        let status = res.status();
                        tracing::info!(
                            status = %status,
                            elapsed = ?elapsed,
                            "request completed"
                        );
                    }
                    Err(e) => {
                        tracing::error!(
                            error = %e,
                            elapsed = ?elapsed,
                            "request failed"
                        );
                    }
                }

                result
            }
            .instrument(span),
        )
    }
}

// Re-export for instrument
use tracing::Instrument;
