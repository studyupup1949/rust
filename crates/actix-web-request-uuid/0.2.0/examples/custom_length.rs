use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web_request_uuid::{get_current_request_id, RequestIDMiddleware};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server with custom request ID configurations...");

    HttpServer::new(|| {
        App::new()
            // Default configuration (36 characters UUID)
            .service(
                web::scope("/default")
                    .wrap(RequestIDMiddleware::new())
                    .route("", web::get().to(handler)),
            )
            // Custom length configuration (16 characters)
            .service(
                web::scope("/short")
                    .wrap(RequestIDMiddleware::new().with_id_length(16))
                    .route("", web::get().to(handler)),
            )
            // Custom length configuration (8 characters)
            .service(
                web::scope("/tiny")
                    .wrap(RequestIDMiddleware::new().with_id_length(8))
                    .route("", web::get().to(handler)),
            )
            // Full UUID format
            .service(
                web::scope("/full")
                    .wrap(RequestIDMiddleware::new().with_full_uuid())
                    .route("", web::get().to(handler)),
            )
            // Simple UUID format (no hyphens)
            .service(
                web::scope("/simple")
                    .wrap(RequestIDMiddleware::new().with_simple_uuid())
                    .route("", web::get().to(handler)),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn handler() -> HttpResponse {
    let request_id = get_current_request_id().unwrap_or_else(|| "unknown".to_string());
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Hello!",
        "request_id": request_id,
        "id_length": request_id.len()
    }))
}
