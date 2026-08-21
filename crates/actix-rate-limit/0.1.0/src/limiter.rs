use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::task::{Context, Poll};

use futures_util::future;

use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    http::{HeaderName, HeaderValue},
    Error, HttpMessage, HttpResponse,
};

use super::redis_backend::*;
use super::types::*;
use super::util::*;

#[derive(Debug, Clone)]
pub struct RateLimit<Id, Backend> {
    _p: PhantomData<Id>,
    backend: Rc<Backend>,
    per_user: Option<LimitType>,
    per_ip: Option<LimitType>,
}

impl<Id, Backend> RateLimit<Id, Backend> {
    fn new(backend: Backend) -> Self
    where
        Backend: RateLimitBackend,
    {
        RateLimit {
            _p: PhantomData::<Id>,
            backend: Rc::new(backend),
            per_user: None,
            per_ip: None,
        }
    }

    /// `quota`: Requests for one user per hour
    pub fn per_user(mut self, quota: LimitType) -> Self {
        debug_assert!(quota > 0);
        self.per_user.replace(quota);
        self
    }

    /// `quota`: Requests for one ip per hour
    pub fn per_ip(mut self, quota: LimitType) -> Self {
        debug_assert!(quota > 0);
        self.per_ip.replace(quota);
        self
    }
}

impl<Id> RateLimit<Id, RedisBackend> {
    pub fn redis_prefix(redis: RedisAddr, prefix: &str) -> Self {
        RateLimit::new(RedisBackend::new(redis, prefix))
    }

    pub fn redis(redis: RedisAddr) -> Self {
        RateLimit::redis_prefix(redis, "RateLimit")
    }
}

impl<S, B, Id, Backend> Transform<S> for RateLimit<Id, Backend>
where
    Id: RateLimitId + 'static,
    Backend: RateLimitBackend + 'static,
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;

    type InitError = ();
    type Transform = RateLimitService<S, Id, Backend>;
    type Future = future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ok(RateLimitService {
            _p: PhantomData::<Id>,
            backend: self.backend.clone(),
            per_user: self.per_user.expect("`per_user` requreid"),
            per_ip: self.per_ip.expect("`per_ip` required"),
            next: Rc::new(RefCell::new(service)),
        })
    }
}

pub struct RateLimitService<S, Id, Backend> {
    _p: PhantomData<Id>,
    backend: Rc<Backend>,
    per_user: LimitType,
    per_ip: LimitType,
    next: Rc<RefCell<S>>,
}

impl<S, Id, Backend> RateLimitService<S, Id, Backend> {
    fn rate_limit(&self, req: &ServiceRequest) -> (String, LimitType)
    where
        Id: RateLimitId + 'static,
    {
        if let Some(id) = req.extensions().get::<Id>() {
            return (id.to_string(), self.per_user);
        }

        // It is important that that the following are NOT in a `else`
        // block, or it will `panic`.
        let id = req
            .connection_info()
            .realip_remote_addr()
            .map(ip_part)
            .map(|ip| format!("ip:{}", ip))
            .unwrap_or_default();
        (id, self.per_ip)
    }
}

impl<S, B, Id, Backend> Service for RateLimitService<S, Id, Backend>
where
    Id: RateLimitId + 'static,
    Backend: RateLimitBackend + 'static,
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;

    type Future = future::LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.next.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let next = self.next.clone();
        let backend = self.backend.clone();
        let (id, limit) = self.rate_limit(&req);

        Box::pin(async move {
            let remaining = match backend.touch(&id, limit).await {
                Err(_) => return Err(HttpResponse::InternalServerError().finish().into()),
                Ok(remaining) => {
                    if remaining == 0 {
                        let mut too_many_requests = HttpResponse::TooManyRequests();
                        for (name, value) in headers(remaining, limit) {
                            too_many_requests.header(name, value);
                        }

                        return Err(too_many_requests.finish().into());
                    }

                    remaining
                }
            };

            // It is important that `borrow_mut()` and `.await` are on
            // separate lines, or it will `panic`.
            let fut = next.borrow_mut().call(req);

            let mut res = fut.await?;
            for (name, value) in headers(remaining, limit) {
                res.headers_mut().insert(name, value);
            }

            Ok(res)
        })
    }
}

fn ip_part(addr: &str) -> &str {
    match addr.rfind(':') {
        Some(at) => &addr[..at],
        None => addr,
    }
}

fn headers(remaining: LimitType, limit: LimitType) -> Vec<(HeaderName, HeaderValue)> {
    let reset = seconds_elapsed_for_next_hour();

    vec![
        (
            HeaderName::from_bytes(b"X-RateLimit-Remaining").unwrap(),
            HeaderValue::from(remaining),
        ),
        (
            HeaderName::from_bytes(b"X-RateLimit-Limit").unwrap(),
            HeaderValue::from(limit),
        ),
        (
            HeaderName::from_bytes(b"X-RateLimit-Reset").unwrap(),
            HeaderValue::from(reset),
        ),
    ]
}
