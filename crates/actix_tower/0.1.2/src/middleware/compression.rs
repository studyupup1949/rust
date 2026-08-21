//! Response compression middleware (native Actix implementation).

use std::future::{ready, Ready};

use actix_service::{Service, Transform};
use actix_web::{
    body::MessageBody,
    dev::{forward_ready, ServiceRequest, ServiceResponse},
    http::header,
    Error, HttpResponse,
};

/// Compression middleware that compresses responses based on `Accept-Encoding`.
///
/// This is a simplified native implementation. For full-featured compression,
/// use `tower-http`'s `CompressionLayer` via the Tower bridge:
///
/// ```no_run
/// use actix_tower::prelude::*;
/// use actix_web::App;
///
/// let app = App::new()
///     .wrap(tower_layer!(tower_http::compression::CompressionLayer::new()));
/// ```
#[derive(Clone, Debug)]
pub struct Compression {
    /// Whether to enable gzip.
    pub gzip: bool,
}

impl Default for Compression {
    fn default() -> Self {
        Self { gzip: true }
    }
}

impl Compression {
    /// Create a new compression middleware with gzip enabled.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable gzip.
    pub fn gzip(mut self, enable: bool) -> Self {
        self.gzip = enable;
        self
    }
}

impl<S, B> Transform<S, ServiceRequest> for Compression
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = CompressionMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CompressionMiddleware {
            service,
            gzip: self.gzip,
        }))
    }
}

/// Compression middleware service.
pub struct CompressionMiddleware<S> {
    service: S,
    gzip: bool,
}

impl<S, B> Service<ServiceRequest> for CompressionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future =
        std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Check Accept-Encoding
        let accept_encoding = req
            .headers()
            .get(header::ACCEPT_ENCODING)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let supports_gzip = self.gzip && accept_encoding.contains("gzip");

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            if supports_gzip {
                // Check content type — only compress text-based responses
                let content_type = res
                    .headers()
                    .get(header::CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_string();

                let should_compress = content_type.starts_with("text/")
                    || content_type.contains("json")
                    || content_type.contains("xml")
                    || content_type.contains("javascript");

                if should_compress {
                    // Read the body
                    let (req, response) = res.into_parts();
                    let status = response.status();
                    let body = response.into_body();
                    let bytes = actix_web::body::to_bytes(body)
                        .await
                        .map_err(|e| actix_web::error::ErrorInternalServerError(e.into()))?;

                    // Compress with gzip
                    use std::io::Write;
                    let mut encoder =
                        flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
                    encoder.write_all(&bytes)?;
                    let compressed = encoder.finish()?;

                    let new_response = HttpResponse::build(status)
                        .insert_header((header::CONTENT_ENCODING, "gzip"))
                        .insert_header((header::CONTENT_TYPE, content_type))
                        .body(compressed);

                    return Ok(ServiceResponse::new(req, new_response));
                }
            }

            Ok(res.map_into_boxed_body())
        })
    }
}
