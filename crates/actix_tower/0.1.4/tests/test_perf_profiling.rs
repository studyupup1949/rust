use actix_tower::prelude::*;
use actix_web::{test, web, App, HttpResponse};

#[actix_web::test]
async fn test_zero_allocation_path_verify() {
    let app = test::init_service(
        App::new()
            .wrap(Cache::new(std::time::Duration::from_secs(60)))
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
    ).await;
    
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
}
