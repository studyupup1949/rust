# Actix SQLX TX

Support running SQLX transactions in Actix web framework.

## Behavior

Running SQLX transactions in Actix web framework is not straightforward. This library provides a way to run SQLX transactions in Actix web framework.
Write your code inside the `with_tx` function and return a `ScopedBoxFuture` from it. The `ScopedBoxFuture` will be executed in the transaction context. 
If the `ScopedBoxFuture` returns an `Ok` value, the transaction will be committed. If the `Future` returns an `Err` value, the transaction will be rolled back.

## Usage example with pgsql driver

```rust

use actix_web::{HttpServer, post, web};
use actix_web::web::Data;
use actix_web_sqlx_tx::http::{ok, Response};
use actix_web_sqlx_tx::tx::with_tx;
use chrono::NaiveDateTime;
use scoped_futures::ScopedFutureExt;
use serde::Deserialize;
use sqlx::{PgPool, Postgres, query_as, Transaction};
use sqlx::postgres::PgPoolOptions;

pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub created_at: NaiveDateTime,
    pub organization_id: i32,
}

pub struct Organization {
    pub id: i32,
    pub name: String,
    pub created_at: NaiveDateTime,
}

pub async fn create_organization<'a>(
    name: impl Into<String>,
    transaction: &mut Transaction<'a, Postgres>,
) -> Result<Organization, sqlx::Error> {
    let name = name.into();
    let created_at = chrono::Local::now().naive_utc();
    let organization = query_as!(
        Organization,
        "INSERT INTO organizations (name, created_at) VALUES ($1, $2) RETURNING *",
        name,
        created_at
    )
        .fetch_one(&mut **transaction)
        .await?;
    Ok(organization)
}

pub async fn create_new_user<'a>(
    email: impl Into<String>,
    password: impl Into<String>,
    organization_id: i32,
    transaction: &mut Transaction<'a, Postgres>,
) -> Result<User, sqlx::Error> {
    let email = email.into();
    let password = password.into();
    let created_at = chrono::Local::now().naive_utc();
    let user = query_as!(
        User,
        "INSERT INTO users (organization_id, email, password, created_at) VALUES ($1, $2, $3, $4) RETURNING *",
        organization_id,
        email,
        password,
        created_at,
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
    org_name: String,
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
            // create new org
            let organization =
                create_organization(create_user_request.org_name.clone(), tx).await?;

            //create new user in org
            let user = create_new_user(
                create_user_request.email.clone(),
                create_user_request.password.clone(),
                organization.id,
                tx,
            )
                .await?;

            //if create new user fails for some reason, we can rollback the transaction

            ok(CreateUserResponse {
                message: format!("User {} created", user.email),
            })
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
            .connect("postgres://username:password@pgsql:5432/dbname")
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


// testing your fn

#[cfg(test)]
mod tests {
    use actix_sqlx_tx::tx::tests::with_tx;
    use scoped_futures::ScopedFutureExt;

    use super::*;

    #[actix_rt::test]
    async fn test_create_new_user() {
        let pool = ...;
        // this actix_sqlx_tx::tx::tests::with_tx will be rolled back at end
        with_tx(&pool, |mut tx| {
            async move {
                let email = "someemail".to_string();
                let password = "somepassword".to_string();
                let user = create_new_user(email.clone(), password.clone(), &mut tx)
                    .await
                    .unwrap();
                assert_eq!(user.email, email.clone());
            }
                .scope_boxed()
        })
            .await;
    }
}


```