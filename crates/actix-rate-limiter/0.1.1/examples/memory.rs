use std::sync::Arc;

use actix_rate_limiter::{
    backend::memory::MemoryBackendProvider,
    limit::{Limit, LimitBuilder},
    limiter::RateLimiterBuilder,
    middleware::RateLimiterMiddlewareFactory,
    route::RouteBuilder,
};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use tokio::sync::Mutex;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/greeting/{username}")]
async fn greeting(username: web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body(username.into_inner())
}

#[post("/greeting/{username}")]
async fn post_greeting(username: web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body(format!("Hello, {}!", username.into_inner()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let limiter = RateLimiterBuilder::new()
        .add_route(RouteBuilder::new().set_path("/").build(), Limit::default())
        .add_route(
            RouteBuilder::new()
                .set_method("POST")
                .set_path("/greeting/[a-zA-Z0-9]*")
                .enable_regex()
                .build(),
            LimitBuilder::new().set_ttl(5).set_amount(1).build(),
        )
        .add_route(
            RouteBuilder::new()
                .set_method("GET")
                .set_path("/greeting/[a-zA-Z0-9]*")
                .enable_regex()
                .build(),
            Limit::default(),
        )
        .build();

    let backend = MemoryBackendProvider::default();
    let rate_limiter = RateLimiterMiddlewareFactory::new(limiter, Arc::new(Mutex::new(backend)));

    HttpServer::new(move || {
        App::new()
            .wrap(rate_limiter.clone())
            .service(hello)
            .service(greeting)
            .service(post_greeting)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
