//! Minimal Actix Web example demonstrating [`FirebaseUser`] extractor.
//!
//! This server exposes two endpoints:
//! - `/protected`: Requires a valid Firebase ID token and returns the decoded user info.
//! - `/whoami`: Returns user info if authenticated, or `"Anonymous"` otherwise.
//!
//! Firebase project ID can be set via the `FIREBASE_PROJECT_ID` environment variable,
//! otherwise it falls back to "your-project-id" for testing purposes.

use actix_firebase_auth::{FirebaseAuth, FirebaseUser};
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Read project ID from environment variable or fallback to a default for dev
    let project_id = env::var("FIREBASE_PROJECT_ID")
        .unwrap_or_else(|_| "your-project-id".to_string());

    // Initialize Firebase Auth client (fetches public keys etc.)
    let auth = match FirebaseAuth::new(&project_id).await {
        Ok(auth) => auth,
        Err(e) => {
            eprintln!("Failed to initialize FirebaseAuth: {e}");
            std::process::exit(1);
        }
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(auth.clone()))
            .service(protected)
            .service(whoami)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

// Protected route — requires a valid Firebase ID token
#[get("/protected")]
async fn protected(user: FirebaseUser) -> impl Responder {
    HttpResponse::Ok().json(user)
}

// Returns the authenticated user's info, or null if unauthenticated
#[get("/whoami")]
async fn whoami(user: Option<FirebaseUser>) -> impl Responder {
    match user {
        Some(u) => HttpResponse::Ok().json(u),
        None => HttpResponse::Ok().body("Anonymous"),
    }
}
