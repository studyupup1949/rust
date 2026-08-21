// Run: cargo run --example redis_tls --features redis-store
//
// Demonstrates Redis store with TLS configuration.
// Requires a TLS-enabled Redis instance.
//
// For production, configure proper certificates.
// This example shows the configuration pattern.
//
// TLS Configuration Notes (parity with the Go implementation):
//
// In the Go version (https://github.com/LdDl/echo-jwt/blob/master/examples/redis_tls/main.go),
// TLS is configured via `createTLSConfig()` which returns `*tls.Config`:
//   - Basic TLS: system CA certificates with TLS 1.2+ minimum
//   - Custom CA: `loadCACertificate(caPath)` reads PEM-encoded CA cert
//   - Mutual TLS: `loadClientCertificate(certPath, keyPath)` for client auth
//   - InsecureSkipVerify: for development only (NOT recommended for production)
//
// In Rust, the `redis` crate handles TLS via the URL scheme:
//   - `rediss://` enables TLS using native-tls or rustls (depending on feature flags)
//   - For custom CA certificates, configure via `redis` crate's TLS options
//   - For mutual TLS, use `redis::ConnectionInfo` with custom TLS connector
//
// Example with rustls (requires `redis` with `tokio-rustls-comp` feature):
//   use rustls::{ClientConfig, RootCertStore};
//   let mut root_store = RootCertStore::empty();
//   // Load custom CA: root_store.add_parsable_certificates(&certs);
//   // Load client cert for mTLS: ClientConfig::builder().with_client_auth_cert(...)
//
// Example with native-tls (requires `redis` with `tokio-native-tls-comp` feature):
//   use native_tls::TlsConnector;
//   let connector = TlsConnector::builder()
//       .add_root_certificate(cert)  // Custom CA
//       .identity(identity)          // Client cert for mTLS
//       .build()?;

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

        // Configure Redis with TLS
        // The `redis` crate supports TLS via the `rediss://` URL scheme.
        // Set `tls: true` to automatically convert `redis://` to `rediss://`.
        let config = RedisConfig {
            addr: "redis://redis.example.com:6380/".to_string(),
            password: Some("your-password".to_string()),
            db: 0,
            pool_size: 10,
            key_prefix: "actix-jwt:".to_string(),
            tls: true, // Enables TLS (rediss:// scheme)
        };

        // Alternative: use rediss:// scheme directly
        // let config = RedisConfig {
        //     addr: "rediss://redis.example.com:6380/".to_string(),
        //     tls: false, // Already using rediss://
        //     ..Default::default()
        // };

        let store = RedisRefreshTokenStore::new(&config)
            .await
            .expect("Failed to connect to Redis with TLS");
        println!("Connected to Redis (TLS) at {}", config.addr);

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
            if login.username == "admin" && login.password == "admin" {
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
        println!("Redis TLS store is enabled");

        HttpServer::new(move || {
            let jwt_data = web::Data::new(jwt_arc.clone());

            App::new()
                .app_data(jwt_data.clone())
                .route("/login", web::post().to(login))
                .route("/refresh", web::post().to(refresh))
                .service(
                    web::scope("/auth")
                        .wrap(jwt_arc.middleware())
                        .route("/hello", web::get().to(hello)),
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
}

#[cfg(feature = "redis-store")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    app::run().await
}

#[cfg(not(feature = "redis-store"))]
fn main() {
    eprintln!("This example requires the 'redis-store' feature:");
    eprintln!("  cargo run --example redis_tls --features redis-store");
    std::process::exit(1);
}
