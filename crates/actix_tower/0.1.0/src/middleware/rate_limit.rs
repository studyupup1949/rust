//! Rate limiting middleware using a simple token-bucket algorithm.

use std::{
    collections::HashMap,
    future::{ready, Ready},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use actix_service::{Service, Transform};
use actix_web::{
    body::MessageBody,
    dev::{forward_ready, ServiceRequest, ServiceResponse},
    http::header,
    Error, HttpResponse,
};
use parking_lot::Mutex;

/// Configuration for the rate limiter.
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed in the window.
    pub max_requests: u32,
    /// Duration of the sliding window.
    pub window: Duration,
}

impl RateLimitConfig {
    /// Create a new rate limit config.
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            max_requests,
            window,
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_secs(60),
        }
    }
}

/// Rate limiting middleware.
///
/// Limits the number of requests per client IP within a time window.
///
/// # Example
///
/// ```no_run
/// use actix_tower::prelude::*;
/// use std::time::Duration;
/// use actix_web::App;
///
/// let app = App::new()
///     .wrap(RateLimit::new(100, Duration::from_secs(60)));
/// ```
#[derive(Clone)]
pub struct RateLimit {
    config: RateLimitConfig,
}

impl RateLimit {
    /// Create a new rate limiter.
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            config: RateLimitConfig::new(max_requests, window),
        }
    }

    /// Use a custom config.
    pub fn with_config(config: RateLimitConfig) -> Self {
        Self { config }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimit
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = RateLimitMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddleware {
            service,
            config: self.config.clone(),
            buckets: Arc::new(Mutex::new(HashMap::new())),
            // Each middleware instance owns its sweep counter.
            // Previously this was a process-global static, which meant multiple
            // RateLimitMiddleware instances coupled each other's sweep timing.
            sweep_counter: Arc::new(AtomicUsize::new(0)),
        }))
    }
}

/// The actual rate limit middleware service.
pub struct RateLimitMiddleware<S> {
    service: S,
    config: RateLimitConfig,
    buckets: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    /// Per-instance counter that controls periodic sweep frequency.
    /// Previously a process-global `static`, which caused all instances to
    /// share a single counter, coupling their sweep timing and potentially
    /// sweeping the wrong instance's buckets at unrelated intervals.
    sweep_counter: Arc<AtomicUsize>,
}

struct CheckResult {
    allowed: bool,
    remaining: u32,
}

impl<S> RateLimitMiddleware<S> {
    fn check(&self, key: &str) -> CheckResult {
        let now = Instant::now();
        let window = self.config.window;
        let max = self.config.max_requests;

        let mut buckets = self.buckets.lock();

        // Periodically sweep expired entries (every 100 checks on THIS instance).
        // Relaxed ordering is correct: the counter is only read and written by
        // this instance's single-threaded Actix worker.
        if self.sweep_counter.fetch_add(1, Ordering::Relaxed) % 100 == 0 {
            buckets.retain(|_, timestamps| {
                timestamps.retain(|&t| now.duration_since(t) < window);
                !timestamps.is_empty()
            });
        }

        let timestamps = buckets.entry(key.to_string()).or_default();

        // Remove timestamps outside the window.
        timestamps.retain(|&t| now.duration_since(t) < window);

        let count = timestamps.len() as u32;
        let remaining = max.saturating_sub(count);
        let allowed = remaining > 0;

        if allowed {
            timestamps.push(now);
        }

        let is_empty = timestamps.is_empty();

        let result = CheckResult {
            allowed,
            // Subtract 1 from remaining because we just consumed one slot.
            remaining: remaining.saturating_sub(if allowed { 1 } else { 0 }),
        };

        if is_empty {
            buckets.remove(key);
        }

        result
    }
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddleware<S>
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
        // Use peer_addr() instead of realip_remote_addr() to prevent trivial
        // IP spoofing via X-Forwarded-For headers if the app is exposed directly.
        let key = req
            .connection_info()
            .peer_addr()
            .unwrap_or("unknown")
            .to_string();

        let result = self.check(&key);

        if !result.allowed {
            let (req, _) = req.into_parts();
            let response = HttpResponse::TooManyRequests()
                .insert_header(("x-ratelimit-remaining", "0"))
                .insert_header(("x-ratelimit-limit", self.config.max_requests.to_string()))
                .body("Rate limit exceeded");
            return Box::pin(ready(Ok(ServiceResponse::new(req, response))));
        }

        let remaining = result.remaining;
        let max = self.config.max_requests;
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?.map_into_boxed_body();
            res.headers_mut().insert(
                header::HeaderName::from_static("x-ratelimit-remaining"),
                header::HeaderValue::from(remaining),
            );
            res.headers_mut().insert(
                header::HeaderName::from_static("x-ratelimit-limit"),
                header::HeaderValue::from(max),
            );
            Ok(res)
        })
    }
}
