use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use actix_web::cookie::Cookie;
use actix_web::dev::ServiceResponse;
use actix_web::test;
use actix_web::{App, HttpMessage, HttpRequest, HttpResponse, web};
use serde_json::Value;

use actix_jwt::{ActixJwtMiddleware, JwtError, Token, extract_claims, get_token};

fn create_test_middleware() -> Arc<ActixJwtMiddleware> {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.max_refresh = Duration::from_secs(3600);
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        if login.username == "admin" && login.password == "admin" {
            Ok(serde_json::json!({"username": "admin", "role": "admin"}))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        if let Some(username) = data.get("username").and_then(|v| v.as_str()) {
            claims.insert("identity".to_string(), Value::String(username.to_string()));
        }
        claims
    }));
    mw.init().unwrap();
    Arc::new(mw)
}

async fn create_test_app(
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
            .route("/logout", web::post().to(logout_handler))
            // Matches the Go implementation's echoHandler: /auth/refresh_token is registered BEFORE
            // middleware, so it does NOT require auth.
            .route("/auth/refresh_token", web::post().to(refresh_handler))
            .service(
                web::scope("/auth")
                    .wrap(jwt.middleware())
                    .route("/hello", web::get().to(hello_handler)),
            ),
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

async fn logout_handler(
    jwt: web::Data<Arc<ActixJwtMiddleware>>,
    req: HttpRequest,
    body: web::Bytes,
) -> HttpResponse {
    jwt.logout_handler(&req, &body).await
}

async fn hello_handler(req: HttpRequest) -> HttpResponse {
    let claims = extract_claims(&req);
    let token = get_token(&req).unwrap_or_default();
    HttpResponse::Ok().json(serde_json::json!({
        "text": "Hello World.",
        "message": "hello",
        "claims": claims,
        "token": token,
    }))
}

fn get_tokens_from_body(body: &[u8]) -> (String, String) {
    let v: Value = serde_json::from_slice(body).unwrap();
    let access = v["access_token"].as_str().unwrap().to_string();
    let refresh = v["refresh_token"].as_str().unwrap().to_string();
    (access, refresh)
}

async fn do_login(
    app: &impl actix_web::dev::Service<
        actix_http::Request,
        Response = ServiceResponse,
        Error = actix_web::Error,
    >,
) -> (String, String) {
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    let body = test::read_body(resp).await;
    get_tokens_from_body(&body)
}

fn make_token_string(alg: &str, identity: &str) -> String {
    let key = b"secret key salt";
    let algorithm = match alg {
        "HS256" => jsonwebtoken::Algorithm::HS256,
        "HS384" => jsonwebtoken::Algorithm::HS384,
        "HS512" => jsonwebtoken::Algorithm::HS512,
        "RS256" => {
            let priv_key = std::fs::read("testdata/jwtRS256.key").unwrap();
            let encoding_key = jsonwebtoken::EncodingKey::from_rsa_pem(&priv_key).unwrap();
            let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
            let now = chrono::Utc::now();
            let claims = serde_json::json!({
                "identity": identity,
                "exp": (now + chrono::Duration::hours(1)).timestamp(),
                "orig_iat": now.timestamp(),
            });
            return jsonwebtoken::encode(&header, &claims, &encoding_key).unwrap();
        }
        _ => panic!("unsupported algorithm: {alg}"),
    };
    let header = jsonwebtoken::Header::new(algorithm);
    let encoding_key = jsonwebtoken::EncodingKey::from_secret(key);
    let now = chrono::Utc::now();
    let claims = serde_json::json!({
        "identity": identity,
        "exp": (now + chrono::Duration::hours(1)).timestamp(),
        "orig_iat": now.timestamp(),
    });
    jsonwebtoken::encode(&header, &claims, &encoding_key).unwrap()
}

// 1. TestMissingKey
#[actix_web::test]
async fn test_missing_key() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.timeout = Duration::from_secs(3600);
    let result = mw.init();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, JwtError::MissingSecretKey),
        "expected MissingSecretKey, got: {err:?}"
    );
}

// 2. TestMissingPrivKey
#[actix_web::test]
async fn test_missing_priv_key() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "zone".to_string();
    mw.signing_algorithm = "RS256".to_string();
    mw.priv_key_file = Some("nonexisting".to_string());
    let result = mw.init();
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), JwtError::NoPrivKeyFile),
        "expected NoPrivKeyFile"
    );
}

// 3. TestMissingPubKey
#[actix_web::test]
async fn test_missing_pub_key() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "zone".to_string();
    mw.signing_algorithm = "RS256".to_string();
    mw.priv_key_file = Some("testdata/jwtRS256.key".to_string());
    mw.pub_key_file = Some("nonexisting".to_string());
    let result = mw.init();
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), JwtError::NoPubKeyFile),
        "expected NoPubKeyFile"
    );
}

// 4. TestInvalidPrivKey
#[actix_web::test]
async fn test_invalid_priv_key() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "zone".to_string();
    mw.signing_algorithm = "RS256".to_string();
    mw.priv_key_file = Some("testdata/invalidprivkey.key".to_string());
    mw.pub_key_file = Some("testdata/jwtRS256.key.pub".to_string());
    let result = mw.init();
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), JwtError::InvalidPrivKey),
        "expected InvalidPrivKey"
    );
}

// 5. TestInvalidPrivKeyBytes
#[actix_web::test]
async fn test_invalid_priv_key_bytes() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "zone".to_string();
    mw.signing_algorithm = "RS256".to_string();
    mw.priv_key_bytes = Some(b"Invalid_Private_Key".to_vec());
    mw.pub_key_file = Some("testdata/jwtRS256.key.pub".to_string());
    let result = mw.init();
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), JwtError::InvalidPrivKey),
        "expected InvalidPrivKey"
    );
}

// 6. TestInvalidPubKey
#[actix_web::test]
async fn test_invalid_pub_key() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "zone".to_string();
    mw.signing_algorithm = "RS256".to_string();
    mw.priv_key_file = Some("testdata/jwtRS256.key".to_string());
    mw.pub_key_file = Some("testdata/invalidpubkey.key".to_string());
    let result = mw.init();
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), JwtError::InvalidPubKey),
        "expected InvalidPubKey"
    );
}

// 7. TestInvalidPubKeyBytes
#[actix_web::test]
async fn test_invalid_pub_key_bytes() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "zone".to_string();
    mw.signing_algorithm = "RS256".to_string();
    mw.priv_key_file = Some("testdata/jwtRS256.key".to_string());
    mw.pub_key_bytes = Some(b"Invalid_Public_Key".to_vec());
    let result = mw.init();
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), JwtError::InvalidPubKey),
        "expected InvalidPubKey"
    );
}

// 8. TestMissingTimeOut
#[actix_web::test]
async fn test_missing_timeout() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::ZERO;
    mw.init().unwrap();
    assert_eq!(mw.timeout, Duration::from_secs(3600));
}

// 9. TestMissingTokenLookup
#[actix_web::test]
async fn test_missing_token_lookup() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.token_lookup = String::new();
    mw.init().unwrap();
    assert_eq!(mw.token_lookup, "header:Authorization");
}

// 10. TestMissingAuthenticatorForLoginHandler
#[actix_web::test]
async fn test_missing_authenticator() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.init().unwrap();
    let jwt = Arc::new(mw);
    let jwt_data = web::Data::new(jwt.clone());

    let app = test::init_service(
        App::new()
            .app_data(jwt_data.clone())
            .route("/login", web::post().to(login_handler)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 500);
}

// 11. TestLoginHandler
#[actix_web::test]
async fn test_login() {
    let jwt = create_test_middleware();
    let app = create_test_app(&jwt).await;

    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);

    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert!(
        v["access_token"].is_string(),
        "response must contain access_token"
    );
    assert!(
        v["refresh_token"].is_string(),
        "response must contain refresh_token"
    );
}

// 12. TestLoginWrongCredentials (part of TestLoginHandler from the Go implementation)
#[actix_web::test]
async fn test_login_wrong_credentials() {
    let jwt = create_test_middleware();
    let app = create_test_app(&jwt).await;

    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "wrong"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);
}

