// Run: cargo run --example oauth_sso
//
// OAuth SSO example with Google and GitHub providers.
// This is a conceptual port showing the pattern - you'll need to add
// an OAuth2 crate (e.g. `oauth2`) to your dependencies for a real implementation.
//
// Environment variables:
//   GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET
//   GITHUB_CLIENT_ID, GITHUB_CLIENT_SECRET
//   JWT_SECRET_KEY (optional, defaults to a dev key)
//   PORT (optional, defaults to 8000)
//
// The key pattern demonstrated here:
// 1. OAuth callback receives user info from provider
// 2. Use `token_generator()` to create JWT tokens for the user
// 3. Set cookies via `set_cookie()` / refresh_cookie_config
// 4. Protected routes use the standard JWT middleware

use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;

use actix_jwt::{ActixJwtMiddleware, JwtError, extract_claims, get_identity};
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use serde_json::{Value, json};

const IDENTITY_KEY: &str = "id";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let bind_addr = format!("0.0.0.0:{}", port);

    let secret = env::var("JWT_SECRET_KEY")
        .unwrap_or_else(|_| "default-secret-key-change-in-production".to_string());

    let mut jwt = ActixJwtMiddleware::new();
    jwt.realm = "oauth-sso-zone".to_string();
    jwt.key = secret.into_bytes();
    jwt.timeout = Duration::from_secs(3600);
    jwt.max_refresh = Duration::from_secs(86400);
    jwt.identity_key = IDENTITY_KEY.to_string();
    jwt.token_lookup = "header:Authorization, query:token, cookie:jwt".to_string();
    jwt.send_cookie = true;
    jwt.secure_cookie = false; // Set to true in production with HTTPS
    jwt.cookie_http_only = true;
    jwt.cookie_max_age = Duration::from_secs(3600);
    jwt.send_authorization = true;

    jwt.authenticator = Some(Arc::new(
        |_req: &HttpRequest, _body: &[u8]| {
            Box::pin(async { Err(JwtError::MissingLoginValues) })
        },
    ));

    jwt.payload_func = Some(Arc::new(|data: &Value| {
        let mut claims = HashMap::new();
        for key in &["id", "email", "name", "provider", "avatar"] {
            if let Some(v) = data.get(key) {
                claims.insert(key.to_string(), v.clone());
            }
        }
        claims
    }));

    jwt.identity_handler = Arc::new(|req: &HttpRequest| {
        let claims = extract_claims(req);
        Some(json!({
            "id": claims.get(IDENTITY_KEY),
            "email": claims.get("email"),
            "name": claims.get("name"),
            "provider": claims.get("provider"),
        }))
    });

    jwt.authorizer = Arc::new(|_req: &HttpRequest, data: &Value| {
        // All authenticated OAuth users are authorized
        data.get("id").is_some()
    });

    jwt.init().expect("JWT middleware init failed");
    let jwt_arc = Arc::new(jwt);

    println!("Listening on {}", bind_addr);
    println!("OAuth SSO Example (conceptual)");
    println!("  POST /oauth/callback - simulate OAuth callback");
    println!("  GET  /api/profile    - protected profile endpoint");

    HttpServer::new(move || {
        let jwt_data = web::Data::new(jwt_arc.clone());

        App::new()
            .app_data(jwt_data.clone())
            .route("/", web::get().to(index))
            // Simulate OAuth callback (in production, this would be the OAuth redirect handler)
            .route("/oauth/callback", web::post().to(oauth_callback))
            .route("/refresh", web::post().to(refresh))
            // Protected routes
            .service(
                web::scope("/api")
                    .wrap(jwt_arc.middleware())
                    .route("/profile", web::get().to(profile))
                    .route("/logout", web::post().to(logout)),
            )
    })
    .bind(&bind_addr)?
    .run()
    .await
}

/// Simulates an OAuth callback. In production, this would:
/// 1. Receive the authorization code from the OAuth provider
/// 2. Exchange it for an access token
/// 3. Fetch user info from the provider
/// 4. Generate JWT tokens
///
/// For this example, send a POST with user info:
/// curl -X POST http://localhost:8000/oauth/callback \
///   -H 'Content-Type: application/json' \
///   -d '{"id":"google_123","email":"user@example.com","name":"Test User","provider":"google"}'
async fn oauth_callback(jwt: web::Data<Arc<ActixJwtMiddleware>>, body: web::Bytes) -> HttpResponse {
    let user_data: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => {
            return HttpResponse::BadRequest().json(json!({"error": "Invalid user data"}));
        }
    };

    // Generate JWT tokens using token_generator
    match jwt.token_generator(&user_data).await {
        Ok(token_pair) => HttpResponse::Ok().json(json!({
            "access_token": token_pair.access_token,
            "token_type": token_pair.token_type,
            "refresh_token": token_pair.refresh_token,
            "expires_at": token_pair.expires_at,
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to generate token: {}", e),
        })),
    }
}

async fn refresh(
    jwt: web::Data<Arc<ActixJwtMiddleware>>,
    req: HttpRequest,
    body: web::Bytes,
) -> HttpResponse {
    jwt.refresh_handler(&req, &body).await
}

async fn logout(
    jwt: web::Data<Arc<ActixJwtMiddleware>>,
    req: HttpRequest,
    body: web::Bytes,
) -> HttpResponse {
    jwt.logout_handler(&req, &body).await
}

async fn index() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "message": "OAuth SSO Example with actix-jwt",
        "endpoints": {
            "oauth_callback": "POST /oauth/callback (simulate OAuth)",
            "profile": "GET /api/profile (requires JWT)",
            "refresh": "POST /refresh",
            "logout": "POST /api/logout (requires JWT)",
        },
    }))
}

async fn profile(req: HttpRequest) -> HttpResponse {
    let claims = extract_claims(&req);
    let identity = get_identity(&req);
    HttpResponse::Ok().json(json!({
        "code": 200,
        "user": identity,
        "claims": {
            "id": claims.get(IDENTITY_KEY),
            "email": claims.get("email"),
            "name": claims.get("name"),
            "provider": claims.get("provider"),
            "avatar": claims.get("avatar"),
        },
    }))
}
