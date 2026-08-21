use actix_tower::prelude::*;
use actix_tower::compat::tower::body::{ActixRequestBody, ActixResponseBody};
use actix_web::{test, web, App, HttpResponse, dev::Service};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tower_service::Service as TowerService;
use std::future::Future;
use std::pin::Pin;

#[derive(Clone)]
struct TestService {
    rc: Arc<Mutex<usize>>,
    poll_ready_yields: Arc<Mutex<usize>>,
}

impl TowerService<http::Request<ActixRequestBody>> for TestService {
    type Response = http::Response<ActixResponseBody<actix_web::body::BoxBody>>;
    type Error = std::convert::Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut yields = self.poll_ready_yields.lock().unwrap();
        if *yields > 0 {
            *yields -= 1;
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(Ok(()))
        }
    }

    fn call(&mut self, _req: http::Request<ActixRequestBody>) -> Self::Future {
        let rc = self.rc.clone();
        Box::pin(async move {
            *rc.lock().unwrap() += 1;
            
            // simulate an await point
            tokio::task::yield_now().await;
            
            *rc.lock().unwrap() += 1;

            let body = ActixResponseBody::from_box_body(actix_web::body::BoxBody::new("ok"));
            Ok(http::Response::new(body))
        })
    }
}

struct TestLayer {
    rc: Arc<Mutex<usize>>,
    poll_ready_yields: Arc<Mutex<usize>>,
}

impl<S> tower_layer::Layer<S> for TestLayer {
    type Service = TestService;
    fn layer(&self, _inner: S) -> Self::Service {
        TestService { rc: self.rc.clone(), poll_ready_yields: self.poll_ready_yields.clone() }
    }
}

/// Test 8: test_rc_refcell_safety
#[actix_web::test]
async fn test_rc_refcell_safety() {
    let rc = Arc::new(Mutex::new(0));
    
    let layer = tower_layer!(TestLayer { rc: rc.clone(), poll_ready_yields: Arc::new(Mutex::new(0)) });

    let app = test::init_service(
        App::new()
            .wrap(layer)
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
    ).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
    assert_eq!(*rc.lock().unwrap(), 2);
}

/// Test 9: test_poll_ready_contract
#[actix_web::test]
async fn test_poll_ready_contract() {
    let rc = Arc::new(Mutex::new(0));
    
    // Simulate backpressure by yielding 3 times before returning Ready
    let layer = tower_layer!(TestLayer { rc: rc.clone(), poll_ready_yields: Arc::new(Mutex::new(3)) });

    let app = test::init_service(
        App::new()
            .wrap(layer)
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
    ).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
    assert_eq!(*rc.lock().unwrap(), 2);
}

/// Test 10: test_task_cancellation_drop
#[actix_web::test]
async fn test_task_cancellation_drop() {
    let rc = Arc::new(Mutex::new(0));
    
    let layer = tower_layer!(TestLayer { rc: rc.clone(), poll_ready_yields: Arc::new(Mutex::new(0)) });

    let app = test::init_service(
        App::new()
            .wrap(layer)
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
    ).await;

    let req = test::TestRequest::get().uri("/").to_request();
    
    // Call the service but drop the future immediately (simulate cancellation)
    let fut = app.call(req);
    drop(fut);

    // Arc count should be exactly the strong count of the clones within the wrapper,
    // plus the one we hold here. None leaked by a suspended future.
    assert_eq!(Arc::strong_count(&rc), 2); // 1 here, 1 in the layer/service created by init_service
}