// 13. TestParseToken (multi-case)
#[actix_web::test]
async fn test_parse_token() {
    let jwt = create_test_middleware();
    let app = create_test_app(&jwt).await;

    // Empty auth header
    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header(("Authorization", ""))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);

    // Invalid auth header (wrong prefix)
    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header(("Authorization", "Test 1234"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);

    // Wrong algorithm (HS384 token on HS256 middleware)
    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header((
            "Authorization",
            format!("Bearer {}", make_token_string("HS384", "admin")),
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);

    // Valid token
    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header((
            "Authorization",
            format!("Bearer {}", make_token_string("HS256", "admin")),
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
}

// 14. TestParseTokenRS256 (= TestRSA with parse cases)
#[actix_web::test]
async fn test_rsa() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.signing_algorithm = "RS256".to_string();
    mw.priv_key_file = Some("testdata/jwtRS256.key".to_string());
    mw.pub_key_file = Some("testdata/jwtRS256.key.pub".to_string());
    mw.timeout = Duration::from_secs(3600);
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        if login.username == "admin" && login.password == "admin" {
            Ok(serde_json::json!({"username": "admin"}))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        if let Some(u) = data.get("username").and_then(|v| v.as_str()) {
            claims.insert("identity".to_string(), Value::String(u.to_string()));
        }
        claims
    }));
    mw.init().unwrap();
    let jwt = Arc::new(mw);
    let jwt_data = web::Data::new(jwt.clone());

    let app = test::init_service(
        App::new()
            .app_data(jwt_data.clone())
            .route("/login", web::post().to(login_handler))
            .service(
                web::scope("/auth")
                    .wrap(jwt.middleware())
                    .route("/hello", web::get().to(hello_handler)),
            ),
    )
    .await;

    // Empty auth header
    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header(("Authorization", ""))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);

    // Invalid auth header
    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header(("Authorization", "Test 1234"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);

    // Login and use RS256 token
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    let body = test::read_body(resp).await;
    let (access_token, _) = get_tokens_from_body(&body);

    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
}

// 15. TestParseTokenKeyFunc
#[actix_web::test]
async fn test_parse_token_key_func() {
    let pub_key_bytes = std::fs::read("testdata/jwtRS256.key.pub").unwrap();

    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.signing_algorithm = "RS256".to_string();
    mw.timeout = Duration::from_secs(3600);
    mw.max_refresh = Duration::from_secs(86400);
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        if login.username == "admin" && login.password == "admin" {
            Ok(serde_json::json!({"username": "admin"}))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        if let Some(u) = data.get("username").and_then(|v| v.as_str()) {
            claims.insert("identity".to_string(), Value::String(u.to_string()));
        }
        claims
    }));
    mw.key_func = Some(Arc::new(move |_header| {
        jsonwebtoken::DecodingKey::from_rsa_pem(&pub_key_bytes)
            .map_err(|e| JwtError::TokenParsing(e.to_string()))
    }));
    mw.init().unwrap();
    let jwt = Arc::new(mw);

    let app = test::init_service(
        App::new().service(
            web::scope("/auth")
                .wrap(jwt.middleware())
                .route("/hello", web::get().to(hello_handler)),
        ),
    )
    .await;

    // Empty auth header
    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header(("Authorization", ""))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);

    // Invalid auth header
    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header(("Authorization", "Test 1234"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);

    // Wrong algorithm (HS384 on RS256 key_func)
    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header((
            "Authorization",
            format!("Bearer {}", make_token_string("HS384", "admin")),
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);

    // Valid RS256 token
    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header((
            "Authorization",
            format!("Bearer {}", make_token_string("RS256", "admin")),
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
}

// 16. TestProtectedRoute
#[actix_web::test]
async fn test_protected_route() {
    let jwt = create_test_middleware();
    let app = create_test_app(&jwt).await;

    let (access_token, _) = do_login(&app).await;

    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);

    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["message"], "hello");
}

// 17. TestUnauthorized (no token)
#[actix_web::test]
async fn test_protected_route_no_token() {
    let jwt = create_test_middleware();
    let app = create_test_app(&jwt).await;

    let req = test::TestRequest::get().uri("/auth/hello").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);
}

// 18. TestExpiredTokenOnAuth
#[actix_web::test]
async fn test_protected_route_expired_token() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(1);
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        if login.username == "admin" && login.password == "admin" {
            Ok(serde_json::json!({"username": "admin"}))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        if let Some(u) = data.get("username").and_then(|v| v.as_str()) {
            claims.insert("identity".to_string(), Value::String(u.to_string()));
        }
        claims
    }));
    mw.time_func = Arc::new(|| chrono::Utc::now() - chrono::Duration::hours(2));
    mw.init().unwrap();
    let jwt = Arc::new(mw);
    let jwt_data = web::Data::new(jwt.clone());

    let app = test::init_service(
        App::new()
            .app_data(jwt_data.clone())
            .route("/login", web::post().to(login_handler))
            .service(
                web::scope("/auth")
                    .wrap(jwt.middleware())
                    .route("/hello", web::get().to(hello_handler)),
            ),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    let body = test::read_body(resp).await;
    let (access_token, _) = get_tokens_from_body(&body);

    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status().as_u16(),
        401,
        "expired token should be rejected"
    );
}

