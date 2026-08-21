use actix_tower::prelude::*;
use actix_web::App;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Greeting {
    name: String,
}

async fn hello(AutoQuery(greeting): AutoQuery<Greeting>) -> impl Responder {
    ApiResponse::ok(serde_json::json!({
        "message": format!("Hello, {}!", greeting.name)
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(RequestId::new())
            .wrap(Timeout::new(std::time::Duration::from_secs(30)))
            .service(web::scope("/api").route("/hello", web::get().to(hello)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
