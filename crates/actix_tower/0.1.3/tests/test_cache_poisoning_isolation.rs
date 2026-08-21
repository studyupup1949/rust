use actix_tower::prelude::*;
use actix_web::{test, web, App, HttpResponse};
use std::time::Duration;

#[actix_web::test]
async fn test_cache_poisoning_isolation() {
    let app = test::init_service(
        App::new()
            .wrap(Cache::new(Duration::from_secs(60)))
            .route("/", web::get().to(|req: actix_web::HttpRequest| async move { 
                let host = req.connection_info().host().to_string();
                HttpResponse::Ok().body(format!("Host: {}", host)) 
            })),
    )
    .await;

    // First request to admin domain
    let req1 = test::TestRequest::get()
        .uri("/")
        .insert_header(("Host", "admin.example.com"))
        .to_request();
    let res1 = test::call_service(&app, req1).await;
    let body1 = test::read_body(res1).await;
    assert_eq!(body1, "Host: admin.example.com");

    // Second request to public domain. It must NOT hit the cache for admin!
    let req2 = test::TestRequest::get()
        .uri("/")
        .insert_header(("Host", "public.example.com"))
        .to_request();
    let res2 = test::call_service(&app, req2).await;
    let body2 = test::read_body(res2).await;
    
    // If cache poisoning was present, body2 would be "Host: admin.example.com"
    assert_eq!(body2, "Host: public.example.com");
}