// 19. TestExpiredTokenOnAuth (explicitly expired token with send_authorization)
#[actix_web::test]
async fn test_expired_token_on_auth_with_send_authorization() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.max_refresh = Duration::from_secs(86400);
    mw.send_authorization = true;
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        if login.username == "admin" && login.password == "admin" {
            Ok(serde_json::json!({"username": "admin"}))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        if let Some(u) = data.get("username").and_then(|v| v.as_str()) {
            claims.insert("identity".to_string(), Value::String(u.to_string()));
        }
        claims
    }));
    mw.init().unwrap();
    let jwt = Arc::new(mw);

    let app = test::init_service(
        App::new().service(
            web::scope("/auth")
                .wrap(jwt.middleware())
                .route("/hello", web::get().to(hello_handler)),
        ),
    )
    .await;

    // Create an already-expired token
    let key = b"secret key salt";
    let now = chrono::Utc::now();
    let expired_claims = serde_json::json!({
        "identity": "admin",
        "exp": (now - chrono::Duration::hours(1)).timestamp(),
        "orig_iat": (now - chrono::Duration::hours(2)).timestamp(),
    });
    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256);
    let encoding_key = jsonwebtoken::EncodingKey::from_secret(key);
    let expired_token = jsonwebtoken::encode(&header, &expired_claims, &encoding_key).unwrap();

    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header(("Authorization", format!("Bearer {expired_token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);
}

// 20. TestRefreshHandler
#[actix_web::test]
async fn test_refresh() {
    let jwt = create_test_middleware();
    let app = create_test_app(&jwt).await;

    let (_, refresh_token) = do_login(&app).await;

    let req = test::TestRequest::post()
        .uri("/refresh")
        .set_json(serde_json::json!({"refresh_token": refresh_token}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);

    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert!(
        v["access_token"].is_string(),
        "refresh must return new access_token"
    );
    assert!(
        v["refresh_token"].is_string(),
        "refresh must return new refresh_token"
    );

    let new_refresh = v["refresh_token"].as_str().unwrap();
    assert_ne!(new_refresh, refresh_token, "refresh token must rotate");
}

// 21. TestRefreshHandlerRS256
#[actix_web::test]
async fn test_refresh_handler_rs256() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.signing_algorithm = "RS256".to_string();
    mw.priv_key_file = Some("testdata/jwtRS256.key".to_string());
    mw.pub_key_file = Some("testdata/jwtRS256.key.pub".to_string());
    mw.timeout = Duration::from_secs(3600);
    mw.max_refresh = Duration::from_secs(86400);
    mw.send_cookie = true;
    mw.cookie_name = "jwt".to_string();
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        if login.username == "admin" && login.password == "admin" {
            Ok(serde_json::json!({"username": "admin"}))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        if let Some(u) = data.get("username").and_then(|v| v.as_str()) {
            claims.insert("identity".to_string(), Value::String(u.to_string()));
        }
        claims
    }));
    mw.init().unwrap();
    let jwt = Arc::new(mw);
    let jwt_data = web::Data::new(jwt.clone());

    let app = test::init_service(
        App::new()
            .app_data(jwt_data.clone())
            .route("/login", web::post().to(login_handler))
            .route("/auth/refresh_token", web::post().to(refresh_handler))
            .service(
                web::scope("/auth")
                    .wrap(jwt.middleware())
                    .route("/hello", web::get().to(hello_handler)),
            ),
    )
    .await;

    // Missing refresh token
    let req = test::TestRequest::post()
        .uri("/auth/refresh_token")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 400);

    // Invalid refresh token
    let req = test::TestRequest::post()
        .uri("/auth/refresh_token")
        .set_json(serde_json::json!({"refresh_token": "invalid_token"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);

    // Valid refresh: login first, then refresh
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    let body = test::read_body(resp).await;
    let (_, refresh_token) = get_tokens_from_body(&body);

    let req = test::TestRequest::post()
        .uri("/auth/refresh_token")
        .set_json(serde_json::json!({"refresh_token": refresh_token}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert!(v["access_token"].as_str().map_or(false, |s| !s.is_empty()));
    assert!(v["refresh_token"].as_str().map_or(false, |s| !s.is_empty()));
    assert_ne!(v["refresh_token"].as_str().unwrap(), refresh_token);
}

// 22. TestRefreshMissingToken
#[actix_web::test]
async fn test_refresh_missing_token() {
    let jwt = create_test_middleware();
    let app = create_test_app(&jwt).await;

    let req = test::TestRequest::post()
        .uri("/refresh")
        .set_json(serde_json::json!({}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 400);
}

// 23. TestValidRefreshToken
#[actix_web::test]
async fn test_valid_refresh_token() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.max_refresh = Duration::from_secs(7200);
    mw.refresh_token_timeout = Duration::from_secs(86400);
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        if login.username == "admin" && login.password == "admin" {
            Ok(serde_json::json!({"username": "admin"}))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        if let Some(u) = data.get("username").and_then(|v| v.as_str()) {
            claims.insert("identity".to_string(), Value::String(u.to_string()));
        }
        claims
    }));
    mw.init().unwrap();
    let jwt = Arc::new(mw);
    let jwt_data = web::Data::new(jwt.clone());

    let app = test::init_service(
        App::new()
            .app_data(jwt_data.clone())
            .route("/login", web::post().to(login_handler))
            .route("/auth/refresh_token", web::post().to(refresh_handler)),
    )
    .await;

    // Login
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    let body = test::read_body(resp).await;
    let (_, refresh_token) = get_tokens_from_body(&body);

    // Refresh
    let req = test::TestRequest::post()
        .uri("/auth/refresh_token")
        .set_json(serde_json::json!({"refresh_token": refresh_token}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
}

// 24. TestExpiredTokenOnRefreshHandler
#[actix_web::test]
async fn test_expired_token_on_refresh_handler() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.refresh_token_timeout = Duration::from_millis(1);
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        if login.username == "admin" && login.password == "admin" {
            Ok(serde_json::json!({"username": "admin"}))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        if let Some(u) = data.get("username").and_then(|v| v.as_str()) {
            claims.insert("identity".to_string(), Value::String(u.to_string()));
        }
        claims
    }));
    mw.init().unwrap();
    let jwt = Arc::new(mw);
    let jwt_data = web::Data::new(jwt.clone());

    let app = test::init_service(
        App::new()
            .app_data(jwt_data.clone())
            .route("/login", web::post().to(login_handler))
            .route("/auth/refresh_token", web::post().to(refresh_handler)),
    )
    .await;

    // Login
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    let body = test::read_body(resp).await;
    let (_, refresh_token) = get_tokens_from_body(&body);

    // Wait for refresh token to expire
    tokio::time::sleep(Duration::from_millis(5)).await;

    // Refresh should fail (expired)
    let req = test::TestRequest::post()
        .uri("/auth/refresh_token")
        .set_json(serde_json::json!({"refresh_token": refresh_token}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);
}

// 25. TestBadTokenOnRefreshHandler
#[actix_web::test]
async fn test_bad_token_on_refresh_handler() {
    let jwt = create_test_middleware();
    let app = create_test_app(&jwt).await;

    let req = test::TestRequest::post()
        .uri("/auth/refresh_token")
        .set_json(serde_json::json!({"refresh_token": "BadToken"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);
}

// 26. TestLogout
#[actix_web::test]
async fn test_logout() {
    let jwt = create_test_middleware();
    let app = create_test_app(&jwt).await;

    let (access_token, refresh_token) = do_login(&app).await;

    let req = test::TestRequest::post()
        .uri("/logout")
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .set_json(serde_json::json!({"refresh_token": refresh_token}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
}

// 27. TestLogout with cookies
#[actix_web::test]
async fn test_logout_with_cookies() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.send_cookie = true;
    mw.cookie_name = "jwt".to_string();
    mw.cookie_domain = Some("example.com".to_string());
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        if login.username == "admin" && login.password == "admin" {
            Ok(serde_json::json!({"username": "admin"}))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        if let Some(u) = data.get("username").and_then(|v| v.as_str()) {
            claims.insert("identity".to_string(), Value::String(u.to_string()));
        }
        claims
    }));
    mw.init().unwrap();
    let jwt = Arc::new(mw);
    let jwt_data = web::Data::new(jwt.clone());

    let app = test::init_service(
        App::new()
            .app_data(jwt_data.clone())
            .route("/logout", web::post().to(logout_handler)),
    )
    .await;

    let req = test::TestRequest::post().uri("/logout").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    let set_cookie = resp.headers().get("Set-Cookie");
    assert!(set_cookie.is_some(), "should set cookie header on logout");
    let cookie_val = set_cookie.unwrap().to_str().unwrap();
    assert!(cookie_val.contains("jwt="), "cookie should contain jwt=");
    assert!(
        cookie_val.contains("Domain=example.com"),
        "cookie should contain domain"
    );
}

// 28. TestAuthorizer
#[actix_web::test]
async fn test_authorizer() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        if login.username == "user" && login.password == "user" {
            Ok(serde_json::json!({"username": "user", "role": "viewer"}))
        } else if login.username == "admin" && login.password == "admin" {
            Ok(serde_json::json!({"username": "admin", "role": "admin"}))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        if let Some(u) = data.get("username").and_then(|v| v.as_str()) {
            claims.insert("identity".to_string(), Value::String(u.to_string()));
        }
        if let Some(r) = data.get("role").and_then(|v| v.as_str()) {
            claims.insert("role".to_string(), Value::String(r.to_string()));
        }
        claims
    }));
    mw.authorizer = Arc::new(|req, _data| {
        let ext = req.extensions();
        if let Some(payload) = ext.get::<actix_jwt::JwtPayload>() {
            if let Some(role) = payload.0.get("role").and_then(|v: &Value| v.as_str()) {
                return role == "admin";
            }
        }
        false
    });
    mw.init().unwrap();
    let jwt = Arc::new(mw);
    let jwt_data = web::Data::new(jwt.clone());

    let app = test::init_service(
        App::new()
            .app_data(jwt_data.clone())
            .route("/login", web::post().to(login_handler))
            .service(
                web::scope("/auth")
                    .wrap(jwt.middleware())
                    .route("/hello", web::get().to(hello_handler)),
            ),
    )
    .await;

    // Login as "user" (viewer role) - should be forbidden
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "user", "password": "user"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    let body = test::read_body(resp).await;
    let (user_token, _) = get_tokens_from_body(&body);

    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header(("Authorization", format!("Bearer {user_token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status().as_u16(),
        403,
        "viewer role should be forbidden"
    );
}

// 29. TestCustomPayload
#[actix_web::test]
async fn test_custom_payload() {
    let jwt = create_test_middleware();
    let app = create_test_app(&jwt).await;

    let (access_token, _) = do_login(&app).await;

    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);

    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        v["claims"]["identity"], "admin",
        "custom payload should include identity claim"
    );
}

// 30. TestClaimsDuringAuthorization
#[actix_web::test]
async fn test_claims_during_authorization() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.max_refresh = Duration::from_secs(86400);
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        if login.username == "admin" && login.password == "admin" {
            Ok(serde_json::json!({"username": "admin"}))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        if let Some(u) = data.get("username").and_then(|v| v.as_str()) {
            claims.insert("identity".to_string(), Value::String(u.to_string()));
            let testkey = if u == "admin" { "1234" } else { "" };
            claims.insert("testkey".to_string(), Value::String(testkey.to_string()));
        }
        claims
    }));
    mw.authorizer = Arc::new(|req, _data| {
        let ext = req.extensions();
        if let Some(payload) = ext.get::<actix_jwt::JwtPayload>() {
            if payload.0.get("identity").and_then(|v| v.as_str()) == Some("admin")
                && payload.0.get("testkey").and_then(|v| v.as_str()) == Some("1234")
            {
                return true;
            }
        }
        false
    });
    mw.init().unwrap();
    let jwt = Arc::new(mw);
    let jwt_data = web::Data::new(jwt.clone());

    let app = test::init_service(
        App::new()
            .app_data(jwt_data.clone())
            .route("/login", web::post().to(login_handler))
            .service(
                web::scope("/auth")
                    .wrap(jwt.middleware())
                    .route("/hello", web::get().to(hello_handler)),
            ),
    )
    .await;

    // Login and use token
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    let body = test::read_body(resp).await;
    let (access_token, _) = get_tokens_from_body(&body);

    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
}

