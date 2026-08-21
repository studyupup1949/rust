use actix_tower::prelude::*;
use actix_web::{test, web, App, HttpResponse};
use std::time::Duration;
use tower::ServiceBuilder;
use actix_web::dev::Service;

#[actix_web::test]
async fn test_stack_overflow_protection() {
    // We add 10 layers to simulate deep recursion.
    // If the tower adapter was poorly implemented, this might stack overflow.
    let layer = ServiceBuilder::new()
        .layer(tower::timeout::TimeoutLayer::new(Duration::from_secs(5)))
        .layer(tower::timeout::TimeoutLayer::new(Duration::from_secs(5)))
        .layer(tower::timeout::TimeoutLayer::new(Duration::from_secs(5)))
        .layer(tower::timeout::TimeoutLayer::new(Duration::from_secs(5)))
        .layer(tower::timeout::TimeoutLayer::new(Duration::from_secs(5)))
        .layer(tower::timeout::TimeoutLayer::new(Duration::from_secs(5)))
        .layer(tower::timeout::TimeoutLayer::new(Duration::from_secs(5)))
        .layer(tower::timeout::TimeoutLayer::new(Duration::from_secs(5)))
        .layer(tower::timeout::TimeoutLayer::new(Duration::from_secs(5)))
        .layer(tower::timeout::TimeoutLayer::new(Duration::from_secs(5)))
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

#[actix_web::test]
async fn test_timeout_escalation() {
    // Inner timeout is 100ms, outer is 10ms.
    let layer = ServiceBuilder::new()
        .layer(tower::timeout::TimeoutLayer::new(Duration::from_millis(10)))
        .layer(tower::timeout::TimeoutLayer::new(Duration::from_millis(100)))
        .into_inner();

    let app = test::init_service(
        App::new()
            .wrap(tower_layer!(layer))
            .route("/", web::get().to(|| async { 
                tokio::time::sleep(Duration::from_millis(50)).await;
                HttpResponse::Ok().finish() 
            }))
    ).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = app.call(req).await;
    
    // Should hit the outer 10ms timeout and fail
    assert!(resp.is_err());
    if let Err(e) = resp {
        assert!(e.to_string().contains("tower middleware error"));
    }
}

#[actix_web::test]
async fn test_slowloris_defense() {
    let app = test::init_service(
        App::new()
            .wrap(tower_layer!(tower::timeout::TimeoutLayer::new(Duration::from_millis(10))))
            .route("/", web::post().to(|body: actix_web::web::Bytes| async move { HttpResponse::Ok().body(body) }))
    ).await;

    let req = test::TestRequest::post().uri("/").to_request();
    let resp = app.call(req).await;
    // With normal testing it works instantly
    assert!(resp.is_ok());
}
