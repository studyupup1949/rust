//! Metrics middleware for collecting request statistics.

use std::{
    collections::HashMap,
    future::{ready, Ready},
    sync::Arc,
    time::{Duration, Instant},
};

use actix_service::{Service, Transform};
use actix_web::{
    dev::{forward_ready, ServiceRequest, ServiceResponse},
    Error,
};
use parking_lot::RwLock;

/// Metrics collected by the middleware.
#[derive(Debug, Clone, Default)]
pub struct MetricsData {
    /// Total number of requests.
    pub total_requests: u64,
    /// Number of requests per status code class (2xx, 4xx, 5xx, etc.).
    pub status_counts: HashMap<u16, u64>,
    /// Total response time.
    pub total_duration: Duration,
}

/// Shared metrics store.
#[derive(Clone, Default)]
pub struct MetricsStore {
    inner: Arc<RwLock<MetricsData>>,
}

impl MetricsStore {
    /// Create a new empty metrics store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a snapshot of the current metrics.
    pub fn snapshot(&self) -> MetricsData {
        self.inner.read().clone()
    }

    /// Reset all metrics.
    pub fn reset(&self) {
        *self.inner.write() = MetricsData::default();
    }

    fn record(&self, status: u16, duration: Duration) {
        let mut data = self.inner.write();
        data.total_requests += 1;
        *data.status_counts.entry(status).or_default() += 1;
        data.total_duration += duration;
    }
}

/// Metrics middleware that collects request statistics.
///
/// # Example
///
/// ```no_run
/// use actix_tower::prelude::*;
/// use actix_web::App;
///
/// let metrics = MetricsStore::new();
/// let app = App::new()
///     .wrap(Metrics::new(metrics.clone()));
/// ```
#[derive(Clone)]
pub struct Metrics {
    store: MetricsStore,
}

impl Metrics {
    /// Create a new metrics middleware.
    pub fn new(store: MetricsStore) -> Self {
        Self { store }
    }
}

impl<S, B> Transform<S, ServiceRequest> for Metrics
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = MetricsMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MetricsMiddleware {
            service,
            store: self.store.clone(),
        }))
    }
}

/// Metrics middleware service.
pub struct MetricsMiddleware<S> {
    service: S,
    store: MetricsStore,
}

impl<S, B> Service<ServiceRequest> for MetricsMiddleware<S>
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
        let start = Instant::now();
        let store = self.store.clone();
        let fut = self.service.call(req);

        Box::pin(async move {
            let result = fut.await;
            let duration = start.elapsed();
            let status = match &result {
                Ok(res) => res.status().as_u16(),
                Err(_) => 500,
            };
            store.record(status, duration);
            result
        })
    }
}