// 31. TestEmptyClaims
#[actix_web::test]
async fn test_empty_claims() {
    let jwt = create_test_middleware();
    let app = create_test_app(&jwt).await;

    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header(("Authorization", "Bearer 1234"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);
}

// 32. TestTokenExpire (refresh with zero max_refresh)
#[actix_web::test]
async fn test_token_expire() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        if login.username == "admin" && login.password == "admin" {
            Ok(serde_json::json!({"username": "admin"}))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        if let Some(u) = data.get("username").and_then(|v| v.as_str()) {
            claims.insert("identity".to_string(), Value::String(u.to_string()));
        }
        claims
    }));
    mw.init().unwrap();
    let jwt = Arc::new(mw);
    let jwt_data = web::Data::new(jwt.clone());

    let app = test::init_service(
        App::new()
            .app_data(jwt_data.clone())
            .route("/auth/refresh_token", web::post().to(refresh_handler)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/auth/refresh_token")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 400);
}

// 33. TestTokenFromQueryString
#[actix_web::test]
async fn test_token_lookup_query() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.token_lookup = "query:token".to_string();
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        if login.username == "admin" && login.password == "admin" {
            Ok(serde_json::json!({"username": "admin"}))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        if let Some(u) = data.get("username").and_then(|v| v.as_str()) {
            claims.insert("identity".to_string(), Value::String(u.to_string()));
        }
        claims
    }));
    mw.init().unwrap();
    let jwt = Arc::new(mw);
    let jwt_data = web::Data::new(jwt.clone());

    let app = test::init_service(
        App::new()
            .app_data(jwt_data.clone())
            .route("/login", web::post().to(login_handler))
            .service(
                web::scope("/auth")
                    .wrap(jwt.middleware())
                    .route("/hello", web::get().to(hello_handler)),
            ),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    let body = test::read_body(resp).await;
    let (access_token, _) = get_tokens_from_body(&body);

    // Header should NOT work when looking from query
    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);

    // Query should work
    let req = test::TestRequest::get()
        .uri(&format!("/auth/hello?token={access_token}"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
}

// 34. TestTokenFromParamPath
#[actix_web::test]
async fn test_token_from_param_path() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.token_lookup = "param:token".to_string();
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        if login.username == "admin" && login.password == "admin" {
            Ok(serde_json::json!({"username": "admin"}))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        if let Some(u) = data.get("username").and_then(|v| v.as_str()) {
            claims.insert("identity".to_string(), Value::String(u.to_string()));
        }
        claims
    }));
    mw.init().unwrap();
    let jwt = Arc::new(mw);
    let jwt_data = web::Data::new(jwt.clone());

    let app = test::init_service(
        App::new()
            .app_data(jwt_data.clone())
            .route("/login", web::post().to(login_handler))
            .service(
                web::resource("/g/{token}/hello")
                    .wrap(jwt.middleware())
                    .route(web::get().to(hello_handler)),
            ),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    let body = test::read_body(resp).await;
    let (access_token, _) = get_tokens_from_body(&body);

    // Param path should work
    let req = test::TestRequest::get()
        .uri(&format!("/g/{access_token}/hello"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
}

// 35. TestTokenFromCookieString
#[actix_web::test]
async fn test_token_lookup_cookie() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.token_lookup = "cookie:jwt".to_string();
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        if login.username == "admin" && login.password == "admin" {
            Ok(serde_json::json!({"username": "admin"}))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        if let Some(u) = data.get("username").and_then(|v| v.as_str()) {
            claims.insert("identity".to_string(), Value::String(u.to_string()));
        }
        claims
    }));
    mw.init().unwrap();
    let jwt = Arc::new(mw);
    let jwt_data = web::Data::new(jwt.clone());

    let app = test::init_service(
        App::new()
            .app_data(jwt_data.clone())
            .route("/login", web::post().to(login_handler))
            .service(
                web::scope("/auth")
                    .wrap(jwt.middleware())
                    .route("/hello", web::get().to(hello_handler)),
            ),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    let body = test::read_body(resp).await;
    let (access_token, _) = get_tokens_from_body(&body);

    // Header should NOT work
    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);

    // Cookie should work
    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .cookie(Cookie::new("jwt", &access_token))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);

    // Verify token is available in response
    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["token"].as_str().unwrap(), access_token);
}

// 36. TestDefineTokenHeadName
#[actix_web::test]
async fn test_define_token_head_name() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.token_head_name = "JWTTOKEN       ".to_string();
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        if login.username == "admin" && login.password == "admin" {
            Ok(serde_json::json!({"username": "admin"}))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        if let Some(u) = data.get("username").and_then(|v| v.as_str()) {
            claims.insert("identity".to_string(), Value::String(u.to_string()));
        }
        claims
    }));
    mw.init().unwrap();
    let jwt = Arc::new(mw);

    let app = test::init_service(
        App::new().service(
            web::scope("/auth")
                .wrap(jwt.middleware())
                .route("/hello", web::get().to(hello_handler)),
        ),
    )
    .await;

    // "Bearer" prefix should NOT work
    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header((
            "Authorization",
            format!("Bearer {}", make_token_string("HS256", "admin")),
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);

    // "JWTTOKEN" prefix should work
    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header((
            "Authorization",
            format!("JWTTOKEN {}", make_token_string("HS256", "admin")),
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
}

// 37. TestHTTPStatusMessageFunc
#[actix_web::test]
async fn test_http_status_message_func_via_http() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.authenticator = Some(Arc::new(|_req, _body| Err(JwtError::FailedAuthentication)));
    mw.http_status_message_func = Arc::new(|_req, err| {
        if matches!(err, JwtError::FailedAuthentication) {
            "Custom error message".to_string()
        } else {
            err.to_string()
        }
    });
    mw.init().unwrap();
    let jwt = Arc::new(mw);
    let jwt_data = web::Data::new(jwt.clone());

    let app = test::init_service(
        App::new()
            .app_data(jwt_data.clone())
            .route("/login", web::post().to(login_handler)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);
    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["message"], "Custom error message");
}

// 38. TestSendAuthorizationBool
#[actix_web::test]
async fn test_send_authorization_bool() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.max_refresh = Duration::from_secs(86400);
    mw.send_authorization = true;
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        if login.username == "admin" && login.password == "admin" {
            Ok(serde_json::json!({"username": "admin"}))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        if let Some(u) = data.get("username").and_then(|v| v.as_str()) {
            claims.insert("identity".to_string(), Value::String(u.to_string()));
        }
        claims
    }));
    mw.init().unwrap();
    let jwt = Arc::new(mw);
    let jwt_data = web::Data::new(jwt.clone());

    let app = test::init_service(
        App::new()
            .app_data(jwt_data.clone())
            .route("/login", web::post().to(login_handler))
            .service(
                web::scope("/auth")
                    .wrap(jwt.middleware())
                    .route("/hello", web::get().to(hello_handler)),
            ),
    )
    .await;

    // Login
    let (access_token, _) = do_login(&app).await;

    // Use token - Authorization header should be echoed back
    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    let auth_header = resp
        .headers()
        .get("Authorization")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        auth_header.starts_with("Bearer "),
        "Authorization header should have Bearer prefix"
    );
}

// 39. TestCheckTokenString
#[actix_web::test]
async fn test_check_token_string() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        if login.username == "admin" && login.password == "admin" {
            Ok(serde_json::json!({"username": "admin"}))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        if let Some(u) = data.get("username").and_then(|v| v.as_str()) {
            claims.insert("identity".to_string(), Value::String(u.to_string()));
        }
        claims
    }));
    mw.init().unwrap();

    // Generate token and parse it
    let data = serde_json::json!({"username": "admin"});
    let (token_string, _) = mw.generate_access_token(&data).unwrap();

    let parsed = mw.parse_token_string(&token_string);
    assert!(parsed.is_ok());
    let token_data = parsed.unwrap();
    let claims = token_data.claims.as_object().unwrap();
    assert_eq!(
        claims.get("identity").and_then(|v| v.as_str()),
        Some("admin")
    );
}

