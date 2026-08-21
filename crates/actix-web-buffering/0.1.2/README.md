[![Documentation](https://docs.rs/actix-web-buffering/badge.svg)](https://docs.rs/actix-web-buffering)
[![crates.io](https://img.shields.io/crates/v/actix-web-buffering.svg)](https://crates.io/crates/actix-web-buffering)

# actix-web-buffering

Request/Response body buffering with support spooled to a temp file on disk

Use it in actix-web middleware. For example this is used at [actix-web-detached-jws-middleware](https://crates.io/crates/actix-web-detached-jws-middleware)

## Example:
```rust
use std::{
    cell::RefCell,
    pin::Pin,
    rc::Rc,
    sync::Arc,
    task::{Context, Poll},
};

use actix_service::Transform;
use actix_web::{
    dev::{Body, Service, ServiceRequest, ServiceResponse},
    web,
    web::BytesMut,
    App, Error, HttpMessage, HttpResponse, HttpServer, Responder,
};
use actix_web_buffering::{
    enable_request_buffering, enable_response_buffering, FileBufferingStreamWrapper,
};
use futures::{
    future::{ok, Ready},
    stream::StreamExt,
    Future, FutureExt,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let wrapper = FileBufferingStreamWrapper::new()
        .tmp_dir(std::env::temp_dir())
        .threshold(1024 * 30)
        .produce_chunk_size(1024 * 30)
        .buffer_limit(Some(1024 * 30 * 10));

    let wrapper = Arc::new(wrapper);

    HttpServer::new(move || {
        let r1 = Arc::clone(&wrapper);
        let r2 = Arc::clone(&wrapper);
        App::new()
            .wrap(Example(r1))
            .wrap(Example(r2))
            .service(web::resource("/").route(web::post().to(echo)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

struct Example(Arc<FileBufferingStreamWrapper>);

impl<S> Transform<S> for Example
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<Body>, Error = Error> + 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<Body>;
    type Error = Error;
    type InitError = ();
    type Transform = ExampleMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ExampleMiddleware {
            service: Rc::new(RefCell::new(service)),
            wrapper: Arc::clone(&self.0),
        })
    }
}
pub struct ExampleMiddleware<S> {
    service: Rc<RefCell<S>>,
    wrapper: Arc<FileBufferingStreamWrapper>,
}

impl<S> Service for ExampleMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<Body>, Error = Error> + 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<Body>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
        let mut svc = self.service.clone();
        let wrapper = self.wrapper.clone();

        async move {
            enable_request_buffering(&wrapper, &mut req);

            let mut stream = req.take_payload();
            let mut body = BytesMut::new();
            while let Some(chunk) = stream.next().await {
                body.extend_from_slice(&chunk.unwrap());
            }
            req.set_payload(stream);
            println!("request body: {:?}", body);

            let svc_res = svc.call(req).await?;

            let mut svc_res = enable_response_buffering(&wrapper, svc_res);

            let mut stream = svc_res.take_body();
            let mut body = BytesMut::new();
            while let Some(chunk) = stream.next().await {
                body.extend_from_slice(&chunk.unwrap());
            }
            let svc_res = svc_res.map_body(|_, _| stream);
            println!("response body: {:?}", body);

            Ok(svc_res)
        }
        .boxed_local()
    }
}
```
