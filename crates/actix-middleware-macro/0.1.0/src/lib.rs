//! # actix-middleware-macro
//!
//! For usage see the doc for the [`create_middleware`] macro.

/// Macro to to reduce boilerplate codes for simple actix_web middleware.
///
/// # Example
/// ```rust
/// use actix_middleware_macro::create_middleware;
///
/// create_middleware!(
///     TimingHeaders,
///     |ctx: &MiddlewareTransform<S>, req: ServiceRequest| {
///         use actix_web::http::header::{HeaderName, HeaderValue};
///         use chrono::Utc;
///     
///         let start = Utc::now();
///     
///         let fut = ctx.service.call(req);
///         Box::pin(async move {
///             let mut res = fut.await?;
///             let duration = Utc::now() - start;
///             res.headers_mut().insert(
///                 HeaderName::from_static("x-app-time-ms"),
///                 HeaderValue::from_str(&format!("{}", duration.num_milliseconds()))?,
///             );
///             Ok(res)
///         })
///     }
/// );
///
/// // Usage
/// #[actix_web::test]
/// async fn works() {
///     let _server = tokio::spawn(async {
///         actix_web::HttpServer::new(|| {
///             actix_web::App::new()
///                 .default_service(web::to(|| async { actix_web::HttpResponse::Ok() }))
///                 .wrap(timing_cors_headers_middleware::Middleware)
///         })
///         .bind("127.1:8080")
///         .unwrap()
///         .run()
///         .await
///         .unwrap();
///     });
///     tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
///     let response = ureq::get("http://127.1:8080").call().unwrap();
///     assert!(response.header("x-app-time-ms").is_some());
/// }
/// ```
#[macro_export]
macro_rules! create_middleware {
    ($name: ident, $code: expr) => {
        paste::paste! {
            mod [<$name:snake _middleware>] {
                use futures_util::future::LocalBoxFuture;
                use std::future::{ready, Ready};

                use actix_web::{
                    dev::{Service, ServiceRequest, ServiceResponse, Transform},
                    Error
                };

                pub struct Middleware;
                pub struct MiddlewareTransform<S> {
                    service: S,
                }

                impl<S, B> Transform<S, ServiceRequest> for Middleware
                where
                    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
                    S::Future: 'static,
                    B: 'static,
                {
                    type Response = ServiceResponse<B>;
                    type Error = Error;
                    type InitError = ();
                    type Transform = MiddlewareTransform<S>;
                    type Future = Ready<Result<Self::Transform, Self::InitError>>;

                    fn new_transform(&self, service: S) -> Self::Future {
                        ready(Ok(MiddlewareTransform { service }))
                    }
                }

                impl<S, B> Service<ServiceRequest> for MiddlewareTransform<S>
                where
                    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
                    S::Future: 'static,
                    B: 'static,
                {
                    type Response = ServiceResponse<B>;

                    type Error = Error;

                    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

                    fn poll_ready(
                        &self,
                        ctx: &mut core::task::Context<'_>,
                    ) -> std::task::Poll<Result<(), Self::Error>> {
                        self.service.poll_ready(ctx)
                    }

                    fn call(&self, req: ServiceRequest) -> Self::Future {
                        $code(self, req)
                    }
                }
            }
        }
    };
}

#[cfg(test)]
mod test {
    use actix_web::web;

    use super::*;

    create_middleware!(
        TimingCorsHeaders,
        |ctx: &MiddlewareTransform<S>, req: ServiceRequest| {
            use actix_web::http::header::{HeaderName, HeaderValue};
            use chrono::Utc;

            let start = Utc::now();

            let fut = ctx.service.call(req);
            Box::pin(async move {
                let mut res = fut.await?;
                let duration = Utc::now() - start;
                res.headers_mut().insert(
                    HeaderName::from_static("x-app-time-ms"),
                    HeaderValue::from_str(&format!("{}", duration.num_milliseconds()))?,
                );
                res.headers_mut().insert(
                    HeaderName::from_static("x-app-time-micros"),
                    HeaderValue::from_str(&format!(
                        "{}",
                        duration.num_microseconds().unwrap_or_default()
                    ))?,
                );
                // CORS header
                res.headers_mut().insert(
                    HeaderName::from_static("access-control-allow-origin"),
                    HeaderValue::from_str("*")?,
                );
                res.headers_mut().insert(
                    HeaderName::from_static("access-control-allow-methods"),
                    HeaderValue::from_str("GET, POST, OPTIONS")?,
                );
                Ok(res)
            })
        }
    );

    #[actix_web::test]
    async fn works() {
        let _server = tokio::spawn(async {
            actix_web::HttpServer::new(|| {
                actix_web::App::new()
                    .default_service(web::to(|| async { actix_web::HttpResponse::Ok() }))
                    .wrap(timing_cors_headers_middleware::Middleware)
            })
            .bind("127.1:8080")
            .unwrap()
            .run()
            .await
            .unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let response = ureq::get("http://127.1:8080").call().unwrap();
        assert!(response.header("x-app-time-ms").is_some());
    }
}