// 40. TestSetCookie
#[actix_web::test]
async fn test_set_cookie() {
    let config = actix_jwt::CookieConfig {
        name: "jwt".to_string(),
        max_age: Duration::from_secs(3600),
        secure: false,
        http_only: true,
        domain: Some("example.com".to_string()),
        same_site: actix_web::cookie::SameSite::Lax,
    };

    let mut builder = HttpResponse::Ok();
    ActixJwtMiddleware::set_cookie(&mut builder, &config, "test-token-value");
    let resp = builder.finish();

    let cookies: Vec<_> = resp.headers().get_all("Set-Cookie").collect();
    assert!(!cookies.is_empty(), "should have Set-Cookie header");
    let cookie_str = cookies[0].to_str().unwrap();
    assert!(cookie_str.contains("jwt=test-token-value"));
    assert!(cookie_str.contains("HttpOnly"));
    assert!(cookie_str.contains("Domain=example.com"));
}

// 41. TestTokenGenerator
#[actix_web::test]
async fn test_token_generator() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.max_refresh = Duration::from_secs(86400);
    mw.authenticator = Some(Arc::new(|_req, _body| Ok(serde_json::json!("admin"))));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        claims.insert("identity".to_string(), data.clone());
        claims
    }));
    mw.init().unwrap();

    let data = serde_json::json!("admin");
    let token_pair = mw.token_generator(&data).await.unwrap();

    assert!(!token_pair.access_token.is_empty());
    assert!(token_pair.refresh_token.is_some());
    assert!(!token_pair.refresh_token.as_ref().unwrap().is_empty());
    assert_eq!(token_pair.token_type, "Bearer");
    assert!(token_pair.expires_at > chrono::Utc::now().timestamp());
    assert!(token_pair.created_at <= chrono::Utc::now().timestamp());
    assert!(token_pair.expires_in() > 0);

    // Parse and verify claims
    let parsed = mw.parse_token_string(&token_pair.access_token).unwrap();
    let claims = parsed.claims.as_object().unwrap();
    assert_eq!(
        claims.get("identity").and_then(|v| v.as_str()),
        Some("admin")
    );
}

// 42. TestTokenGeneratorWithRevocation
#[actix_web::test]
async fn test_token_generator_with_revocation() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.max_refresh = Duration::from_secs(86400);
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        claims.insert("identity".to_string(), data.clone());
        claims
    }));
    mw.init().unwrap();

    let data = serde_json::json!("admin");

    // Generate first token pair
    let old_pair = mw.token_generator(&data).await.unwrap();
    let old_refresh = old_pair.refresh_token.as_ref().unwrap().clone();

    // Generate new pair with revocation of old
    let new_pair = mw
        .token_generator_with_revocation(&data, &old_refresh)
        .await
        .unwrap();
    let new_refresh = new_pair.refresh_token.as_ref().unwrap().clone();

    assert_ne!(old_refresh, new_refresh);

    // Old refresh token should no longer be valid in the store
    let store = &mw.refresh_token_store;
    let result = store.get(&old_refresh).await;
    assert!(result.is_err(), "old refresh token should be revoked");

    // New refresh token should be valid
    let result = store.get(&new_refresh).await;
    assert!(result.is_ok(), "new refresh token should be valid");

    // Revoking non-existent token should still succeed
    let another = mw
        .token_generator_with_revocation(&data, "non_existent_token")
        .await;
    assert!(another.is_ok());
}

// 43. TestTokenStruct
#[actix_web::test]
async fn test_token_struct() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.max_refresh = Duration::from_secs(86400);
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        claims.insert("identity".to_string(), data.clone());
        claims
    }));
    mw.init().unwrap();

    let data = serde_json::json!("admin");
    let token_pair = mw.token_generator(&data).await.unwrap();

    let expires_in = token_pair.expires_in();
    assert!(
        expires_in > 3500,
        "expires_in should be > 3500, got {expires_in}"
    );
    assert!(
        expires_in <= 3600,
        "expires_in should be <= 3600, got {expires_in}"
    );
    assert!(!token_pair.access_token.is_empty());
    assert_eq!(token_pair.token_type, "Bearer");
    assert!(token_pair.refresh_token.is_some());
    assert!(token_pair.expires_at > chrono::Utc::now().timestamp());
    assert!(token_pair.created_at > 0);
    assert!(token_pair.created_at <= chrono::Utc::now().timestamp());
}

// 44. TestSkipper
#[actix_web::test]
async fn test_skipper() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        if login.username == "admin" && login.password == "admin" {
            Ok(serde_json::json!({"username": "admin"}))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));
    mw.skipper = Some(Arc::new(|req| req.path() == "/auth/public"));
    mw.init().unwrap();
    let jwt = Arc::new(mw);

    let app = test::init_service(
        App::new().service(
            web::scope("/auth")
                .wrap(jwt.middleware())
                .route(
                    "/public",
                    web::get().to(|| async {
                        HttpResponse::Ok().json(serde_json::json!({"public": true}))
                    }),
                )
                .route("/hello", web::get().to(hello_handler)),
        ),
    )
    .await;

    let req = test::TestRequest::get().uri("/auth/public").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status().as_u16(),
        200,
        "skipped path should be accessible without token"
    );

    let req = test::TestRequest::get().uri("/auth/hello").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status().as_u16(),
        401,
        "non-skipped path should require auth"
    );
}

// 45. TestBeforeFunc
#[actix_web::test]
async fn test_before_func() {
    let before_called = Arc::new(AtomicBool::new(false));
    let before_called_clone = before_called.clone();

    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.authenticator = Some(Arc::new(|_req, _body| Err(JwtError::FailedAuthentication)));
    mw.before_func = Some(Arc::new(move |_req| {
        before_called_clone.store(true, Ordering::SeqCst);
    }));
    mw.init().unwrap();
    let jwt = Arc::new(mw);

    let app = test::init_service(
        App::new().service(
            web::scope("/auth")
                .wrap(jwt.middleware())
                .route("/hello", web::get().to(hello_handler)),
        ),
    )
    .await;

    let req = test::TestRequest::get().uri("/auth/hello").to_request();
    let _resp = test::call_service(&app, req).await;
    assert!(
        before_called.load(Ordering::SeqCst),
        "before_func should be called before token extraction"
    );
}

// 46. TestSuccessHandler
#[actix_web::test]
async fn test_success_handler() {
    let success_called = Arc::new(AtomicBool::new(false));
    let success_called_clone = success_called.clone();

    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        if login.username == "admin" && login.password == "admin" {
            Ok(serde_json::json!({"username": "admin"}))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        if let Some(u) = data.get("username").and_then(|v| v.as_str()) {
            claims.insert("identity".to_string(), Value::String(u.to_string()));
        }
        claims
    }));
    mw.success_handler = Some(Arc::new(move |_req| {
        success_called_clone.store(true, Ordering::SeqCst);
        Ok(())
    }));
    mw.init().unwrap();
    let jwt = Arc::new(mw);
    let jwt_data = web::Data::new(jwt.clone());

    let app = test::init_service(
        App::new()
            .app_data(jwt_data.clone())
            .route("/login", web::post().to(login_handler))
            .service(
                web::scope("/auth")
                    .wrap(jwt.middleware())
                    .route("/hello", web::get().to(hello_handler)),
            ),
    )
    .await;

    let (access_token, _) = do_login(&app).await;

    success_called.store(false, Ordering::SeqCst);
    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    assert!(
        success_called.load(Ordering::SeqCst),
        "success_handler should be called on valid token"
    );
}

// 47. TestErrorHandler
#[actix_web::test]
async fn test_error_handler() {
    let error_called = Arc::new(AtomicBool::new(false));
    let error_called_clone = error_called.clone();

    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.authenticator = Some(Arc::new(|_req, _body| Err(JwtError::FailedAuthentication)));
    mw.error_handler = Some(Arc::new(move |_req, _err| {
        error_called_clone.store(true, Ordering::SeqCst);
        Some(JwtError::Forbidden)
    }));
    mw.init().unwrap();
    let jwt = Arc::new(mw);

    let app = test::init_service(
        App::new().service(
            web::scope("/auth")
                .wrap(jwt.middleware())
                .route("/hello", web::get().to(hello_handler)),
        ),
    )
    .await;

    let req = test::TestRequest::get().uri("/auth/hello").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(
        error_called.load(Ordering::SeqCst),
        "error_handler should be called on auth failure"
    );
    assert_eq!(
        resp.status().as_u16(),
        403,
        "error_handler returned Forbidden"
    );
}

