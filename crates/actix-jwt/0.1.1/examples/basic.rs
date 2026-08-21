use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;

use actix_jwt::{ActixJwtMiddleware, JwtError, extract_claims, get_identity};
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use serde::Deserialize;
use serde_json::{Value, json};

const IDENTITY_KEY: &str = "id";
const USER_ADMIN: &str = "admin";

#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let bind_addr = format!("0.0.0.0:{}", port);

    // Build the JWT middleware
    let mut jwt = ActixJwtMiddleware::new();
    jwt.realm = "test zone".to_string();
    jwt.key = b"secret key".to_vec();
    jwt.timeout = Duration::from_secs(3600);
    jwt.max_refresh = Duration::from_secs(3600);
    jwt.identity_key = IDENTITY_KEY.to_string();
    jwt.token_lookup = "header:Authorization, query:token, cookie:jwt".to_string();
    jwt.token_head_name = "Bearer".to_string();

    // Authenticator: validate username/password, return user data as JSON Value
    jwt.authenticator = Some(Arc::new(|_req: &HttpRequest, body: &[u8]| {
        let login: LoginRequest =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;

        if (login.username == USER_ADMIN && login.password == USER_ADMIN)
            || (login.username == "test" && login.password == "test")
        {
            Ok(json!({
                "user_name": login.username,
                "first_name": "Wu",
                "last_name": "Bo-Yi",
            }))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));

    // PayloadFunc: extract identity claims from user data returned by authenticator
    jwt.payload_func = Some(Arc::new(|data: &Value| {
        let mut claims = HashMap::new();
        if let Some(user_name) = data.get("user_name") {
            claims.insert(IDENTITY_KEY.to_string(), user_name.clone());
        }
        claims
    }));

    // IdentityHandler: reconstruct user data from JWT claims stored in request extensions
    jwt.identity_handler = Arc::new(|req: &HttpRequest| {
        let claims = extract_claims(req);
        claims.get(IDENTITY_KEY).cloned()
    });

    // Authorizer: only allow the "admin" user to access protected routes
    jwt.authorizer = Arc::new(|_req: &HttpRequest, data: &Value| data.as_str() == Some(USER_ADMIN));

    // Unauthorized response
    jwt.unauthorized = Arc::new(|_req: &HttpRequest, code: u16, message: &str| {
        HttpResponse::build(
            actix_web::http::StatusCode::from_u16(code)
                .unwrap_or(actix_web::http::StatusCode::UNAUTHORIZED),
        )
        .json(json!({
            "code": code,
            "message": message,
        }))
    });

    // Logout response: show logged-out user info
    jwt.logout_response = Arc::new(|req: &HttpRequest| {
        let claims = extract_claims(req);
        let identity = get_identity(req);

        let mut response = json!({
            "code": 200,
            "message": "Successfully logged out",
        });

        if let Some(user_id) = claims.get(IDENTITY_KEY) {
            response["logged_out_user"] = user_id.clone();
        }
        if let Some(user) = identity {
            response["user_info"] = user;
        }

        HttpResponse::Ok().json(response)
    });

    // Initialize (validates config and prepares keys)
    jwt.init().expect("JWT middleware init failed");

    let jwt_arc = Arc::new(jwt);

    println!("Listening on {}", bind_addr);

    HttpServer::new(move || {
        let jwt_data = web::Data::new(jwt_arc.clone());

        App::new()
            .app_data(jwt_data.clone())
            // Public routes
            .route("/login", web::post().to(login))
            .route("/refresh", web::post().to(refresh))
            // Protected routes
            .service(
                web::scope("/auth")
                    .wrap(jwt_arc.middleware())
                    .route("/hello", web::get().to(hello))
                    .route("/logout", web::post().to(logout)),
            )
    })
    .bind(&bind_addr)?
    .run()
    .await
}

async fn login(
    jwt: web::Data<Arc<ActixJwtMiddleware>>,
    req: HttpRequest,
    body: web::Bytes,
) -> HttpResponse {
    jwt.login_handler(&req, &body).await
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

async fn hello(req: HttpRequest) -> HttpResponse {
    let claims = extract_claims(&req);
    let identity = get_identity(&req);

    HttpResponse::Ok().json(json!({
        "userID": claims.get(IDENTITY_KEY),
        "userName": identity,
        "text": "Hello World.",
    }))
}
