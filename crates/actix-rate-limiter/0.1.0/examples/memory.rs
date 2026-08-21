use std::sync::Arc;

use actix_rate_limiter::{
    backend::memory::MemoryBackendProvider, middleware::RateLimiterMiddlewareFactory, Limit,
    RateLimiterBuilder,
};
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use tokio::sync::Mutex;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/greeting/{username}")]
async fn greeting(username: web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body(username.into_inner())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let limiter = RateLimiterBuilder::new()
        .add_route("/", Limit { ttl: 5, amount: 1 })
        .add_route("/greeting/[a-zA-Z0-9]*", Limit { ttl: 1, amount: 5 })
        .build();
    let backend = MemoryBackendProvider::default();
    let rate_limiter = RateLimiterMiddlewareFactory::new(limiter, Arc::new(Mutex::new(backend)));

    HttpServer::new(move || {
        App::new()
            .wrap(rate_limiter.clone())
            .service(hello)
            .service(greeting)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
