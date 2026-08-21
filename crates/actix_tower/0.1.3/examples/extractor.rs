use actix_tower::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct CreateUser {
    name: String,
    email: String,
}

impl Validator for CreateUser {
    fn validate(&self) -> Result<(), String> {
        not_empty(&self.name, "name")?;
        is_email(&self.email, "email")?;
        Ok(())
    }
}

#[derive(Serialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

// Using AutoJson — no .into_inner() needed
async fn create_user(AutoJson(payload): AutoJson<CreateUser>) -> impl Responder {
    TypedResponse::created(User {
        id: 1,
        name: payload.name,
        email: payload.email,
    })
}

// Using ValidatedJson — validation happens automatically
async fn create_user_validated(
    ValidatedJson(payload): ValidatedJson<CreateUser>,
) -> impl Responder {
    TypedResponse::created(User {
        id: 2,
        name: payload.name,
        email: payload.email,
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/users", web::post().to(create_user))
            .route("/users/validated", web::post().to(create_user_validated))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
