use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use futures_util::future;
use std::task::{Context, Poll};

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpMessage, HttpResponse};

use super::types::*;
use super::util::*;

#[derive(Debug, Clone)]
pub struct RateLimit<Id, Backend> {
    _p: PhantomData<Id>,
    backend: Rc<Backend>,
    per_user: LimitType,
    per_ip: LimitType,
}

impl<Id, Backend> RateLimit<Id, Backend> {
    pub(crate) fn new(backend: Backend) -> Self
    where
        Id: RateLimitId,
        Backend: RateLimitBackend,
    {
        RateLimit {
            _p: PhantomData,
            backend: Rc::new(backend),
            per_user: 0,
            per_ip: 0,
        }
    }

    pub(crate) fn backend_mut(&mut self) -> &mut Backend {
        Rc::get_mut(&mut self.backend).expect("Multiple copies exist")
    }

    /// Set requests for one user per hour
    /// > No limits if `quota` is 0.
    pub fn per_user(mut self, quota: LimitType) -> Self {
        self.per_user = quota;
        self
    }

    /// Set requests for one ip per hour
    /// > No limits if `quota` is 0.
    pub fn per_ip(mut self, quota: LimitType) -> Self {
        self.per_ip = quota;
        self
    }
}

impl<Id, Backend, S, B> Transform<S> for RateLimit<Id, Backend>
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
    type Transform = RateLimitMiddleware<Id, Backend, S>;
    type Future = future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ok(RateLimitMiddleware {
            _p: PhantomData::<Id>,
            next: Rc::new(RefCell::new(service)),
            headers: Rc::new(Headers::new()),
            backend: self.backend.clone(),
            per_user: self.per_user,
            per_ip: self.per_ip,
        })
    }
}

pub struct RateLimitMiddleware<Id, Backend, S> {
    _p: PhantomData<Id>,
    next: Rc<RefCell<S>>,
    backend: Rc<Backend>,
    headers: Rc<Headers>,
    per_user: LimitType,
    per_ip: LimitType,
}

impl<Id, Backend, S> RateLimitMiddleware<Id, Backend, S>
where
    Id: RateLimitId + 'static,
    Backend: RateLimitBackend + 'static,
{
    fn limit(&self, req: &ServiceRequest) -> (String, LimitType) {
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

impl<Id, Backend, S, B> Service for RateLimitMiddleware<Id, Backend, S>
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
        let headers = self.headers.clone();
        let backend = self.backend.clone();
        let (id, limit) = self.limit(&req);

        Box::pin(async move {
            if limit > 0 {
                match backend.touch(&id, limit).await.map_err(fail)? {
                    remaining if remaining == 0 => {
                        let mut res = HttpResponse::TooManyRequests();
                        for (name, value) in headers.of(remaining, limit) {
                            res.header(name, value);
                        }

                        Err(res.finish().into())
                    }

                    remaining => {
                        let fut = next.borrow_mut().call(req);

                        let mut res = fut.await?;
                        for (name, value) in headers.of(remaining, limit) {
                            res.headers_mut().insert(name, value);
                        }

                        Ok(res)
                    }
                }
            } else {
                // It is important that `borrow_mut()` and `.await` are on
                // separate lines, or it will `panic`.
                let fut = next.borrow_mut().call(req);
                let res = fut.await?;

                Ok(res)
            }
        })
    }
}

fn fail<T>(_: T) -> HttpResponse {
    HttpResponse::InternalServerError().finish()
}

fn ip_part(addr: &str) -> &str {
    match addr.rfind(':') {
        Some(at) => &addr[..at],
        None => addr,
    }
}

use actix_web::http::{HeaderName, HeaderValue};

struct Headers {
    remaining: HeaderName,
    reset: HeaderName,
    limit: HeaderName,
}

impl Headers {
    pub fn new() -> Self {
        Headers {
            remaining: HeaderName::from_bytes(b"X-RateLimit-Remaining").unwrap(),
            reset: HeaderName::from_bytes(b"X-RateLimit-Reset").unwrap(),
            limit: HeaderName::from_bytes(b"X-RateLimit-Limit").unwrap(),
        }
    }

    #[rustfmt::skip]
    pub fn of(&self, remaining: LimitType, limit: LimitType) -> Vec<(HeaderName, HeaderValue)> {
        let reset = seconds_elapsed_for_next_hour();

        vec![
            (HeaderName::from(&self.remaining), HeaderValue::from(remaining)),
            (HeaderName::from(&self.reset), HeaderValue::from(reset)),
            (HeaderName::from(&self.limit), HeaderValue::from(limit)),
        ]
    }
}
