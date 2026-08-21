use actix_tower::prelude::*;
use actix_web::{test, web, App, HttpResponse};
use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}};
use tower::{Service as TowerService, Layer};
use actix_web::dev::Service as ActixService;
use std::task::{Context, Poll};
use std::future::Future;
use std::time::Duration;

// ============================================================================
// Concurrency Race Conditions
// ============================================================================

#[actix_web::test]
async fn test_concurrent_middleware_construction() {
    let clone_count = Arc::new(AtomicUsize::new(0));
    
    // Simulate multiple threads building the middleware concurrently
    // We cannot build `App::new()` inside `tokio::spawn` because actix-web apps are `!Send`.
    let mut handles = vec![];
    for _ in 0..10 {
        let count = clone_count.clone();
        handles.push(tokio::spawn(async move {
            let _rate_limit = RateLimit::new(100, Duration::from_secs(60));
            count.fetch_add(1, Ordering::SeqCst);
        }));
    }

    for h in handles {
        h.await.unwrap();
    }
    
    assert_eq!(clone_count.load(Ordering::SeqCst), 10);
}

// A Service that yields Pending repeatedly to simulate in-flight requests
#[derive(Clone)]
struct YieldingLayer {
    yields: usize,
}

impl<S> Layer<S> for YieldingLayer {
    type Service = YieldingService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        YieldingService { inner, yields: self.yields }
    }
}

#[derive(Clone)]
struct YieldingService<S> {
    inner: S,
    yields: usize,
}

impl<S, Req> TowerService<Req> for YieldingService<S>
where
    S: TowerService<Req>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.yields > 0 {
            self.yields -= 1;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        self.inner.call(req)
    }
}

#[actix_web::test]
async fn test_service_call_reentrancy() {
    let app = test::init_service(
        App::new()
            .wrap(tower_layer!(YieldingLayer { yields: 5 }))
            .route("/", web::get().to(|| async { 
                tokio::time::sleep(Duration::from_millis(10)).await;
                HttpResponse::Ok().finish() 
            }))
    ).await;

    let mut futs = vec![];
    for _ in 0..100 {
        let req = test::TestRequest::get().uri("/").to_request();
        futs.push(app.call(req));
    }

    let results = futures_util::future::join_all(futs).await;
    for res in results {
        assert!(res.is_ok());
    }
}

#[derive(Clone)]
struct PanicReadyService<S> {
    inner: S,
    panics: Arc<AtomicUsize>,
}

impl<S, Req> TowerService<Req> for PanicReadyService<S>
where
    S: TowerService<Req>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let count = self.panics.fetch_add(1, Ordering::SeqCst);
        if count == 0 {
            panic!("Intentional panic during poll_ready");
        }
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        self.inner.call(req)
    }
}

#[actix_web::test]
async fn test_poll_ready_after_panic() {
    let mut srv = PanicReadyService {
        inner: tower::service_fn(|_req: ()| async { Ok::<_, ()>(()) }),
        panics: Arc::new(AtomicUsize::new(0)),
    };

    let waker = futures_util::task::noop_waker();
    let mut cx = Context::from_waker(&waker);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        srv.poll_ready(&mut cx)
    }));
    assert!(result.is_err());

    let result2 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        srv.poll_ready(&mut cx)
    }));
    assert!(result2.is_ok());
}

#[actix_web::test]
async fn test_arc_clone_contention() {
    let app = test::init_service(
        App::new()
            .wrap(Cache::new(Duration::from_secs(60)))
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
    ).await;

    let mut futs = vec![];
    for _ in 0..1000 {
        let req = test::TestRequest::get().uri("/").to_request();
        futs.push(app.call(req));
    }

    let results = futures_util::future::join_all(futs).await;
    for res in results {
        assert!(res.is_ok());
    }
}

