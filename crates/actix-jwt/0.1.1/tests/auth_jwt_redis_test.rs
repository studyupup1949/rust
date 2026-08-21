#![cfg(feature = "redis-store")]

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use actix_web::dev::ServiceResponse;
use actix_web::test;
use actix_web::{App, HttpRequest, HttpResponse, web};
use serde_json::Value;

use actix_jwt::store::redis::{RedisConfig, RedisRefreshTokenStore};
use actix_jwt::{ActixJwtMiddleware, JwtError, TokenStore, extract_claims, get_token};
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::redis::Redis;

async fn start_redis() -> (testcontainers::ContainerAsync<Redis>, String) {
    let container = Redis::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(6379).await.unwrap();
    let url = format!("redis://127.0.0.1:{}/", port);
    (container, url)
}

fn create_redis_mw(store: Arc<dyn TokenStore>) -> ActixJwtMiddleware {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.max_refresh = Duration::from_secs(86400);
    mw.refresh_token_timeout = Duration::from_secs(86400);
    mw.identity_key = "id".to_string();
    mw.refresh_token_store = store;
    mw.authenticator = Some(Arc::new(|_req, body| {
        let result = (|| -> Result<serde_json::Value, JwtError> {
            #[derive(serde::Deserialize)]
            struct Login {
                username: String,
                password: String,
            }
            let login: Login =
                serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
            if login.username == "admin" && login.password == "admin" {
                Ok(serde_json::json!({"username": "admin", "userid": 1}))
            } else {
                Err(JwtError::FailedAuthentication)
            }
        })();
        Box::pin(async move { result })
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        if let Some(v) = data.get("userid") {
            claims.insert("id".to_string(), v.clone());
        }
        if let Some(v) = data.get("username") {
            claims.insert("username".to_string(), v.clone());
        }
        claims
    }));
    mw.init().unwrap();
    mw
}

async fn create_redis_app(
    jwt: &Arc<ActixJwtMiddleware>,
) -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = ServiceResponse,
    Error = actix_web::Error,
> {
    let jwt_data = web::Data::new(jwt.clone());
    test::init_service(
        App::new()
            .app_data(jwt_data.clone())
            .route("/login", web::post().to(login_handler))
            .route("/refresh", web::post().to(refresh_handler))
            .service(web::scope("/auth").wrap(jwt.middleware()).route(
                "/hello",
                web::get().to(|req: HttpRequest| async move {
                    let claims = extract_claims(&req);
                    let token = get_token(&req).unwrap_or_default();
                    HttpResponse::Ok().json(serde_json::json!({
                        "message": "hello",
                        "claims": claims,
                        "token": token,
                    }))
                }),
            )),
    )
    .await
}

async fn login_handler(
    jwt: web::Data<Arc<ActixJwtMiddleware>>,
    req: HttpRequest,
    body: web::Bytes,
) -> HttpResponse {
    jwt.login_handler(&req, &body).await
}

async fn refresh_handler(
    jwt: web::Data<Arc<ActixJwtMiddleware>>,
    req: HttpRequest,
    body: web::Bytes,
) -> HttpResponse {
    jwt.refresh_handler(&req, &body).await
}

fn get_tokens_from_body(body: &[u8]) -> (String, Option<String>) {
    let v: Value = serde_json::from_slice(body).unwrap();
    let access = v["access_token"].as_str().unwrap().to_string();
    let refresh = v["refresh_token"].as_str().map(|s| s.to_string());
    (access, refresh)
}

// 1. TestEchoJWTMiddleware_RedisStore_Integration / LoginAndRefreshWithRedis
#[actix_web::test]
async fn test_redis_login_and_refresh_flow() {
    let (_container, url) = start_redis().await;

    let config = RedisConfig {
        addr: url,
        key_prefix: "test-jwt:".to_string(),
        ..Default::default()
    };
    let store = RedisRefreshTokenStore::new(&config).await.unwrap();
    let store: Arc<dyn TokenStore> = Arc::new(store);

    let mw = create_redis_mw(store);
    let jwt = Arc::new(mw);
    let app = create_redis_app(&jwt).await;

    // Login
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200, "login should succeed");
    let body = test::read_body(resp).await;
    let (access_token, refresh_token) = get_tokens_from_body(&body);
    assert!(!access_token.is_empty());
    let refresh_token = refresh_token.expect("response should contain refresh_token");
    assert!(!refresh_token.is_empty());

    // Use access token on protected endpoint
    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status().as_u16(),
        200,
        "protected endpoint should be accessible with valid token"
    );

    // Refresh
    let req = test::TestRequest::post()
        .uri("/refresh")
        .set_json(serde_json::json!({"refresh_token": refresh_token}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200, "refresh should succeed");
    let body = test::read_body(resp).await;
    let (new_access, new_refresh) = get_tokens_from_body(&body);
    assert!(!new_access.is_empty());
    assert!(
        new_refresh.is_some(),
        "refresh response should contain new refresh_token"
    );
}

