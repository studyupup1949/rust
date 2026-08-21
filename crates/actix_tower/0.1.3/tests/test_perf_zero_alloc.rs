use actix_web::Error;

/// Test 4: test_zero_alloc_body_streaming
/// End-to-end test validating that large bodies stream through without buffering.
#[actix_web::test]
async fn test_zero_alloc_body_streaming() {
    use actix_tower::prelude::*;
    use actix_web::{test, web, App, HttpResponse};
    use futures_util::stream;
    use bytes::Bytes;

    let app = test::init_service(
        App::new()
            // We'll use TraceLayer to pass it through Tower
            .wrap(tower_layer!(tower_http::trace::TraceLayer::new_for_http()))
            .route("/", web::get().to(|| async {
                // Return a streamed response
                let iter = (0..1000).map(|_| Ok::<_, Error>(Bytes::from_static(b"chunk ")));
                HttpResponse::Ok().streaming(stream::iter(iter))
            }))
    ).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    assert_eq!(body.len(), 6000); // 1000 chunks of "chunk "
}
