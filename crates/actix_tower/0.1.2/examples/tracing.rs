use actix_tower::prelude::*;
use actix_web::App;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    HttpServer::new(|| {
        App::new()
            .wrap(RequestId::new())
            .wrap(TracingMiddleware::default())
            .route(
                "/",
                web::get().to(|| async { HttpResponse::Ok().body("Hello with tracing!") }),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
