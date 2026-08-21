use actix_web::error::ErrorBadRequest;
use actix_web::{
    Error, FromRequest, HttpMessage, HttpRequest,
    dev::{self, Service, Transform},
};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse};
use futures_util::future::LocalBoxFuture;
use std::future::{Ready, ready};
use std::task::{Context, Poll};
use uuid::Uuid;

/// Request ID middleware factory.
pub struct RequestIDWrapper;

impl<S, B> Transform<S, ServiceRequest> for RequestIDWrapper
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestIDMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestIDMiddleware { service }))
    }
}

/// Actual actix-web middleware
pub struct RequestIDMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestIDMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let request_id = Uuid::new_v4();

        req.extensions_mut().insert(RequestID(request_id));

        let fut = self.service.call(req);
        Box::pin(async move { fut.await })
    }
}

/// Request ID extractor
#[derive(Clone, Debug)]
pub struct RequestID(pub Uuid);

impl FromRequest for RequestID {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        if let Some(req_id) = req.extensions().get::<RequestID>() {
            ready(Ok(req_id.clone()))
        } else {
            ready(Err(ErrorBadRequest("request id is missing")))
        }
    }
}

impl std::fmt::Display for RequestID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}
