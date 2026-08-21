# Actix SQLX TX

Support running SQLX transactions in Actix web framework.

## Behavior

Running SQLX transactions in Actix web framework is not straightforward. This library provides a way to run SQLX transactions in Actix web framework.
Write your code inside the `with_tx` function and return a `ScopedBoxFuture` from it. The `ScopedBoxFuture` will be executed in the transaction context. 
If the `ScopedBoxFuture` returns an `Ok` value, the transaction will be committed. If the `Future` returns an `Err` value, the transaction will be rolled back.

## Usage example with pgsql driver

```rust

use actix_sqlx_tx::http::{HttpResponse, Response};
use actix_sqlx_tx::tx::with_tx;
use actix_web::{HttpServer, post, web};
use actix_web::web::Data;
use chrono::NaiveDateTime;
use scoped_futures::ScopedFutureExt;
use serde::Deserialize;
use sqlx::{PgPool, Postgres, query_as, Transaction};
use sqlx::postgres::PgPoolOptions;

#[allow(warnings)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub created_at: NaiveDateTime,
}

pub async fn create_new_user<'a>(
    email: impl Into<String>,
    password: impl Into<String>,
    transaction: &mut Transaction<'a, Postgres>,
) -> Result<User, sqlx::Error> {
    let email = email.into();
    let password = password.into();
    let created_at = chrono::Local::now().naive_utc();
    let user = query_as!(
        User,
        "INSERT INTO users (email, password, created_at) VALUES ($1, $2, $3) RETURNING *",
        email,
        password,
        created_at
    )
    .fetch_one(&mut **transaction)
    .await?;
    Ok(user)
}

#[derive(serde::Serialize)]
struct CreateUserResponse {
    message: String,
}

#[derive(Deserialize)]
struct CreateUserRequest {
    email: String,
    password: String,
}

#[post("/users")]
async fn create_user(
    create_user_request: web::Json<CreateUserRequest>,
    pool: Data<PgPool>,
) -> Response {
    with_tx(&pool, |tx| {
        async move {
            match create_new_user(
                create_user_request.email.clone(),
                create_user_request.password.clone(),
                tx,
            )
            .await
            {
                Ok(user) => Ok(HttpResponse::Ok().json(CreateUserResponse {
                    message: format!("User {} created", user.email),
                })),
                Err(e) => Ok(HttpResponse::BadRequest().json(CreateUserResponse {
                    message: format!("Error: {}", e),
                })),
            }
        }
        .scope_boxed()
    })
    .await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = Data::new(
        PgPoolOptions::new()
            .max_connections(10)
            .connect("postgres://user:passws@pgsql:5432/db")
            .await
            .expect("Failed to create pool"),
    );

    HttpServer::new(move || {
        actix_web::App::new()
            .app_data(pool.clone())
            .service(create_user)
    })
    .bind(("0.0.0.0", 9091))
    .unwrap()
    .run()
    .await
}



```