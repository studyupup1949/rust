// Run: cargo run --example authorization
//
// Users:
//   admin/admin (role: admin) - access to all routes
//   user/user   (role: user)  - access to /user/* and /auth/profile
//   guest/guest (role: guest) - access to /auth/hello only
//
// curl -s -X POST http://localhost:8000/login -H 'Content-Type: application/json' -d '{"username":"admin","password":"admin"}'
// curl -s http://localhost:8000/auth/hello -H 'Authorization: Bearer <token>'
// curl -s http://localhost:8000/admin/users -H 'Authorization: Bearer <token>'

use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;

use actix_jwt::{ActixJwtMiddleware, JwtError, extract_claims, get_identity};
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use serde_json::{Value, json};

const IDENTITY_KEY: &str = "id";
const ROLE_KEY: &str = "role";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let bind_addr = format!("0.0.0.0:{}", port);

    let mut jwt = ActixJwtMiddleware::new();
    jwt.realm = "authorization example".to_string();
    jwt.key = b"secret key".to_vec();
    jwt.timeout = Duration::from_secs(3600);
    jwt.max_refresh = Duration::from_secs(3600);
    jwt.identity_key = IDENTITY_KEY.to_string();
    jwt.token_lookup = "header:Authorization, query:token, cookie:jwt".to_string();
    jwt.token_head_name = "Bearer".to_string();

    jwt.authenticator = Some(Arc::new(|_req: &HttpRequest, body: &[u8]| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;

        let users: HashMap<&str, (&str, &str)> = HashMap::from([
            ("admin", ("admin", "admin")),
            ("user", ("user", "user")),
            ("guest", ("guest", "guest")),
        ]);

        if let Some((pass, role)) = users.get(login.username.as_str()) {
            if login.password == *pass {
                return Ok(json!({"user_name": login.username, "role": role}));
            }
        }
        Err(JwtError::FailedAuthentication)
    }));

    jwt.payload_func = Some(Arc::new(|data: &Value| {
        let mut claims = HashMap::new();
        if let Some(v) = data.get("user_name") {
            claims.insert(IDENTITY_KEY.to_string(), v.clone());
        }
        if let Some(v) = data.get("role") {
            claims.insert(ROLE_KEY.to_string(), v.clone());
        }
        claims
    }));

    jwt.identity_handler = Arc::new(|req: &HttpRequest| {
        let claims = extract_claims(req);
        Some(json!({
            "user_name": claims.get(IDENTITY_KEY),
            "role": claims.get(ROLE_KEY),
        }))
    });

    jwt.authorizer = Arc::new(|req: &HttpRequest, data: &Value| {
        let role = data.get("role").and_then(|v| v.as_str()).unwrap_or("");
        let path = req.path();

        // Admin has access to everything
        if role == "admin" {
            return true;
        }

        // Admin routes - only admin allowed
        if path.starts_with("/admin/") {
            return false;
        }

        // User routes - user and admin roles
        if path.starts_with("/user/") {
            return role == "user";
        }

        // Auth routes with specific rules
        if path.starts_with("/auth/") {
            return match path {
                "/auth/hello" | "/auth/whoami" | "/auth/logout" => true,
                "/auth/profile" => role == "user",
                _ => false,
            };
        }

        false
    });

    jwt.unauthorized = Arc::new(|req: &HttpRequest, code: u16, message: &str| {
        HttpResponse::build(
            actix_web::http::StatusCode::from_u16(code)
                .unwrap_or(actix_web::http::StatusCode::UNAUTHORIZED),
        )
        .json(json!({
            "code": code,
            "message": message,
            "path": req.path(),
            "method": req.method().as_str(),
        }))
    });

    jwt.init().expect("JWT middleware init failed");
    let jwt_arc = Arc::new(jwt);

    println!("Listening on {}", bind_addr);
    println!("Users: admin/admin, user/user, guest/guest");

    HttpServer::new(move || {
        let jwt_data = web::Data::new(jwt_arc.clone());

        App::new()
            .app_data(jwt_data.clone())
            .route("/login", web::post().to(login))
            .route("/refresh", web::post().to(refresh))
            .route("/info", web::get().to(info))
            // Admin routes
            .service(
                web::scope("/admin")
                    .wrap(jwt_arc.middleware())
                    .route("/users", web::get().to(admin_users))
                    .route("/users", web::post().to(create_user))
                    .route("/users/{id}", web::delete().to(delete_user))
                    .route("/settings", web::get().to(admin_settings))
                    .route("/reports", web::get().to(admin_reports)),
            )
            // User routes
            .service(
                web::scope("/user")
                    .wrap(jwt_arc.middleware())
                    .route("/profile", web::get().to(user_profile))
                    .route("/profile", web::put().to(update_profile))
                    .route("/settings", web::get().to(user_settings)),
            )
            // General auth routes
            .service(
                web::scope("/auth")
                    .wrap(jwt_arc.middleware())
                    .route("/hello", web::get().to(hello))
                    .route("/whoami", web::get().to(whoami))
                    .route("/profile", web::get().to(auth_profile))
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

async fn info() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "message": "Authorization Example API",
        "users": {
            "admin": {"password": "admin", "role": "admin", "access": "All routes"},
            "user": {"password": "user", "role": "user", "access": "/user/* and /auth/profile"},
            "guest": {"password": "guest", "role": "guest", "access": "/auth/hello only"},
        },
        "routes": {
            "public": ["/login", "/refresh", "/info"],
            "admin": ["/admin/users", "/admin/settings", "/admin/reports"],
            "user": ["/user/profile", "/user/settings"],
            "auth": ["/auth/hello", "/auth/profile", "/auth/logout"],
        },
    }))
}