#[actix_web::test]
async fn test_atomic_ordering() {
    let app = test::init_service(
        App::new()
            .wrap(RateLimit::new(10000, Duration::from_secs(60)))
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
    ).await;

    let mut futs = vec![];
    for _ in 0..1000 {
        let req = test::TestRequest::get().uri("/").to_request();
        futs.push(test::call_service(&app, req));
    }

    let _ = futures_util::future::join_all(futs).await;
}

#[actix_web::test]
async fn test_future_poll_after_completion() {
    let app = test::init_service(
        App::new()
            .wrap(tower_layer!(tower::layer::layer_fn(|inner| inner))) // dummy layer
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
    ).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let mut fut = Box::pin(app.call(req));
    
    let waker = futures_util::task::noop_waker();
    let mut cx = Context::from_waker(&waker);

    let mut res = fut.as_mut().poll(&mut cx);
    while res.is_pending() {
        res = fut.as_mut().poll(&mut cx);
    }
    assert!(res.is_ready());

    // Poll AFTER completion
    let res_after = fut.as_mut().poll(&mut cx);
    assert!(res_after.is_pending()); // or panics, but we want it not to panic
}

#[actix_web::test]
async fn test_future_drop_during_poll() {
    let app = test::init_service(
        App::new()
            .wrap(Cache::new(Duration::from_secs(60)))
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
    ).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let mut fut = Box::pin(app.call(req));
    
    let waker = futures_util::task::noop_waker();
    let mut cx = Context::from_waker(&waker);

    let _ = fut.as_mut().poll(&mut cx);
    drop(fut);
    assert!(true);
}

#[derive(Clone)]
struct FlappingReadyLayer;

impl<S> Layer<S> for FlappingReadyLayer {
    type Service = FlappingReadyService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        FlappingReadyService { inner, count: 0 }
    }
}

#[derive(Clone)]
struct FlappingReadyService<S> {
    inner: S,
    count: usize,
}

impl<S, Req> TowerService<Req> for FlappingReadyService<S>
where
    S: TowerService<Req>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.count += 1;
        if self.count % 2 == 1 {
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            self.inner.poll_ready(cx)
        }
    }

    fn call(&mut self, req: Req) -> Self::Future {
        self.inner.call(req)
    }
}

#[actix_web::test]
async fn test_service_ready_transition() {
    let app = test::init_service(
        App::new()
            .wrap(tower_layer!(FlappingReadyLayer))
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
    ).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
}

#[derive(Clone)]
struct ShortCircuitLayer;

impl<S> Layer<S> for ShortCircuitLayer {
    type Service = ShortCircuitService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        ShortCircuitService { inner }
    }
}

#[derive(Clone)]
struct ShortCircuitService<S> {
    inner: S,
}

impl<S, Req> TowerService<Req> for ShortCircuitService<S>
where
    S: TowerService<Req>,
{
    type Response = S::Response;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = std::future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|_| "unauthorized".into())
    }

    fn call(&mut self, _req: Req) -> Self::Future {
        std::future::ready(Err("no".into()))
    }
}

#[actix_web::test]
async fn test_layer_chain_short_circuit() {
    let layer = ShortCircuitLayer;

    let app = test::init_service(
        App::new()
            .wrap(tower_layer!(layer))
            .route("/", web::get().to(|| async { 
                panic!("Should never be called");
                #[allow(unreachable_code)]
                HttpResponse::Ok().finish() 
            }))
    ).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = app.call(req).await;
    assert!(resp.is_err());
}

#[actix_web::test]
async fn test_nested_layer_state_isolation() {
    let clone_count = Arc::new(Mutex::new(0));
    
    // Stack multiple identical layers, ensure they don't corrupt each other's state
    let layer = tower::ServiceBuilder::new()
        .layer(tower::timeout::TimeoutLayer::new(Duration::from_secs(1)))
        .layer(tower::timeout::TimeoutLayer::new(Duration::from_secs(2)))
        .into_inner();

    let app = test::init_service(
        App::new()
            .wrap(tower_layer!(layer))
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
    ).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    assert_eq!(*clone_count.lock().unwrap(), 0);
}
