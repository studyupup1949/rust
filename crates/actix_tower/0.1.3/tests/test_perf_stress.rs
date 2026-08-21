use actix_tower::prelude::*;
use actix_web::{test, web, App, HttpResponse};
use futures_util::future::join_all;

/// Test 19: test_connection_churn
/// Tests stability and performance when many simulated clients rapidly connect and disconnect.
#[actix_web::test]
async fn test_connection_churn() {
    let app = test::init_service(
        App::new()
            .wrap(tower_layer!(tower_http::trace::TraceLayer::new_for_http()))
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
    ).await;

    // Simulate 500 concurrent connections doing exactly 1 request
    let mut futs = Vec::with_capacity(500);
    for _ in 0..500 {
        let req = test::TestRequest::get().uri("/").to_request();
        // Since we are using an initialized service `app` via Actix testing framework, 
        // we can just call it concurrently if we clone the service.
        // `init_service` returns a single instance, but we can do sequential calls
        // rapidly to test churn, or we can use the `call` API directly.
        futs.push(test::call_service(&app, req));
    }

    let results = join_all(futs).await;

    for res in results {
        assert_eq!(res.status().as_u16(), 200);
    }
}

/// Test 20: test_memory_stability_under_load
/// Sends many requests sequentially through the optimized bridge to ensure no leaks
/// happen across requests. (This acts as a memory stability smoke test).
#[actix_web::test]
async fn test_memory_stability_under_load() {
    let app = test::init_service(
        App::new()
            .wrap(tower_layer!(tower_http::trace::TraceLayer::new_for_http()))
            .route("/", web::post().to(|| async { HttpResponse::Ok().finish() }))
    ).await;

    // Send 10,000 requests. If the thread-local Rc leaked memory, this would trigger
    // significant growth. The actual validation is passing the test without an OOM
    // and maintaining execution speed.
    for _ in 0..10_000 {
        let req = test::TestRequest::post()
            .uri("/")
            .insert_header(("x-test-header", "some value"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
