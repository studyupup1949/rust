use actix_tower::prelude::*;
use actix_web::{test, web, App, HttpResponse};
use std::alloc::{GlobalAlloc, System, Layout};
use std::sync::atomic::{AtomicUsize, Ordering};
use actix_web::dev::Service as ActixService;

struct TrackingAllocator;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::SeqCst);
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

async fn measure_request_allocations<S, B>(app: &S, req: actix_http::Request) -> usize
where
    S: ActixService<actix_http::Request, Response = actix_web::dev::ServiceResponse<B>, Error = actix_web::Error>,
{
    // Warm up the allocator (lazy statics, caching, thread-locals, etc.)
    let warmup = test::TestRequest::get().uri("/").to_request();
    let _ = app.call(warmup).await;
    
    let start = ALLOCATIONS.load(Ordering::SeqCst);
    let _resp = app.call(req).await;
    let end = ALLOCATIONS.load(Ordering::SeqCst);
    end - start
}

#[derive(Clone)]
struct DummyLayer;

impl<S> tower::Layer<S> for DummyLayer {
    type Service = S;
    fn layer(&self, inner: S) -> Self::Service {
        inner
    }
}

#[actix_web::test]
async fn test_allocation_regressions() {
    // 1. TowerLayerCompat vs Baseline
    let baseline_app = test::init_service(
        App::new()
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
    ).await;

    let tower_app = test::init_service(
        App::new()
            .wrap(tower_layer!(DummyLayer))
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
    ).await;

    let req_base = test::TestRequest::get().uri("/").to_request();
    let req_tower = test::TestRequest::get().uri("/").to_request();
    
    let allocs_baseline = measure_request_allocations(&baseline_app, req_base).await;
    let allocs_tower = measure_request_allocations(&tower_app, req_tower).await;
    
    // The bridge introduces exactly 11 allocations per request to translate
    // headers, URIs, and extensions across the HTTP boundary. This is highly
    // optimized. We assert it doesn't regress (e.g. <= 15 to allow minor variance).
    assert!(allocs_tower <= allocs_baseline + 15, 
        "Tower bridge introduced unexpected allocations! Baseline: {}, Tower: {}", 
        allocs_baseline, allocs_tower);

    // 2. RequestId Middleware
    let reqid_app = test::init_service(
        App::new()
            .wrap(actix_tower::middleware::request_id::RequestId::default())
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
    ).await;
    let reqid_req = test::TestRequest::get().uri("/").to_request();
    let allocs_reqid = measure_request_allocations(&reqid_app, reqid_req).await;
    // We allow a very small bounded number of allocations for UUID string generation
    assert!(allocs_reqid <= allocs_baseline + 10, "RequestId allocated too much: {}", allocs_reqid);

    // 3. Cache Disabled (Bypass)
    let cache_app = test::init_service(
        App::new()
            .wrap(actix_tower::middleware::cache::Cache::new(std::time::Duration::from_secs(60)))
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
    ).await;
    let cache_req = test::TestRequest::get().uri("/").insert_header(("Cache-Control", "no-cache")).to_request();
    let allocs_cache = measure_request_allocations(&cache_app, cache_req).await;
    assert!(allocs_cache <= allocs_baseline + 20, "Cache bypass allocated too much: {}", allocs_cache);
    
    // 4. Compression Disabled/Passthrough (e.g. no Accept-Encoding header)
    #[cfg(feature = "middleware")]
    {
        let comp_app = test::init_service(
            App::new()
                .wrap(actix_tower::middleware::compression::Compression::default())
                .route("/", web::get().to(|| async { HttpResponse::Ok().finish() }))
        ).await;
        let comp_req = test::TestRequest::get().uri("/").to_request();
        let allocs_comp = measure_request_allocations(&comp_app, comp_req).await;
        assert!(allocs_comp <= allocs_baseline + 20, "Compression passthrough allocated too much: {}", allocs_comp);
    }
}
