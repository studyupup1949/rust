//! Implementation of the rate-limiting middleware.

use std::{
    future::{ready, Ready},
    rc::Rc,
    sync::Arc,
};

use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures::{future::LocalBoxFuture, FutureExt};
use log::debug;
use tokio::sync::Mutex;

use crate::{backend::BackendProvider, RateLimiter};

/// Factory that creates an instance of the `RateLimitedMiddleware`. You should
/// use this factory for wrapping the crate's middleware.
#[derive(Clone, Debug)]
pub struct RateLimiterMiddlewareFactory<BP: BackendProvider>
where
    BP: Sized + Clone,
{
    limiter: RateLimiter,
    backend: Arc<Mutex<BP>>,
}

impl<BP: BackendProvider> RateLimiterMiddlewareFactory<BP>
where
    BP: Sized + Clone,
{
    /// Creates a new instance of `RateLimiterMiddlewareFactory` with the
    /// provided limiter and backend.
    pub fn new(limiter: RateLimiter, backend: Arc<Mutex<BP>>) -> Self {
        Self { limiter, backend }
    }
}

impl<S, B, BP: BackendProvider> Transform<S, ServiceRequest> for RateLimiterMiddlewareFactory<BP>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    BP: Sized + Clone + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = RateLimiterMiddleware<S, BP>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimiterMiddleware {
            limiter: self.limiter.clone(),
            backend: self.backend.clone(),
            service: Rc::new(service),
        }))
    }
}

/// Rate-limiting middleware itself. Should never be initialized by the end-user.
#[derive(Clone, Debug)]
pub struct RateLimiterMiddleware<S, BP: BackendProvider>
where
    BP: Sized + Clone,
{
    limiter: RateLimiter,
    backend: Arc<Mutex<BP>>,
    service: Rc<S>,
}

impl<S, B, BP: BackendProvider> Service<ServiceRequest> for RateLimiterMiddleware<S, BP>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    BP: Sized + Clone + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path();
        let limit = self.limiter.get_limit(path);
        let connection_info = req.connection_info().clone();

        let ip = if connection_info.realip_remote_addr().is_some() {
            connection_info.realip_remote_addr().unwrap()
        } else {
            connection_info.peer_addr().unwrap()
        };

        let id = format!("{}:{}", ip, path);
        let srv = self.service.clone();
        let backend = self.backend.clone();

        debug!("Rate-Limiter middleware is called with id {}.", id);
        async move {
            let result = backend.lock().await.validate_request(&id, limit).await;
            if result.is_err() {
                debug!("{} is rate-limited.", id);
                return Ok(req
                    .into_response(HttpResponse::TooManyRequests())
                    .map_into_right_body());
            }

            let response = srv.call(req).await?;
            Ok(response.map_into_left_body())
        }
        .boxed_local()
    }
}