// 48. TestContinueOnIgnoredError
#[actix_web::test]
async fn test_continue_on_ignored_error() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.authenticator = Some(Arc::new(|_req, _body| Err(JwtError::FailedAuthentication)));
    mw.continue_on_ignored_error = true;
    mw.error_handler = Some(Arc::new(|_req, _err| None));
    mw.init().unwrap();
    let jwt = Arc::new(mw);

    let app = test::init_service(App::new().service(
        web::scope("/auth").wrap(jwt.middleware()).route(
            "/hello",
            web::get().to(|| async {
                HttpResponse::Ok().json(serde_json::json!({"message": "public access"}))
            }),
        ),
    ))
    .await;

    let req = test::TestRequest::get().uri("/auth/hello").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status().as_u16(),
        200,
        "should continue when error is ignored"
    );

    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["message"], "public access");
}

// 49. TestTokenParsingError
#[actix_web::test]
async fn test_token_parsing_error() {
    let err = JwtError::TokenParsing("invalid signature".to_string());
    assert!(err.is_token_parsing());
    assert!(!err.is_token_extraction());
    assert_eq!(err.to_string(), "invalid signature");
}

// 50. TestTokenExtractionError
#[actix_web::test]
async fn test_token_extraction_error() {
    let err = JwtError::TokenExtraction("no token found".to_string());
    assert!(err.is_token_extraction());
    assert!(!err.is_token_parsing());
    assert_eq!(err.to_string(), "no token found");
}

// 51. TestWWWAuthenticateHeader
#[actix_web::test]
async fn test_www_authenticate_header() {
    let test_cases: Vec<(&str, &str, &str)> = vec![
        (
            "test zone",
            "Bearer invalid_token",
            r#"Bearer realm="test zone""#,
        ),
        ("my custom realm", "", r#"Bearer realm="my custom realm""#),
        (
            "test-zone_123",
            "Bearer invalid",
            r#"Bearer realm="test-zone_123""#,
        ),
        (
            "api realm",
            "Bearer not.a.valid.jwt.token",
            r#"Bearer realm="api realm""#,
        ),
    ];

    for (realm, auth_header, expected) in test_cases {
        let mut mw = ActixJwtMiddleware::new();
        mw.realm = realm.to_string();
        mw.key = b"secret key salt".to_vec();
        mw.timeout = Duration::from_secs(3600);
        mw.max_refresh = Duration::from_secs(86400);
        mw.authenticator = Some(Arc::new(|_req, body| {
            #[derive(serde::Deserialize)]
            struct Login {
                username: String,
                password: String,
            }
            let login: Login =
                serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
            if login.username == "admin" && login.password == "admin" {
                Ok(serde_json::json!({"username": "admin"}))
            } else {
                Err(JwtError::FailedAuthentication)
            }
        }));
        mw.init().unwrap();
        let jwt = Arc::new(mw);

        let app = test::init_service(
            App::new().service(
                web::scope("/auth")
                    .wrap(jwt.middleware())
                    .route("/hello", web::get().to(hello_handler)),
            ),
        )
        .await;

        let mut req_builder = test::TestRequest::get().uri("/auth/hello");
        if !auth_header.is_empty() {
            req_builder = req_builder.insert_header(("Authorization", auth_header));
        }
        let req = req_builder.to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 401);
        let www_auth = resp
            .headers()
            .get("WWW-Authenticate")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(www_auth, expected, "realm={realm}");
    }
}

