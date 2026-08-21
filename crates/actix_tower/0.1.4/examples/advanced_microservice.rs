#![allow(deprecated)]

use actix_tower::compat::tower::TowerLayerCompat;
use actix_tower::middleware::RateLimit;
use actix_tower::prelude::*;
use actix_web::{web, App, FromRequest, HttpRequest, HttpResponse, HttpServer};
use std::future::{ready, Ready};
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

// ===========================================================================
// 1. Authentication Extractor
// ===========================================================================

/// A mock user extracted from an Authorization header.
#[derive(Debug, Clone)]
struct AuthenticatedUser {
    username: String,
}

impl FromRequest for AuthenticatedUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        // Mock JWT validation: normally you'd parse `Authorization: Bearer <token>`
        if let Some(auth) = req.headers().get("authorization") {
            if let Ok(token) = auth.to_str() {
                if token == "Bearer secret-admin-token" {
                    return ready(Ok(AuthenticatedUser {
                        username: "admin".to_string(),
                    }));
                }
            }
        }
        
        // Return our standardized API error envelope (returns a 401 JSON response)
        ready(Err(ApiError::unauthorized(
            "Missing or invalid Authorization token",
        )))
    }
}

// ===========================================================================
// 2. Request & Response Payload Types
// ===========================================================================

#[derive(serde::Deserialize)]
struct CreateResourceRequest {
    name: String,
    data: String,
}

#[derive(serde::Serialize)]
struct ResourceResponse {
    id: String,
    name: String,
    data: String,
    created_by: String,
}

// ===========================================================================
// 3. Handlers
// ===========================================================================

/// A protected endpoint that requires authentication and validates the JSON payload.
/// Notice how ergonomic this is: no `.into_inner()` calls required.
async fn create_resource(
    user: AuthenticatedUser,
    body: AutoJson<CreateResourceRequest>,
) -> actix_web::Result<HttpResponse> {
    // Simulate some database latency that the Tower TimeoutLayer might catch
    tokio::time::sleep(Duration::from_millis(100)).await;

    let response = ResourceResponse {
        id: uuid::Uuid::new_v4().to_string(),
        name: body.name.clone(), // Access fields directly!
        data: body.data.clone(),
        created_by: user.username,
    };

    Ok(HttpResponse::Created().json(response))
}

/// A simple health check endpoint.
async fn health_check() -> &'static str {
    "OK"
}

// ===========================================================================
// 4. Application Assembly
// ===========================================================================

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging so the Tower TraceLayer has somewhere to output to
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("Starting advanced microservice on http://127.0.0.1:8080");
    println!("Try: curl -H 'Authorization: Bearer secret-admin-token' -H 'Content-Type: application/json' -d '{{\"name\": \"test\", \"data\": \"hello\"}}' http://127.0.0.1:8080/resource");

    HttpServer::new(|| {
        App::new()
            // 1. Tower Trace (Logging) - Outermost layer so it logs everything
            .wrap(TowerLayerCompat::new(TraceLayer::new_for_http()))
            
            // 2. Tower Timeout - Abort requests taking longer than 2 seconds
            // This protects the Actix worker from being tied up forever
            .wrap(TowerLayerCompat::new(TimeoutLayer::new(Duration::from_secs(2))))
            
            // 3. Native Rate Limit - 5 requests per second per IP
            // We put this BEFORE authentication so attackers can't spam expensive password hashes
            .wrap(RateLimit::new(5, Duration::from_secs(1)))
            
            // 4. Routes
            .route("/health", web::get().to(health_check))
            .route("/resource", web::post().to(create_resource))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
