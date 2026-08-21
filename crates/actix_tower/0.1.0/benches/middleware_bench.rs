use actix_tower::prelude::*;
use actix_web::{test, web, App, HttpResponse};
use criterion::{criterion_group, criterion_main, Criterion};
use std::time::Duration;

async fn plain_actix() {
    let app = test::init_service(App::new().route(
        "/",
        web::get().to(|| async { HttpResponse::Ok().body("hello") }),
    ))
    .await;
    let req = test::TestRequest::get().uri("/").to_request();
    let _resp = test::call_service(&app, req).await;
}

async fn actix_with_tower() {
    let app = test::init_service(
        App::new()
            .wrap(TowerLayerCompat::new(
                tower_http::timeout::TimeoutLayer::with_status_code(
                    http::StatusCode::GATEWAY_TIMEOUT,
                    Duration::from_secs(30),
                ),
            ))
            .route(
                "/",
                web::get().to(|| async { HttpResponse::Ok().body("hello") }),
            ),
    )
    .await;
    let req = test::TestRequest::get().uri("/").to_request();
    let _resp = test::call_service(&app, req).await;
}

async fn actix_with_builtin() {
    let app = test::init_service(
        App::new()
            .wrap(Timeout::new(Duration::from_secs(30)))
            .route(
                "/",
                web::get().to(|| async { HttpResponse::Ok().body("hello") }),
            ),
    )
    .await;
    let req = test::TestRequest::get().uri("/").to_request();
    let _resp = test::call_service(&app, req).await;
}

fn bench_middleware(c: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    c.bench_function("plain_actix", |b| {
        b.to_async(&runtime).iter(plain_actix);
    });

    c.bench_function("actix_with_tower_compat", |b| {
        b.to_async(&runtime).iter(actix_with_tower);
    });

    c.bench_function("actix_with_builtin_middleware", |b| {
        b.to_async(&runtime).iter(actix_with_builtin);
    });
}

criterion_group!(benches, bench_middleware);
criterion_main!(benches);
