// Run: cargo run --example redis_store --features redis-store
//
// Demonstrates Redis store with explicit configuration.
// Requires a running Redis instance on localhost:6379.
//
// curl -s -X POST http://localhost:8000/login -H 'Content-Type: application/json' -d '{"username":"admin","password":"admin"}'
// curl -s http://localhost:8000/auth/hello -H 'Authorization: Bearer <token>'
// curl -s http://localhost:8000/auth/store-info -H 'Authorization: Bearer <token>'

#[cfg(feature = "redis-store")]
mod app {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;

    use actix_jwt::store::redis::{RedisConfig, RedisRefreshTokenStore};
    use actix_jwt::{ActixJwtMiddleware, JwtError, extract_claims, get_identity};
    use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
    use serde_json::{Value, json};

    const IDENTITY_KEY: &str = "id";

    pub async fn run() -> std::io::Result<()> {
        let bind_addr = "0.0.0.0:8000";

        // Configure Redis with explicit options
        let config = RedisConfig {
            addr: "redis://127.0.0.1:6379/".to_string(),
            password: None,
            db: 0,
            pool_size: 10,
            key_prefix: "actix-jwt:".to_string(),
            tls: false,
        };

        let store = RedisRefreshTokenStore::new(&config)
            .await
            .expect("Failed to connect to Redis");
        println!("Connected to Redis at {}", config.addr);

        let mut jwt = ActixJwtMiddleware::new();
        jwt.realm = "test zone".to_string();
        jwt.key = b"secret key".to_vec();
        jwt.timeout = Duration::from_secs(3600);
        jwt.max_refresh = Duration::from_secs(3600);
        jwt.identity_key = IDENTITY_KEY.to_string();
        jwt.refresh_token_store = Arc::new(store);
        jwt.token_lookup = "header:Authorization, query:token, cookie:jwt".to_string();

        jwt.authenticator = Some(Arc::new(|_req: &HttpRequest, body: &[u8]| {
            #[derive(serde::Deserialize)]
            struct Login {
                username: String,
                password: String,
            }
            let login: Login =
                serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
            if (login.username == "admin" && login.password == "admin")
                || (login.username == "test" && login.password == "test")
            {
                Ok(json!({"user_name": login.username}))
            } else {
                Err(JwtError::FailedAuthentication)
            }
        }));

        jwt.payload_func = Some(Arc::new(|data: &Value| {
            let mut claims = HashMap::new();
            if let Some(v) = data.get("user_name") {
                claims.insert(IDENTITY_KEY.to_string(), v.clone());
            }
            claims
        }));

        jwt.identity_handler = Arc::new(|req: &HttpRequest| {
            let claims = extract_claims(req);
            claims.get(IDENTITY_KEY).cloned()
        });

        jwt.authorizer =
            Arc::new(|_req: &HttpRequest, data: &Value| data.as_str() == Some("admin"));

        jwt.init().expect("JWT middleware init failed");
        let jwt_arc = Arc::new(jwt);

        println!("Listening on {}", bind_addr);

        HttpServer::new(move || {
            let jwt_data = web::Data::new(jwt_arc.clone());

            App::new()
                .app_data(jwt_data.clone())
                .route("/login", web::post().to(login))
                .route("/refresh", web::post().to(refresh))
                .service(
                    web::scope("/auth")
                        .wrap(jwt_arc.middleware())
                        .route("/hello", web::get().to(hello))
                        .route("/store-info", web::get().to(store_info)),
                )
        })
        .bind(bind_addr)?
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

    async fn hello(req: HttpRequest) -> HttpResponse {
        let claims = extract_claims(&req);
        let identity = get_identity(&req);
        HttpResponse::Ok().json(json!({
            "userID": claims.get(IDENTITY_KEY),
            "userName": identity,
            "text": "Hello World.",
        }))
    }

    async fn store_info() -> HttpResponse {
        HttpResponse::Ok().json(json!({
            "store": "redis",
            "message": "Using Redis-backed refresh token store",
        }))
    }
}

#[cfg(feature = "redis-store")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    app::run().await
}

#[cfg(not(feature = "redis-store"))]
fn main() {
    eprintln!("This example requires the 'redis-store' feature:");
    eprintln!("  cargo run --example redis_store --features redis-store");
    std::process::exit(1);
}
