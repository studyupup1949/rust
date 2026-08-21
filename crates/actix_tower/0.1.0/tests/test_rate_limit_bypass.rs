use actix_tower::prelude::*;
use actix_web::{test, web, App, HttpResponse};
use std::time::Duration;

#[actix_web::test]
async fn test_rate_limiter_bypass_prevention() {
    let app = test::init_service(
        App::new()
            .wrap(RateLimit::new(1, Duration::from_secs(60)))
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() })),
    )
    .await;

    // First request should succeed
    let req = test::TestRequest::get()
        .uri("/")
        .insert_header(("x-forwarded-for", "1.1.1.1"))
        .to_request();
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), actix_web::http::StatusCode::OK);

    // Second request with a DIFFERENT x-forwarded-for MUST STILL BE BLOCKED
    // because peer_addr() is the same (None in test context, meaning they share the "unknown" key).
    let req = test::TestRequest::get()
        .uri("/")
        .insert_header(("x-forwarded-for", "2.2.2.2"))
        .to_request();
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), actix_web::http::StatusCode::TOO_MANY_REQUESTS);
}
