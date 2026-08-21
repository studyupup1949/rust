//! Criterion benchmarks for actix_tower middleware performance.
//!
//! # Benchmark groups
//!
//! - `cold_start` — measures the **total round-trip latency** of each stack
//!   (plain Actix, Actix + Tower compat, Actix + built-in middleware).
//!   Each iteration creates a fresh service and processes one request,
//!   so it includes middleware initialisation overhead.
//!
//! - `hot_path` — measures the **per-request translation cost** in isolation.
//!   The service is initialised once and reused across iterations, giving a
//!   cleaner signal on the bridge overhead with a warm allocator.

use actix_tower::prelude::*;
use actix_web::{test, web, App, HttpResponse};
use criterion::{criterion_group, criterion_main, Criterion};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Helpers — build services once and reuse across hot-path iterations
// ---------------------------------------------------------------------------

async fn make_plain_service() -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    test::init_service(App::new().route(
        "/",
        web::get().to(|| async { HttpResponse::Ok().body("hello") }),
    ))
    .await
}

async fn make_tower_service() -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    test::init_service(
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
    .await
}

async fn make_builtin_service() -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    test::init_service(
        App::new()
            .wrap(Timeout::new(Duration::from_secs(30)))
            .route(
                "/",
                web::get().to(|| async { HttpResponse::Ok().body("hello") }),
            ),
    )
    .await
}

// ---------------------------------------------------------------------------
// Cold-start benchmarks (include service initialisation)
// ---------------------------------------------------------------------------

async fn plain_actix_cold() {
    let app = make_plain_service().await;
    let req = test::TestRequest::get().uri("/").to_request();
    let _resp = test::call_service(&app, req).await;
}

async fn actix_with_tower_cold() {
    let app = make_tower_service().await;
    let req = test::TestRequest::get().uri("/").to_request();
    let _resp = test::call_service(&app, req).await;
}

async fn actix_with_builtin_cold() {
    let app = make_builtin_service().await;
    let req = test::TestRequest::get().uri("/").to_request();
    let _resp = test::call_service(&app, req).await;
}

// ---------------------------------------------------------------------------
// Benchmark groups
// ---------------------------------------------------------------------------

fn bench_overhead(c: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let mut group = c.benchmark_group("cold_start");
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("plain_actix", |b| {
        b.to_async(&runtime).iter(plain_actix_cold);
    });

    group.bench_function("actix_with_tower_compat", |b| {
        b.to_async(&runtime).iter(actix_with_tower_cold);
    });

    group.bench_function("actix_with_builtin_middleware", |b| {
        b.to_async(&runtime).iter(actix_with_builtin_cold);
    });

    group.finish();
}

fn bench_hot_path(c: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    // Pre-build all services — only per-request cost is measured.
    let plain_svc = runtime.block_on(make_plain_service());
    let tower_svc = runtime.block_on(make_tower_service());
    let builtin_svc = runtime.block_on(make_builtin_service());

    let mut group = c.benchmark_group("hot_path");
    group.measurement_time(Duration::from_secs(10));
    // Use a tight sample size to get stable ns-level timing.
    group.sample_size(500);
    // Explicitly track throughput (elements/sec)
    group.throughput(criterion::Throughput::Elements(1));

    group.bench_function("plain_actix", |b| {
        b.to_async(&runtime).iter(|| async {
            let req = test::TestRequest::get().uri("/").to_request();
            test::call_service(&plain_svc, req).await
        });
    });

    group.bench_function("tower_compat_bridge", |b| {
        b.to_async(&runtime).iter(|| async {
            let req = test::TestRequest::get().uri("/").to_request();
            test::call_service(&tower_svc, req).await
        });
    });

    group.bench_function("builtin_timeout_middleware", |b| {
        b.to_async(&runtime).iter(|| async {
            let req = test::TestRequest::get().uri("/").to_request();
            test::call_service(&builtin_svc, req).await
        });
    });

    group.finish();
}

criterion_group!(benches, bench_overhead, bench_hot_path);
criterion_main!(benches);