async fn hello(req: HttpRequest) -> HttpResponse {
    let claims = extract_claims(&req);
    let identity = get_identity(&req);
    let user_name = identity
        .as_ref()
        .and_then(|v| v.get("user_name"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let role = identity
        .as_ref()
        .and_then(|v| v.get("role"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    HttpResponse::Ok().json(json!({
        "message": "Hello World!",
        "userID": claims.get(IDENTITY_KEY),
        "userName": user_name,
        "role": role,
        "access": "all authenticated users",
    }))
}

async fn whoami(req: HttpRequest) -> HttpResponse {
    let claims = extract_claims(&req);
    let identity = get_identity(&req);
    HttpResponse::Ok().json(json!({
        "identity": claims.get(IDENTITY_KEY),
        "role": claims.get(ROLE_KEY),
        "user": identity,
        "claims": claims,
        "access": "all authenticated users",
    }))
}

async fn auth_profile(req: HttpRequest) -> HttpResponse {
    let claims = extract_claims(&req);
    let identity = get_identity(&req);
    let user_name = identity
        .as_ref()
        .and_then(|v| v.get("user_name"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let role = identity
        .as_ref()
        .and_then(|v| v.get("role"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    HttpResponse::Ok().json(json!({
        "message": "Profile Information",
        "userID": claims.get(IDENTITY_KEY),
        "userName": user_name,
        "role": role,
        "access": "user and admin roles only",
    }))
}

async fn admin_users() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "message": "Admin Users Management",
        "users": ["admin", "user1", "user2", "guest1"],
        "access": "admin only",
    }))
}

async fn create_user() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "message": "User created successfully",
        "access": "admin only",
    }))
}

async fn delete_user(path: web::Path<String>) -> HttpResponse {
    let user_id = path.into_inner();
    HttpResponse::Ok().json(json!({
        "message": "User deleted successfully",
        "user_id": user_id,
        "access": "admin only",
    }))
}

async fn admin_settings() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "message": "Admin Settings",
        "settings": {"max_users": 100, "allow_registration": true},
        "access": "admin only",
    }))
}

async fn admin_reports() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "message": "Admin Reports",
        "reports": ["daily_usage", "user_activity", "system_health"],
        "access": "admin only",
    }))
}

async fn user_profile(req: HttpRequest) -> HttpResponse {
    let identity = get_identity(&req);
    let user_name = identity
        .as_ref()
        .and_then(|v| v.get("user_name"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let role = identity
        .as_ref()
        .and_then(|v| v.get("role"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    HttpResponse::Ok().json(json!({
        "message": "User Profile",
        "username": user_name,
        "role": role,
        "access": "user and admin only",
    }))
}

async fn update_profile(req: HttpRequest) -> HttpResponse {
    let identity = get_identity(&req);
    let user_name = identity
        .as_ref()
        .and_then(|v| v.get("user_name"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    HttpResponse::Ok().json(json!({
        "message": "Profile updated successfully",
        "username": user_name,
        "access": "user and admin only",
    }))
}

async fn user_settings(req: HttpRequest) -> HttpResponse {
    let identity = get_identity(&req);
    let user_name = identity
        .as_ref()
        .and_then(|v| v.get("user_name"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    HttpResponse::Ok().json(json!({
        "message": "User Settings",
        "username": user_name,
        "settings": {"theme": "dark", "notifications": true},
        "access": "user and admin only",
    }))
}
