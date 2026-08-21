use actix_tower::prelude::*;
use actix_web::{test, web, App, HttpResponse};
use std::time::Duration;
use futures_util::stream;

#[actix_web::test]
async fn test_cache_oom_vulnerability() {
    let app = test::init_service(
        App::new()
            .wrap(Cache::new(Duration::from_secs(60)))
            .route(
                "/",
                web::get().to(|| async {
                    // A stream that yields 10MB of data, despite saying 10 bytes
                    let mut count = 0;
                    let iter = std::iter::from_fn(move || {
                        if count < 10 {
                            count += 1;
                            Some(Ok::<_, actix_web::Error>(actix_web::web::Bytes::from(vec![0u8; 1024 * 1024])))
                        } else {
                            None
                        }
                    });
                    let stream = stream::iter(iter);
                    
                    HttpResponse::Ok()
                        .insert_header(("content-length", "10"))
                        .streaming(stream)
                }),
            ),
    )
    .await;

    let req = test::TestRequest::get().uri("/").to_request();
    // Use try_call_service so we don't panic on error
    let res = test::try_call_service(&app, req).await;
    
    assert!(res.is_err(), "Cache middleware must return an error when stream exceeds limit");
    
    let err = res.err().unwrap();
    // It should be a 413 Payload Too Large
    assert_eq!(err.error_response().status(), actix_web::http::StatusCode::PAYLOAD_TOO_LARGE);
}
