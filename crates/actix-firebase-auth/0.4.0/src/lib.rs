//! # actix-firebase-auth
//!
//! This crate provides Firebase JWT verification for the `actix-web` framework,
//! using Google's public JWKs to validate ID tokens issued by Firebase Authentication.
//!
//! ## Example
//!
//! ```no_run
//! use actix_web::{web, App, HttpServer, HttpResponse, Responder};
//! use actix_firebase_auth::{FirebaseAuth, FirebaseUser, Result};
//!
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//!     let auth = FirebaseAuth::new("your-project-id").await.unwrap(); // Don't forget to handle this error
//!
//!     HttpServer::new(move || {
//!         App::new()
//!             .app_data(web::Data::new(auth.clone()))
//!             .route("/profile", web::get().to(get_profile))
//!     })
//!     .bind(("127.0.0.1", 8080))?
//!     .run()
//!     .await
//! }
//!
//! async fn get_profile(
//!     auth: web::Data<FirebaseAuth>,
//!     user: FirebaseUser,
//! ) -> HttpResponse {
//!     HttpResponse::Ok().json(user)
//! }
//! ```

mod client;
mod error;
mod impls;
mod jwk;
mod user;

pub use client::*;
pub use error::*;
pub use user::*;