// 52. TestWWWAuthenticateHeaderOnRefresh
#[actix_web::test]
async fn test_www_authenticate_header_on_refresh() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "refresh realm".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.max_refresh = Duration::from_secs(86400);
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        if login.username == "admin" && login.password == "admin" {
            Ok(serde_json::json!({"username": "admin"}))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        if let Some(u) = data.get("username").and_then(|v| v.as_str()) {
            claims.insert("identity".to_string(), Value::String(u.to_string()));
        }
        claims
    }));
    mw.init().unwrap();
    let jwt = Arc::new(mw);
    let jwt_data = web::Data::new(jwt.clone());

    let app = test::init_service(
        App::new()
            .app_data(jwt_data.clone())
            .route("/auth/refresh_token", web::post().to(refresh_handler)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/auth/refresh_token")
        .set_json(serde_json::json!({"refresh_token": "invalid_refresh_token"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);
    let www_auth = resp
        .headers()
        .get("WWW-Authenticate")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(www_auth, r#"Bearer realm="refresh realm""#);
}

// 53. TestWWWAuthenticateHeaderNotSetOnSuccess
#[actix_web::test]
async fn test_www_authenticate_header_not_set_on_success() {
    let jwt = create_test_middleware();
    let app = create_test_app(&jwt).await;

    // Login should not have WWW-Authenticate
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    assert!(
        resp.headers().get("WWW-Authenticate").is_none(),
        "success should not set WWW-Authenticate"
    );

    let body = test::read_body(resp).await;
    let (access_token, _) = get_tokens_from_body(&body);

    // Successful auth should not have WWW-Authenticate
    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    assert!(
        resp.headers().get("WWW-Authenticate").is_none(),
        "success should not set WWW-Authenticate"
    );
}

// 54. TestWWWAuthenticateHeaderWithDifferentRealms
#[actix_web::test]
async fn test_www_authenticate_header_with_different_realms() {
    let realms = vec![
        ("actix jwt", "actix jwt"),   // default
        ("API Server", "API Server"), // with space
        ("my-api", "my-api"),         // with dash
        ("realm_test", "realm_test"), // with underscore
        ("MyApp v1.0", "MyApp v1.0"), // with version
        ("", "actix jwt"),            // empty (should use default)
    ];

    for (realm, expected_realm) in realms {
        let mut mw = ActixJwtMiddleware::new();
        mw.realm = realm.to_string();
        mw.key = b"secret key salt".to_vec();
        mw.timeout = Duration::from_secs(3600);
        mw.max_refresh = Duration::from_secs(86400);
        mw.authenticator = Some(Arc::new(|_req, body| {
            #[derive(serde::Deserialize)]
            struct Login {
                username: String,
                password: String,
            }
            let login: Login =
                serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
            if login.username == "admin" && login.password == "admin" {
                Ok(serde_json::json!({"username": "admin"}))
            } else {
                Err(JwtError::FailedAuthentication)
            }
        }));
        mw.init().unwrap();
        let jwt = Arc::new(mw);

        let app = test::init_service(
            App::new().service(
                web::scope("/auth")
                    .wrap(jwt.middleware())
                    .route("/hello", web::get().to(hello_handler)),
            ),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/auth/hello")
            .insert_header(("Authorization", "Bearer invalid"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 401);
        let www_auth = resp
            .headers()
            .get("WWW-Authenticate")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(
            www_auth,
            format!(r#"Bearer realm="{expected_realm}""#),
            "realm={realm:?}"
        );
    }
}

// 55. TestStandardJWTClaimsInPayloadFunc
#[actix_web::test]
async fn test_standard_jwt_claims_in_payload_func() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.max_refresh = Duration::from_secs(86400);
    mw.authenticator = Some(Arc::new(|_req, _body| Ok(serde_json::json!("user123"))));
    mw.payload_func = Some(Arc::new(|data| {
        let user_id = data.as_str().unwrap_or("unknown");
        let mut claims = HashMap::new();
        claims.insert("sub".to_string(), Value::String(user_id.to_string()));
        claims.insert("iss".to_string(), Value::String("my-app".to_string()));
        claims.insert("aud".to_string(), Value::String("my-api".to_string()));
        claims.insert(
            "jti".to_string(),
            Value::String("unique-token-id-12345".to_string()),
        );
        claims.insert("identity".to_string(), Value::String(user_id.to_string()));
        claims.insert("role".to_string(), Value::String("admin".to_string()));
        claims
    }));
    mw.init().unwrap();

    let data = serde_json::json!("user123");
    let token_pair = mw.token_generator(&data).await.unwrap();

    let parsed = mw.parse_token_string(&token_pair.access_token).unwrap();
    let claims = parsed.claims.as_object().unwrap();

    assert_eq!(claims.get("sub").and_then(|v| v.as_str()), Some("user123"));
    assert_eq!(claims.get("iss").and_then(|v| v.as_str()), Some("my-app"));
    assert_eq!(claims.get("aud").and_then(|v| v.as_str()), Some("my-api"));
    assert_eq!(
        claims.get("jti").and_then(|v| v.as_str()),
        Some("unique-token-id-12345")
    );
    assert_eq!(
        claims.get("identity").and_then(|v| v.as_str()),
        Some("user123")
    );
    assert_eq!(claims.get("role").and_then(|v| v.as_str()), Some("admin"));
    assert!(claims.contains_key("exp"));
    assert!(claims.contains_key("orig_iat"));
}

// 56. TestFrameworkClaimsCannotBeOverwritten
#[actix_web::test]
async fn test_framework_claims_cannot_be_overwritten() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.max_refresh = Duration::from_secs(86400);
    let fixed_time = chrono::Utc::now();
    let fixed_time_clone = fixed_time;
    mw.time_func = Arc::new(move || fixed_time_clone);
    mw.authenticator = Some(Arc::new(|_req, _body| Ok(serde_json::json!("user123"))));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        // Try to override framework claims
        claims.insert("exp".to_string(), serde_json::json!(9999999999i64));
        claims.insert("orig_iat".to_string(), serde_json::json!(1111111111i64));
        claims.insert("identity".to_string(), data.clone());
        claims
    }));
    mw.init().unwrap();

    let data = serde_json::json!("user123");
    let token_pair = mw.token_generator(&data).await.unwrap();

    let parsed = mw.parse_token_string(&token_pair.access_token).unwrap();
    let claims = parsed.claims.as_object().unwrap();

    // exp should be set by framework (fixed_time + 1 hour), NOT 9999999999
    let exp = claims.get("exp").and_then(|v| v.as_i64()).unwrap();
    let expected_exp = (fixed_time + chrono::Duration::hours(1)).timestamp();
    assert_eq!(exp, expected_exp, "exp should be framework-controlled");

    // orig_iat should be set by framework, NOT 1111111111
    let orig_iat = claims.get("orig_iat").and_then(|v| v.as_i64()).unwrap();
    assert_eq!(
        orig_iat,
        fixed_time.timestamp(),
        "orig_iat should be framework-controlled"
    );

    // identity should be preserved (not a framework claim)
    assert_eq!(
        claims.get("identity").and_then(|v| v.as_str()),
        Some("user123")
    );
}

// 57. TestAllStandardClaimsCanBeSet
#[actix_web::test]
async fn test_all_standard_claims_can_be_set() {
    let test_cases: Vec<(&str, Value)> = vec![
        ("sub", Value::String("user-12345".to_string())),
        ("iss", Value::String("https://auth.example.com".to_string())),
        ("aud", Value::String("https://api.example.com".to_string())),
        (
            "jti",
            Value::String("550e8400-e29b-41d4-a716-446655440000".to_string()),
        ),
    ];

    for (claim_key, claim_value) in test_cases {
        let claim_key_owned = claim_key.to_string();
        let claim_value_clone = claim_value.clone();

        let mut mw = ActixJwtMiddleware::new();
        mw.realm = "test zone".to_string();
        mw.key = b"secret key salt".to_vec();
        mw.timeout = Duration::from_secs(3600);
        mw.authenticator = Some(Arc::new(|_req, _body| Ok(serde_json::json!("user"))));
        mw.payload_func = Some(Arc::new(move |data| {
            let mut claims = HashMap::new();
            claims.insert(claim_key_owned.clone(), claim_value_clone.clone());
            claims.insert("identity".to_string(), data.clone());
            claims
        }));
        mw.init().unwrap();

        let data = serde_json::json!("user");
        let token_pair = mw.token_generator(&data).await.unwrap();
        let parsed = mw.parse_token_string(&token_pair.access_token).unwrap();
        let claims = parsed.claims.as_object().unwrap();

        assert_eq!(
            claims.get(claim_key),
            Some(&claim_value),
            "claim {claim_key} should be set correctly"
        );
    }
}

// 58. TestSubClaimAsUserIdentifier
#[actix_web::test]
async fn test_sub_claim_as_user_identifier() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.identity_key = "sub".to_string();
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        Ok(serde_json::json!({"username": login.username}))
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let user_id = data
            .get("username")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let mut claims = HashMap::new();
        claims.insert("sub".to_string(), Value::String(user_id.to_string()));
        claims.insert("name".to_string(), Value::String("Test User".to_string()));
        claims.insert(
            "email".to_string(),
            Value::String("test@example.com".to_string()),
        );
        claims
    }));
    mw.identity_handler = Arc::new(|req| {
        let ext = req.extensions();
        let payload = ext.get::<actix_jwt::JwtPayload>()?;
        payload.0.get("sub").cloned()
    });
    mw.authorizer = Arc::new(|req, _data| {
        let ext = req.extensions();
        if let Some(payload) = ext.get::<actix_jwt::JwtPayload>() {
            if let Some(sub) = payload.0.get("sub").and_then(|v| v.as_str()) {
                return sub == "admin";
            }
        }
        false
    });
    mw.init().unwrap();
    let jwt = Arc::new(mw);
    let jwt_data = web::Data::new(jwt.clone());

    let app = test::init_service(
        App::new()
            .app_data(jwt_data.clone())
            .route("/login", web::post().to(login_handler))
            .service(
                web::scope("/auth")
                    .wrap(jwt.middleware())
                    .route("/hello", web::get().to(hello_handler)),
            ),
    )
    .await;

    // Login
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    let body = test::read_body(resp).await;
    let (access_token, _) = get_tokens_from_body(&body);

    // Use token
    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);

    // Verify claims
    let parsed = jwt.parse_token_string(&access_token).unwrap();
    let claims = parsed.claims.as_object().unwrap();
    assert_eq!(claims.get("sub").and_then(|v| v.as_str()), Some("admin"));
    assert_eq!(
        claims.get("name").and_then(|v| v.as_str()),
        Some("Test User")
    );
    assert_eq!(
        claims.get("email").and_then(|v| v.as_str()),
        Some("test@example.com")
    );
}

// 59. TestGenerateTokenResponse
#[actix_web::test]
async fn test_generate_token_response() {
    let now = chrono::Utc::now();
    let token = Token {
        access_token: "test.access.token".to_string(),
        token_type: "Bearer".to_string(),
        refresh_token: Some("test-refresh-token".to_string()),
        expires_at: (now + chrono::Duration::hours(1)).timestamp(),
        created_at: now.timestamp(),
    };

    let resp = ActixJwtMiddleware::generate_token_response(&token);
    assert_eq!(
        resp.get("access_token").and_then(|v| v.as_str()),
        Some("test.access.token")
    );
    assert_eq!(
        resp.get("token_type").and_then(|v| v.as_str()),
        Some("Bearer")
    );
    assert!(resp.get("refresh_token").and_then(|v| v.as_str()).is_some());
    assert!(resp.get("expires_in").is_some());
}

// 60. TestLoginHandlerWithCookie
#[actix_web::test]
async fn test_login_handler_with_cookie() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.send_cookie = true;
    mw.cookie_name = "jwt".to_string();
    mw.cookie_domain = Some("example.com".to_string());
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        if login.username == "admin" && login.password == "admin" {
            Ok(serde_json::json!({"username": "admin"}))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        if let Some(u) = data.get("username").and_then(|v| v.as_str()) {
            claims.insert("identity".to_string(), Value::String(u.to_string()));
        }
        claims
    }));
    mw.init().unwrap();
    let jwt = Arc::new(mw);
    let jwt_data = web::Data::new(jwt.clone());

    let app = test::init_service(
        App::new()
            .app_data(jwt_data.clone())
            .route("/login", web::post().to(login_handler)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);

    let cookies: Vec<_> = resp.headers().get_all("Set-Cookie").collect();
    assert!(!cookies.is_empty(), "should set cookies on login");
    let cookie_str = cookies[0].to_str().unwrap();
    assert!(
        cookie_str.starts_with("jwt="),
        "cookie should start with jwt="
    );
    assert!(
        cookie_str.contains("Domain=example.com"),
        "cookie should contain domain"
    );
    assert!(
        cookie_str.contains("Max-Age=3600"),
        "cookie should contain max-age"
    );
}

