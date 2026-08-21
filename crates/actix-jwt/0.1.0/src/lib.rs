//! Full-featured JWT authentication middleware for [actix-web].
//!
//! This crate is a Rust port of
//! [`echo-jwt`](https://github.com/LdDl/echo-jwt) (Go implementation, itself a port of
//! [`gin-jwt`](https://github.com/appleboy/gin-jwt) to the Echo framework).
//! It goes far beyond simple token validation - it provides login, logout and
//! refresh handlers, refresh-token rotation with a pluggable store, cookie
//! management, RSA / HMAC signing, RBAC authorizer callback and more.
//!
//! # Feature flags
//!
//! | Flag | Default | Description |
//! |------|---------|-------------|
//! | `redis-store` | off | Enables `RedisRefreshTokenStore` backed by the [`redis`](https://crates.io/crates/redis) crate. |
//!
//! # Quick start
//!
//! ```rust,no_run
//! use std::sync::Arc;
//!
//! use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
//! use actix_jwt::{ActixJwtMiddleware, extract_claims};
//!
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//!     let mut jwt = ActixJwtMiddleware::new();
//!     jwt.key = b"my-secret-key".to_vec();
//!     jwt.authenticator = Some(Arc::new(|_req, body| {
//!         #[derive(serde::Deserialize)]
//!         struct Login { username: String, password: String }
//!         let creds: Login = serde_json::from_slice(body)
//!             .map_err(|_| actix_jwt::JwtError::MissingLoginValues)?;
//!         if creds.username == "admin" && creds.password == "admin" {
//!             Ok(serde_json::json!({"username": creds.username}))
//!         } else {
//!             Err(actix_jwt::JwtError::FailedAuthentication)
//!         }
//!     }));
//!     jwt.init().expect("JWT middleware init");
//!
//!     let jwt = Arc::new(jwt);
//!     let jwt_data = web::Data::new(jwt.clone());
//!
//!     HttpServer::new(move || {
//!         App::new()
//!             .app_data(jwt_data.clone())
//!             .route("/login", web::post().to({
//!                 let j = jwt.clone();
//!                 move |req: HttpRequest, body: web::Bytes| {
//!                     let j = j.clone();
//!                     async move { j.login_handler(&req, &body).await }
//!                 }
//!             }))
//!             .service(
//!                 web::scope("/api")
//!                     .wrap(jwt.middleware())
//!                     .route("/hello", web::get().to(|req: HttpRequest| async move {
//!                         let claims = extract_claims(&req);
//!                         HttpResponse::Ok().json(claims)
//!                     })),
//!             )
//!     })
//!     .bind("127.0.0.1:8080")?
//!     .run()
//!     .await
//! }
//! ```

pub mod core;
pub mod errors;
pub mod middleware;
pub mod store;

pub use core::{RefreshTokenData, Token, TokenStore};
pub use errors::JwtError;
pub use middleware::{
    ActixJwtMiddleware, CookieConfig, JwtAuth, JwtIdentity, JwtPayload, JwtTokenString,
    extract_claims, get_identity, get_token,
};
pub use store::{InMemoryRefreshTokenStore, default_store, new_memory_store};
