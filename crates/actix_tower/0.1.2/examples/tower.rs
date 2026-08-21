use actix_tower::prelude::*;
use actix_web::App;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            // Use tower-http middleware via the Tower bridge
            .wrap(tower_layer!(tower_http::trace::TraceLayer::new_for_http()))
            .wrap(tower_layer!(
                tower_http::compression::CompressionLayer::new()
            ))
            .route(
                "/",
                web::get().to(|| async { HttpResponse::Ok().body("Hello from Tower + Actix!") }),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
