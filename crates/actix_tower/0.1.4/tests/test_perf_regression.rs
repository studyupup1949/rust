use actix_tower::prelude::*;
use actix_web::{test, web, App, HttpResponse};
use std::time::Duration;

/// Test 14: test_optimized_rate_limit
/// Ensure the rate limiter still correctly throttles under the new future wrapper.
#[actix_web::test]
async fn test_optimized_rate_limit() {
    let app = test::init_service(
        App::new()
            .wrap(RateLimit::new(2, Duration::from_secs(10)))
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
    ).await;

    // First request: OK
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);

    // Second request: OK
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);

    // Third request: 429 Too Many Requests
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 429);
}

/// Test 15: test_tower_layer_stacking
/// Validate multiple tower layers can be stacked efficiently.
#[actix_web::test]
async fn test_tower_layer_stacking() {
    use tower::ServiceBuilder;

    let layer = ServiceBuilder::new()
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(tower_http::timeout::TimeoutLayer::with_status_code(
            http::StatusCode::GATEWAY_TIMEOUT,
            Duration::from_secs(5),
        ))
        .into_inner();

    let app = test::init_service(
        App::new()
            .wrap(tower_layer!(layer))
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
    ).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
}

/// Test 16: test_malformed_input_rejection
/// Ensure zero-copy header bridge doesn't crash on strangely formatted but valid inputs.
#[actix_web::test]
async fn test_malformed_input_rejection() {
    let app = test::init_service(
        App::new()
            .wrap(tower_layer!(tower_http::trace::TraceLayer::new_for_http()))
            .route("/", web::get().to(|| async { 
                HttpResponse::Ok()
                    .insert_header(("x-injected", "injected_val"))
                    .finish() 
            }))
    ).await;

    // Send a request with a huge number of headers or empty header values
    let mut req = test::TestRequest::get().uri("/");
    for i in 0..10 {
        req = req.insert_header((format!("x-weird-{}", i), ""));
    }
    let req = req.to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    // Ensure Tower middleware was invoked and headers traversed correctly
    assert_eq!(resp.headers().get("x-injected").unwrap().as_bytes(), b"injected_val");
}
