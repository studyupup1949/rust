use actix_service::{Service, Transform};
use actix_tower::prelude::*;
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    test, web, App, Error, HttpResponse,
};
use std::{
    future::{ready, Ready},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Context, Poll},
};

// A mock Actix middleware that exerts backpressure
#[derive(Clone)]
struct BackpressureMiddleware {
    is_ready: Arc<AtomicBool>,
}

impl<S, B> Transform<S, ServiceRequest> for BackpressureMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = BackpressureService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(BackpressureService {
            service,
            is_ready: self.is_ready.clone(),
        }))
    }
}

struct BackpressureService<S> {
    service: S,
    is_ready: Arc<AtomicBool>,
}

impl<S, B> Service<ServiceRequest> for BackpressureService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = S::Future;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.is_ready.load(Ordering::SeqCst) {
            self.service.poll_ready(cx)
        } else {
            // Force pending to exert backpressure
            Poll::Pending
        }
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        self.service.call(req)
    }
}

#[actix_web::test]
async fn test_tower_backpressure_propagation() {
    let is_ready = Arc::new(AtomicBool::new(false));

    // Wrap the app with a Tower middleware, and place the Backpressure middleware INSIDE it.
    // If Tower layer properly propagates poll_ready, the whole app will be pending.
    let app = test::init_service(
        App::new()
            // Tower middleware on the outside
            .wrap(tower_layer!(tower_http::timeout::TimeoutLayer::new(
                std::time::Duration::from_secs(10)
            )))
            // Actix backpressure middleware on the inside
            .wrap(BackpressureMiddleware {
                is_ready: is_ready.clone(),
            })
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() })),
    )
    .await;

    let mut cx = std::task::Context::from_waker(futures_util::task::noop_waker_ref());
    
    // Test that the app is currently PENDING (backpressure is successfully propagated through Tower)
    let poll = actix_service::Service::poll_ready(&app, &mut cx);
    assert!(poll.is_pending(), "Backpressure was not propagated!");

    // Release backpressure
    is_ready.store(true, Ordering::SeqCst);
    
    // Now it should be READY
    let poll = actix_service::Service::poll_ready(&app, &mut cx);
    assert!(poll.is_ready(), "Service did not become ready!");
}
