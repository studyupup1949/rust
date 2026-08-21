use actix_tower::prelude::*;
use actix_web::{App, HttpRequest};

#[derive(Clone)]
struct BearerAuth;

impl AuthExtractor for BearerAuth {
    type Principal = String;

    fn extract(&self, req: &HttpRequest) -> Option<String> {
        req.headers()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .map(|s| s.to_string())
    }
}

async fn protected(req: HttpRequest) -> impl Responder {
    let user = req.get_extension::<String>();
    HttpResponse::Ok().json(serde_json::json!({
        "user": user
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(Authentication::new(BearerAuth))
            .route("/me", web::get().to(protected))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
