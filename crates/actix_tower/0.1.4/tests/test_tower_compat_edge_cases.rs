use actix_tower::prelude::*;
use actix_web::{test, web, App, HttpResponse};
use std::sync::{Arc, Mutex};
use tower::{Service as TowerService, Layer};
use actix_web::dev::Service as ActixService;
use std::task::{Context, Poll};
use std::future::Future;
use std::pin::Pin;

// ============================================================================
// Tower Compatibility Edge Cases
// ============================================================================

// A Tower layer that tracks clones
#[derive(Clone)]
struct CloneTrackingLayer {
    clone_count: Arc<Mutex<usize>>,
}

impl<S> Layer<S> for CloneTrackingLayer {
    type Service = CloneTrackingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CloneTrackingService {
            inner,
            clone_count: self.clone_count.clone(),
        }
    }
}

#[derive(Clone)]
struct CloneTrackingService<S> {
    inner: S,
    clone_count: Arc<Mutex<usize>>,
}

impl<S, Req> TowerService<Req> for CloneTrackingService<S>
where
    S: TowerService<Req> + Clone,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        let mut count = self.clone_count.lock().unwrap();
        *count += 1;
        self.inner.call(req)
    }
}

#[actix_web::test]
async fn test_tower_service_clone_behavior() {
    let clone_count = Arc::new(Mutex::new(0));
    let layer = CloneTrackingLayer { clone_count: clone_count.clone() };

    let app = test::init_service(
        App::new()
            .wrap(tower_layer!(layer))
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
    ).await;

    // Call it multiple times, ensure clones share the arc but don't incorrectly
    // leak or mutate shared state inappropriately.
    for _ in 0..5 {
        let req = test::TestRequest::get().uri("/").to_request();
        let _ = test::call_service(&app, req).await;
    }

    assert_eq!(*clone_count.lock().unwrap(), 5);
}

// A layer that records execution order
#[derive(Clone)]
struct OrderingLayer {
    id: usize,
    order: Arc<Mutex<Vec<usize>>>,
}

impl<S> Layer<S> for OrderingLayer {
    type Service = OrderingService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        OrderingService { inner, id: self.id, order: self.order.clone() }
    }
}

#[derive(Clone)]
struct OrderingService<S> {
    inner: S,
    id: usize,
    order: Arc<Mutex<Vec<usize>>>,
}

impl<S, Req> TowerService<Req> for OrderingService<S>
where
    S: TowerService<Req>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        self.order.lock().unwrap().push(self.id);
        self.inner.call(req)
    }
}

#[actix_web::test]
async fn test_tower_layer_ordering() {
    let order = Arc::new(Mutex::new(Vec::new()));
    
    // Stack multiple Tower layers using ServiceBuilder
    let layer = tower::ServiceBuilder::new()
        .layer(OrderingLayer { id: 1, order: order.clone() })
        .layer(OrderingLayer { id: 2, order: order.clone() })
        .layer(OrderingLayer { id: 3, order: order.clone() })
        .into_inner();

    let app = test::init_service(
        App::new()
            .wrap(tower_layer!(layer))
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
    ).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let _ = test::call_service(&app, req).await;

    // Tower ServiceBuilder executes outer layer first, wrapping inner layers.
    // layer(1) wraps layer(2) wraps layer(3) wraps inner_service.
    // Call order flows outside-in: 1 -> 2 -> 3
    assert_eq!(*order.lock().unwrap(), vec![1, 2, 3]);
}

// Error propagation service
#[derive(Clone)]
struct ErrorLayer;

impl<S> Layer<S> for ErrorLayer {
    type Service = ErrorService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        ErrorService { inner }
    }
}

#[derive(Clone)]
struct ErrorService<S> {
    inner: S,
}