// 61. TestBeforeFuncNotCalledWhenSkipped
#[actix_web::test]
async fn test_before_func_not_called_when_skipped() {
    let before_called = Arc::new(AtomicBool::new(false));
    let before_called_clone = before_called.clone();

    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.authenticator = Some(Arc::new(|_req, _body| Err(JwtError::FailedAuthentication)));
    mw.skipper = Some(Arc::new(|_req| true)); // skip everything
    mw.before_func = Some(Arc::new(move |_req| {
        before_called_clone.store(true, Ordering::SeqCst);
    }));
    mw.init().unwrap();
    let jwt = Arc::new(mw);

    let app = test::init_service(
        App::new().service(
            web::scope("/auth")
                .wrap(jwt.middleware())
                .route("/hello", web::get().to(hello_handler)),
        ),
    )
    .await;

    let req = test::TestRequest::get().uri("/auth/hello").to_request();
    let _resp = test::call_service(&app, req).await;
    assert!(
        !before_called.load(Ordering::SeqCst),
        "before_func should NOT be called when skipper returns true"
    );
}

// 62. TestSuccessHandlerError
#[actix_web::test]
async fn test_success_handler_error() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        if login.username == "admin" && login.password == "admin" {
            Ok(serde_json::json!({"username": "admin"}))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        if let Some(u) = data.get("username").and_then(|v| v.as_str()) {
            claims.insert("identity".to_string(), Value::String(u.to_string()));
        }
        claims
    }));
    mw.success_handler = Some(Arc::new(|_req| Err(JwtError::Forbidden)));
    mw.init().unwrap();
    let jwt = Arc::new(mw);
    let jwt_data = web::Data::new(jwt.clone());

    let app = test::init_service(
        App::new()
            .app_data(jwt_data.clone())
            .route("/login", web::post().to(login_handler))
            .service(
                web::scope("/auth")
                    .wrap(jwt.middleware())
                    .route("/hello", web::get().to(hello_handler)),
            ),
    )
    .await;

    let (access_token, _) = do_login(&app).await;

    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_ne!(
        resp.status().as_u16(),
        200,
        "success_handler error should stop the chain"
    );
}

// 63. TestErrorHandlerWithInvalidToken
#[actix_web::test]
async fn test_error_handler_with_invalid_token() {
    let received_parsing = Arc::new(AtomicBool::new(false));
    let received_parsing_clone = received_parsing.clone();

    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.authenticator = Some(Arc::new(|_req, _body| Err(JwtError::FailedAuthentication)));
    mw.error_handler = Some(Arc::new(move |_req, err| {
        if err.is_token_parsing() {
            received_parsing_clone.store(true, Ordering::SeqCst);
        }
        Some(err)
    }));
    mw.init().unwrap();
    let jwt = Arc::new(mw);

    let app = test::init_service(
        App::new().service(
            web::scope("/auth")
                .wrap(jwt.middleware())
                .route("/hello", web::get().to(hello_handler)),
        ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/auth/hello")
        .insert_header(("Authorization", "Bearer invalid.token.here"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);
    assert!(
        received_parsing.load(Ordering::SeqCst),
        "invalid token should produce a token parsing error"
    );
}

// 64. TestContinueOnIgnoredErrorFalse
#[actix_web::test]
async fn test_continue_on_ignored_error_false() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.authenticator = Some(Arc::new(|_req, _body| Err(JwtError::FailedAuthentication)));
    mw.continue_on_ignored_error = false;
    mw.error_handler = Some(Arc::new(|_req, _err| None));
    mw.init().unwrap();
    let jwt = Arc::new(mw);

    let app = test::init_service(App::new().service(
        web::scope("/auth").wrap(jwt.middleware()).route(
            "/hello",
            web::get().to(|| async {
                HttpResponse::Ok().json(serde_json::json!({"message": "Hello World"}))
            }),
        ),
    ))
    .await;

    let req = test::TestRequest::get().uri("/auth/hello").to_request();
    let resp = test::call_service(&app, req).await;
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        !body_str.contains("Hello World"),
        "handler chain should be stopped when ContinueOnIgnoredError is false"
    );
}

// 65. TestSkipperWithLogin
#[actix_web::test]
async fn test_skipper_with_login() {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.authenticator = Some(Arc::new(|_req, body| {
        #[derive(serde::Deserialize)]
        struct Login {
            username: String,
            password: String,
        }
        let login: Login =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
        if login.username == "admin" && login.password == "admin" {
            Ok(serde_json::json!({"username": "admin"}))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));
    mw.payload_func = Some(Arc::new(|data| {
        let mut claims = HashMap::new();
        if let Some(u) = data.get("username").and_then(|v| v.as_str()) {
            claims.insert("identity".to_string(), Value::String(u.to_string()));
        }
        claims
    }));
    mw.skipper = Some(Arc::new(|req| req.path().starts_with("/auth/public")));
    mw.init().unwrap();
    let jwt = Arc::new(mw);
    let jwt_data = web::Data::new(jwt.clone());

    let app = test::init_service(
        App::new()
            .app_data(jwt_data.clone())
            .route("/login", web::post().to(login_handler))
            .service(
                web::scope("/auth")
                    .wrap(jwt.middleware())
                    .route(
                        "/public/info",
                        web::get().to(|| async {
                            HttpResponse::Ok().json(serde_json::json!({"public": true}))
                        }),
                    )
                    .route("/hello", web::get().to(hello_handler)),
            ),
    )
    .await;

    // Public route works without auth
    let req = test::TestRequest::get()
        .uri("/auth/public/info")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["public"], true);

    // Auth route still requires token
    let req = test::TestRequest::get().uri("/auth/hello").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);
}

// 66. TestEchoJWTMiddleware_FunctionalOptionsOnly (RedisConfig construction)
// In the Go implementation this tests EnableRedisStore() with functional options.
// In Rust, RedisConfig is a plain struct - test construction with various fields.
#[cfg(feature = "redis-store")]
#[actix_web::test]
async fn test_redis_config_construction() {
    use actix_jwt::store::redis::RedisConfig;

    // Default configuration
    let config = RedisConfig::default();
    assert_eq!(config.addr, "redis://127.0.0.1:6379/");
    assert!(config.password.is_none());
    assert_eq!(config.db, 0);
    assert_eq!(config.pool_size, 10);
    assert_eq!(config.key_prefix, "actix-jwt:");
    assert!(!config.tls);

    // Custom address
    let config = RedisConfig {
        addr: "redis://redis.example.com:6379/".to_string(),
        ..Default::default()
    };
    assert_eq!(config.addr, "redis://redis.example.com:6379/");
    assert!(config.password.is_none());
    assert_eq!(config.db, 0);

    // Custom auth
    let config = RedisConfig {
        addr: "redis://redis.example.com:6379/".to_string(),
        password: Some("testpass".to_string()),
        db: 5,
        ..Default::default()
    };
    assert_eq!(config.addr, "redis://redis.example.com:6379/");
    assert_eq!(config.password, Some("testpass".to_string()));
    assert_eq!(config.db, 5);

    // Custom pool
    let config = RedisConfig {
        pool_size: 20,
        ..Default::default()
    };
    assert_eq!(config.pool_size, 20);

    // Custom key prefix
    let config = RedisConfig {
        key_prefix: "test-app:".to_string(),
        ..Default::default()
    };
    assert_eq!(config.key_prefix, "test-app:");

    // All options
    let config = RedisConfig {
        addr: "redis://custom.redis.com:6379/".to_string(),
        password: Some("custom-password".to_string()),
        db: 3,
        pool_size: 25,
        key_prefix: "custom-prefix:".to_string(),
        tls: true,
    };
    assert_eq!(config.addr, "redis://custom.redis.com:6379/");
    assert_eq!(config.password, Some("custom-password".to_string()));
    assert_eq!(config.db, 3);
    assert_eq!(config.pool_size, 25);
    assert_eq!(config.key_prefix, "custom-prefix:");
    assert!(config.tls);

    // Override - second config replaces first (no mutation like in the Go implementation)
    let _first = RedisConfig {
        addr: "redis://first.redis.com:6379/".to_string(),
        password: Some("first-pass".to_string()),
        db: 1,
        ..Default::default()
    };
    let second = RedisConfig {
        addr: "redis://second.redis.com:6379/".to_string(),
        password: Some("second-pass".to_string()),
        db: 2,
        key_prefix: "second:".to_string(),
        ..Default::default()
    };
    assert_eq!(second.addr, "redis://second.redis.com:6379/");
    assert_eq!(second.password, Some("second-pass".to_string()));
    assert_eq!(second.db, 2);
    assert_eq!(second.key_prefix, "second:");
}