// 2. TestEchoJWTMiddleware_RedisStore_Integration / TokenPersistenceAcrossRequests
#[actix_web::test]
async fn test_redis_token_persistence_across_requests() {
    let (_container, url) = start_redis().await;

    let config = RedisConfig {
        addr: url,
        key_prefix: "test-jwt:".to_string(),
        ..Default::default()
    };
    let store = RedisRefreshTokenStore::new(&config).await.unwrap();
    let store: Arc<dyn TokenStore> = Arc::new(store);

    let mw = create_redis_mw(store);
    let jwt = Arc::new(mw);
    let app = create_redis_app(&jwt).await;

    // Login
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    let body = test::read_body(resp).await;
    let (_, refresh_token) = get_tokens_from_body(&body);
    let mut refresh_token = refresh_token.unwrap();

    // Multiple sequential refreshes (token rotation)
    for i in 0..3 {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let req = test::TestRequest::post()
            .uri("/refresh")
            .set_json(serde_json::json!({"refresh_token": refresh_token}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status().as_u16(),
            200,
            "refresh {} should succeed",
            i + 1
        );

        let body = test::read_body(resp).await;
        let (_, new_refresh) = get_tokens_from_body(&body);
        refresh_token = new_refresh
            .unwrap_or_else(|| panic!("refresh {} should return new refresh_token", i + 1));
    }
}

// 3. TestEchoJWTMiddleware_RedisStore_Integration / RedisStoreOperations
#[actix_web::test]
async fn test_redis_store_operations_via_middleware() {
    let (_container, url) = start_redis().await;

    let config = RedisConfig {
        addr: url,
        key_prefix: "test-jwt:".to_string(),
        ..Default::default()
    };
    let store = RedisRefreshTokenStore::new(&config).await.unwrap();
    store.flush_db().await.unwrap();

    let store_arc: Arc<dyn TokenStore> = Arc::new(store);

    let mw = create_redis_mw(store_arc.clone());
    let jwt = Arc::new(mw);
    let app = create_redis_app(&jwt).await;

    // Login to create a refresh token in the store
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);

    // Verify token is in the store
    let count = store_arc.count().await.unwrap();
    assert!(
        count >= 1,
        "store should contain at least 1 refresh token after login"
    );

    // Direct store operations
    let test_token = "direct-test-token";
    let test_data = serde_json::json!({"test": "data"});
    let expiry = chrono::Utc::now() + chrono::Duration::hours(1);

    store_arc
        .set(test_token, test_data.clone(), expiry)
        .await
        .unwrap();

    let retrieved = store_arc.get(test_token).await.unwrap();
    assert_eq!(retrieved, test_data);

    store_arc.delete(test_token).await.unwrap();

    let result = store_arc.get(test_token).await;
    assert!(result.is_err(), "token should not exist after deletion");
}

// 4. TestEchoJWTMiddleware_RedisStoreFallback
// In Rust, there is no auto-fallback - connecting with invalid config returns an error.
// The user must handle the fallback themselves. This test verifies the error behavior.
#[actix_web::test]
async fn test_redis_store_fallback_to_memory() {
    // Use localhost with a port that is almost certainly not running Redis.
    // ConnectionManager retries internally, so we wrap in a timeout.
    let config = RedisConfig {
        addr: "redis://127.0.0.1:1/".to_string(),
        ..Default::default()
    };

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        RedisRefreshTokenStore::new(&config),
    )
    .await;

    // Either timeout or connection error - both mean Redis is unavailable
    let is_err = match &result {
        Err(_) => true,     // timeout
        Ok(Err(_)) => true, // connection error
        Ok(Ok(_)) => false, // unexpected success
    };
    assert!(is_err, "Should fail for invalid Redis configuration");

    // Unwrap to get the inner result for the fallback pattern
    let result = result.unwrap_or(Err(JwtError::Internal("timeout".into())));
    assert!(
        result.is_err(),
        "Should return error for invalid Redis configuration"
    );

    // In Rust, the fallback is manual: user creates in-memory store on error
    let store: Arc<dyn TokenStore> = match result {
        Ok(s) => Arc::new(s),
        Err(_) => Arc::new(actix_jwt::store::InMemoryRefreshTokenStore::new()),
    };

    // The fallback in-memory store should work
    let count = store.count().await.unwrap();
    assert_eq!(count, 0);

    // Middleware with fallback store should work
    let mw = create_redis_mw(store);
    let jwt = Arc::new(mw);
    let app = create_redis_app(&jwt).await;

    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status().as_u16(),
        200,
        "middleware with fallback memory store should work"
    );
}
