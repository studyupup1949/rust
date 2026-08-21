//! Timeout middleware — cancels requests that exceed a duration.

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use actix_service::{Service, Transform};
use actix_web::{
    body::MessageBody,
    dev::{forward_ready, ServiceRequest, ServiceResponse},
    Error,
};
use pin_project_lite::pin_project;
use tokio::time::Sleep;

/// Middleware that applies a timeout to each request.
///
/// If a request takes longer than the configured duration, a 408 Request Timeout
/// (or 503 Service Unavailable) is returned.
///
/// # Example
///
/// ```no_run
/// use actix_tower::prelude::*;
/// use std::time::Duration;
/// use actix_web::App;
///
/// let app = App::new()
///     .wrap(Timeout::new(Duration::from_secs(30)));
/// ```
#[derive(Clone, Debug)]
pub struct Timeout {
    duration: Duration,
}

impl Timeout {
    /// Create a new `Timeout` middleware with the given duration.
    pub fn new(duration: Duration) -> Self {
        Self { duration }
    }

    /// Set the timeout duration.
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}

impl<S, B> Transform<S, ServiceRequest> for Timeout
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = TimeoutMiddleware<S>;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(TimeoutMiddleware {
            service,
            timeout: self.duration,
        }))
    }
}

/// The actual timeout middleware service.
pub struct TimeoutMiddleware<S> {
    service: S,
    timeout: Duration,
}

impl<S, B> Service<ServiceRequest> for TimeoutMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = TimeoutFuture<S::Future>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        TimeoutFuture {
            fut: self.service.call(req),
            sleep: tokio::time::sleep(self.timeout),
        }
    }
}

pin_project! {
    /// Future for `TimeoutMiddleware`.
    pub struct TimeoutFuture<F> {
        #[pin]
        fut: F,
        #[pin]
        sleep: Sleep,
    }
}

impl<F, B> Future for TimeoutFuture<F>
where
    F: Future<Output = Result<ServiceResponse<B>, Error>>,
    B: MessageBody + 'static,
{
    type Output = Result<ServiceResponse, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        // Race the service future against the timeout
        match this.fut.poll(cx) {
            Poll::Ready(result) => {
                let res = result?;
                Poll::Ready(Ok(res.map_into_boxed_body()))
            }
            Poll::Pending => {
                // Check if the timeout has elapsed
                if this.sleep.poll(cx).is_ready() {
                    Poll::Ready(Err(actix_web::error::ErrorRequestTimeout(
                        "Request timed out",
                    )))
                } else {
                    Poll::Pending
                }
            }
        }
    }
}
