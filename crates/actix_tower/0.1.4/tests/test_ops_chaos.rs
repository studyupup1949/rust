use actix_tower::prelude::*;
use actix_web::{test, web, App, HttpResponse};
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::time::Duration;
use actix_web::dev::Service as ActixService;

#[derive(Clone)]
struct ChaosDelayLayer {
    delay: Duration,
}

impl<S> tower::Layer<S> for ChaosDelayLayer {
    type Service = ChaosDelayService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        ChaosDelayService { inner, delay: self.delay }
    }
}

#[derive(Clone)]
struct ChaosDelayService<S> {
    inner: S,
    delay: Duration,
}

impl<S, Req> tower::Service<Req> for ChaosDelayService<S>
where
    S: tower::Service<Req> + Clone + 'static,
    Req: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        let mut inner = self.inner.clone();
        let delay = self.delay;
        Box::pin(async move {
            tokio::time::sleep(delay).await;
            inner.call(req).await
        })
    }
}

#[actix_web::test]
async fn test_chaos_delay_injection() {
    let layer = ChaosDelayLayer { delay: Duration::from_millis(10) };
    let app = test::init_service(
        App::new()
            .wrap(tower_layer!(layer))
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
    ).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = app.call(req).await;
    assert!(resp.is_ok());
}

#[derive(Clone)]
struct MetricsLayer {
    counter: Arc<AtomicUsize>,
}

impl<S> tower::Layer<S> for MetricsLayer {
    type Service = MetricsService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        MetricsService { inner, counter: self.counter.clone() }
    }
}

#[derive(Clone)]
struct MetricsService<S> {
    inner: S,
    counter: Arc<AtomicUsize>,
}

impl<S, Req> tower::Service<Req> for MetricsService<S>
where
    S: tower::Service<Req>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        self.counter.fetch_add(1, Ordering::SeqCst);
        self.inner.call(req)
    }
}

#[actix_web::test]
async fn test_ecosystem_metrics() {
    let counter = Arc::new(AtomicUsize::new(0));
    let layer = MetricsLayer { counter: counter.clone() };

    let app = test::init_service(
        App::new()
            .wrap(tower_layer!(layer))
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
    ).await;

    for _ in 0..10 {
        let req = test::TestRequest::get().uri("/").to_request();
        let _ = app.call(req).await;
    }

    assert_eq!(counter.load(Ordering::SeqCst), 10);
}

#[actix_web::test]
async fn test_graceful_shutdown_behavior() {
    let layer = tower::timeout::TimeoutLayer::new(Duration::from_secs(60));
    let app = test::init_service(
        App::new()
            .wrap(tower_layer!(layer))
            .route("/", web::get().to(|| async { 
                tokio::time::sleep(Duration::from_millis(50)).await;
                HttpResponse::Ok().finish() 
            }))
    ).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let fut = app.call(req);
    
    // Simulate drop mid-flight
    let mut pinned = Box::pin(fut);
    let waker = futures_util::task::noop_waker();
    let mut cx = std::task::Context::from_waker(&waker);
    let _ = std::future::Future::poll(pinned.as_mut(), &mut cx);
    
    drop(pinned);
}
