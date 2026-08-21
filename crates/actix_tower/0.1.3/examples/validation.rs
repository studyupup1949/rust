use actix_tower::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Pagination {
    page: u32,
    per_page: u32,
}

impl Validator for Pagination {
    fn validate(&self) -> Result<(), String> {
        in_range(self.page, 1, 1000, "page")?;
        in_range(self.per_page, 1, 100, "per_page")?;
        Ok(())
    }
}

async fn list_items(ValidatedQuery(pagination): ValidatedQuery<Pagination>) -> impl Responder {
    ApiResponse::ok(serde_json::json!({
        "page": pagination.page,
        "per_page": pagination.per_page,
        "items": [],
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/items", web::get().to(list_items)))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