impl<S, Req> TowerService<Req> for ErrorService<S>
where
    S: TowerService<Req>,
{
    type Response = S::Response;
    // Tower standard error is BoxError, but in actix_tower we convert it.
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.inner.poll_ready(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(_)) => Poll::Ready(Err("inner error".into())),
            Poll::Pending => Poll::Pending,
        }
    }

    fn call(&mut self, _req: Req) -> Self::Future {
        Box::pin(async { Err("simulated tower error".into()) })
    }
}

#[actix_web::test]
async fn test_tower_error_propagation() {
    let app = test::init_service(
        App::new()
            .wrap(tower_layer!(ErrorLayer))
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
    ).await;

    let req = test::TestRequest::get().uri("/").to_request();
    
    // In actix-web, middleware errors bubble up as actix_web::Error.
    // Call service usually returns a successful response if it's caught by error handlers,
    // but a hard error from middleware without an error handler will panic in tests unless caught,
    // or return a 500. Let's see what actix test does.
    let resp = app.call(req).await;
    
    // Actix allows middleware to return errors, which translates to a 500 response.
    match resp {
        Ok(res) => {
            // It might be mapped to an HttpResponse by a default error handler.
            assert_eq!(res.status().as_u16(), 500);
        }
        Err(e) => {
            // It bubbles up as an actix error wrapped in TowerError
            assert_eq!(e.to_string(), "tower middleware error: simulated tower error");
        }
    }
}

#[actix_web::test]
async fn test_tower_ready_service_drop() {
    let clone_count = Arc::new(Mutex::new(0));
    let layer = CloneTrackingLayer { clone_count: clone_count.clone() };

    let app = test::init_service(
        App::new()
            .wrap(tower_layer!(layer))
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
    ).await;

    let srv = app;
    
    // Drive it to ready but drop before calling
    let _ = std::future::poll_fn(|cx| srv.poll_ready(cx)).await;
    drop(srv);
    
    // Resources should be cleanly released. Arc count should drop.
    assert_eq!(Arc::strong_count(&clone_count), 1);
}

// A service that maintains state after an error
#[derive(Clone)]
struct FallibleStateLayer {
    error_counter: Arc<Mutex<usize>>,
}

impl<S> Layer<S> for FallibleStateLayer {
    type Service = FallibleStateService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        FallibleStateService { inner, error_counter: self.error_counter.clone() }
    }
}

#[derive(Clone)]
struct FallibleStateService<S> {
    inner: S,
    error_counter: Arc<Mutex<usize>>,
}

impl<S, Req> TowerService<Req> for FallibleStateService<S>
where
    S: TowerService<Req> + 'static,
    S::Error: std::fmt::Debug + Into<Box<dyn std::error::Error + Send + Sync>>,
    S::Future: 'static,
{
    type Response = S::Response;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|_| "bad".into())
    }

    fn call(&mut self, req: Req) -> Self::Future {
        let mut count = self.error_counter.lock().unwrap();
        *count += 1;
        let should_err = *count % 2 == 1; // Error on odds, succeed on evens

        if should_err {
            Box::pin(async { Err("forced error".into()) })
        } else {
            let fut = self.inner.call(req);
            Box::pin(async move {
                fut.await.map_err(|_| "bad inner".into())
            })
        }
    }
}

#[actix_web::test]
async fn test_tower_call_after_error() {
    let error_counter = Arc::new(Mutex::new(0));
    
    let app = test::init_service(
        App::new()
            .wrap(tower_layer!(FallibleStateLayer { error_counter }))
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
    ).await;

    // Call 1: Error
    let req1 = test::TestRequest::get().uri("/").to_request();
    let resp1 = app.call(req1).await;
    assert!(resp1.is_err());

    // Call 2: Success
    let req2 = test::TestRequest::get().uri("/").to_request();
    let resp2 = app.call(req2).await;
    assert!(resp2.is_ok());
    assert_eq!(resp2.unwrap().status().as_u16(), 200);

    // Call 3: Error
    let req3 = test::TestRequest::get().uri("/").to_request();
    let resp3 = app.call(req3).await;
    assert!(resp3.is_err());
}
