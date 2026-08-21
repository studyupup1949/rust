use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use actix_web::cookie::Cookie;
use actix_web::dev::ServiceResponse;
use actix_web::test;
use actix_web::{App, HttpRequest, HttpResponse, web};
use serde_json::Value;

use actix_jwt::{ActixJwtMiddleware, JwtError, extract_claims, get_token};

fn cookie_authenticator() -> Arc<
    dyn Fn(
            &HttpRequest,
            &[u8],
        ) -> Pin<Box<dyn std::future::Future<Output = Result<Value, JwtError>> + Send>>
        + Send
        + Sync,
> {
    Arc::new(|_req, body| {
        let result = (|| -> Result<serde_json::Value, JwtError> {
            #[derive(serde::Deserialize)]
            struct Login {
                username: String,
                password: String,
            }
            let login: Login =
                serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;
            if login.username == "admin" && login.password == "admin" {
                Ok(serde_json::json!(login.username))
            } else {
                Err(JwtError::FailedAuthentication)
            }
        })();
        Box::pin(async move { result })
    })
}

fn cookie_payload_func() -> Arc<dyn Fn(&Value) -> HashMap<String, Value> + Send + Sync> {
    Arc::new(|data| {
        let mut claims = HashMap::new();
        claims.insert("identity".to_string(), data.clone());
        claims
    })
}

fn create_cookie_mw(
    send_cookie: bool,
    cookie_name: Option<&str>,
    refresh_cookie_name: Option<&str>,
) -> ActixJwtMiddleware {
    let mut mw = ActixJwtMiddleware::new();
    mw.realm = "test zone".to_string();
    mw.key = b"secret key salt".to_vec();
    mw.timeout = Duration::from_secs(3600);
    mw.max_refresh = Duration::from_secs(86400);
    mw.refresh_token_timeout = Duration::from_secs(86400);
    mw.authenticator = Some(cookie_authenticator());
    mw.payload_func = Some(cookie_payload_func());
    mw.send_cookie = send_cookie;
    if let Some(name) = cookie_name {
        mw.cookie_name = name.to_string();
    }
    if let Some(name) = refresh_cookie_name {
        mw.refresh_token_cookie_name = name.to_string();
    }
    mw
}

async fn create_cookie_app(
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
        "claims": claims,
        "token": token,
    }))
}

fn get_tokens_from_body(body: &[u8]) -> (String, Option<String>) {
    let v: Value = serde_json::from_slice(body).unwrap();
    let access = v["access_token"].as_str().unwrap().to_string();
    let refresh = v["refresh_token"].as_str().map(|s| s.to_string());
    (access, refresh)
}

fn collect_set_cookie_headers(resp: &ServiceResponse) -> Vec<String> {
    resp.headers()
        .get_all("Set-Cookie")
        .map(|v| v.to_str().unwrap().to_string())
        .collect()
}

// 1. TestSetRefreshTokenCookie
#[actix_web::test]
async fn test_set_refresh_token_cookie() {
    let mut mw = create_cookie_mw(true, None, Some("refresh_token"));
    mw.cookie_domain = Some("example.com".to_string());
    mw.secure_cookie = false;
    mw.cookie_http_only = true;
    mw.init().unwrap();

    let config = mw.refresh_cookie_config();
    assert_eq!(config.name, "refresh_token");
    assert!(config.secure); // refresh tokens always secure
    assert!(config.http_only); // refresh tokens always httpOnly
    assert_eq!(config.domain, Some("example.com".to_string()));
    assert!(config.max_age.as_secs() > 0);
}

// 2. TestSetRefreshTokenCookieDisabled
#[actix_web::test]
async fn test_set_refresh_token_cookie_disabled() {
    let mut mw = create_cookie_mw(false, None, None);
    mw.init().unwrap();
    let jwt = Arc::new(mw);

    let app = create_cookie_app(&jwt).await;

    // Login
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);

    // Should not set any cookies when SendCookie is false
    let cookies = collect_set_cookie_headers(&resp);
    assert!(
        cookies.is_empty(),
        "Should not set cookies when send_cookie=false"
    );
}

// 3. TestExtractRefreshTokenFromCookie
#[actix_web::test]
async fn test_extract_refresh_token_from_cookie() {
    let mut mw = create_cookie_mw(true, None, Some("refresh_token"));
    mw.init().unwrap();

    let req = test::TestRequest::get()
        .uri("/test")
        .cookie(Cookie::new(
            "refresh_token",
            "test-refresh-token-from-cookie",
        ))
        .to_http_request();

    let token = mw.extract_refresh_token(&req, b"");
    assert_eq!(token, Some("test-refresh-token-from-cookie".to_string()));
}

// 4. TestExtractRefreshTokenPriority
#[actix_web::test]
async fn test_extract_refresh_token_priority() {
    let mut mw = create_cookie_mw(true, None, Some("refresh_token"));
    mw.init().unwrap();

    // Cookie should have highest priority over form body
    let req = test::TestRequest::post()
        .uri("/test")
        .cookie(Cookie::new("refresh_token", "from-cookie"))
        .insert_header(("Content-Type", "application/x-www-form-urlencoded"))
        .to_http_request();

    let body = b"refresh_token=from-form";
    let token = mw.extract_refresh_token(&req, body);
    assert_eq!(
        token,
        Some("from-cookie".to_string()),
        "Cookie should have highest priority"
    );
}

// 5. TestLoginHandlerSetsRefreshTokenCookie
#[actix_web::test]
async fn test_login_handler_sets_refresh_token_cookie() {
    let mut mw = create_cookie_mw(true, Some("jwt"), Some("refresh_token"));
    mw.init().unwrap();
    let jwt = Arc::new(mw);

    let app = create_cookie_app(&jwt).await;

    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);

    let cookies = collect_set_cookie_headers(&resp);
    assert!(
        cookies.len() >= 2,
        "Should set at least 2 cookies, got {}",
        cookies.len()
    );

    let has_jwt = cookies.iter().any(|c| c.contains("jwt="));
    let has_refresh = cookies.iter().any(|c| c.contains("refresh_token="));

    assert!(has_jwt, "Should set JWT cookie");
    assert!(has_refresh, "Should set refresh token cookie");

    // Verify response contains tokens
    let body = test::read_body(resp).await;
    let (access_token, refresh_token) = get_tokens_from_body(&body);
    assert!(!access_token.is_empty());
    assert!(refresh_token.is_some());
}

// 6. TestRefreshHandlerWithCookie
#[actix_web::test]
async fn test_refresh_handler_with_cookie() {
    let mut mw = create_cookie_mw(true, Some("jwt"), Some("refresh_token"));
    mw.init().unwrap();
    let jwt = Arc::new(mw);

    let app = create_cookie_app(&jwt).await;

    // Login first
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    let body = test::read_body(resp).await;
    let (_, refresh_token) = get_tokens_from_body(&body);
    let refresh_token = refresh_token.unwrap();

    // Refresh with cookie
    let req = test::TestRequest::post()
        .uri("/refresh")
        .cookie(Cookie::new("refresh_token", &refresh_token))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);

    // Check new tokens returned
    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).unwrap();
    let new_access = v["access_token"].as_str();
    let new_refresh = v["refresh_token"].as_str();
    assert!(new_access.is_some(), "Should return new access token");
    assert!(new_refresh.is_some(), "Should return new refresh token");
    assert_ne!(
        refresh_token,
        new_refresh.unwrap(),
        "Refresh token should be rotated"
    );
}

// 7. TestRefreshHandlerWithoutCookie
#[actix_web::test]
async fn test_refresh_handler_without_cookie() {
    let mut mw = create_cookie_mw(true, Some("jwt"), None);
    mw.init().unwrap();
    let jwt = Arc::new(mw);

    let app = create_cookie_app(&jwt).await;

    // Login first
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    let body = test::read_body(resp).await;
    let (_, refresh_token) = get_tokens_from_body(&body);
    let refresh_token = refresh_token.unwrap();

    // Refresh with form data
    let req = test::TestRequest::post()
        .uri("/refresh")
        .insert_header(("Content-Type", "application/x-www-form-urlencoded"))
        .set_payload(format!("refresh_token={refresh_token}"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);

    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert!(v["access_token"].as_str().is_some());

    // Login again for JSON body test
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body = test::read_body(resp).await;
    let (_, refresh_token2) = get_tokens_from_body(&body);
    let refresh_token2 = refresh_token2.unwrap();

    // Refresh with JSON body
    let req = test::TestRequest::post()
        .uri("/refresh")
        .set_json(serde_json::json!({"refresh_token": refresh_token2}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);

    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert!(v["access_token"].as_str().is_some());
}

// 8. TestLogoutHandlerClearsRefreshTokenCookie
#[actix_web::test]
async fn test_logout_handler_clears_refresh_token_cookie() {
    let mut mw = create_cookie_mw(true, Some("jwt"), Some("refresh_token"));
    mw.init().unwrap();
    let jwt = Arc::new(mw);

    let app = create_cookie_app(&jwt).await;

    // Login first
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    let body = test::read_body(resp).await;
    let (access_token, refresh_token) = get_tokens_from_body(&body);
    let refresh_token = refresh_token.unwrap();

    // Logout with cookies
    let req = test::TestRequest::post()
        .uri("/logout")
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .cookie(Cookie::new("jwt", &access_token))
        .cookie(Cookie::new("refresh_token", &refresh_token))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);

    // Check that both cookies are cleared (Max-Age negative)
    let cookies = collect_set_cookie_headers(&resp);
    assert!(
        cookies.len() >= 2,
        "Should clear cookies, got {} Set-Cookie headers",
        cookies.len()
    );

    let has_jwt_clear = cookies
        .iter()
        .any(|c| c.contains("jwt=") && (c.contains("Max-Age=0") || c.contains("Max-Age=-1")));
    let has_refresh_clear = cookies.iter().any(|c| {
        c.contains("refresh_token=") && (c.contains("Max-Age=0") || c.contains("Max-Age=-1"))
    });

    assert!(has_jwt_clear, "Should clear JWT cookie");
    assert!(has_refresh_clear, "Should clear refresh token cookie");
}

// 9. TestRefreshTokenRevocationOnLogout
#[actix_web::test]
async fn test_refresh_token_revocation_on_logout() {
    let mut mw = create_cookie_mw(false, None, None);
    mw.init().unwrap();
    let jwt = Arc::new(mw);

    let app = create_cookie_app(&jwt).await;

    // Login
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);
    let body = test::read_body(resp).await;
    let (access_token, refresh_token) = get_tokens_from_body(&body);
    let refresh_token = refresh_token.unwrap();

    // Logout to revoke refresh token
    let req = test::TestRequest::post()
        .uri("/logout")
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .insert_header(("Content-Type", "application/x-www-form-urlencoded"))
        .set_payload(format!("refresh_token={refresh_token}"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);

    // Try to use revoked refresh token
    let req = test::TestRequest::post()
        .uri("/refresh")
        .insert_header(("Content-Type", "application/x-www-form-urlencoded"))
        .set_payload(format!("refresh_token={refresh_token}"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);
}

// 10. TestRefreshTokenCookieName
#[actix_web::test]
async fn test_refresh_token_cookie_name() {
    let custom_name = "my_refresh_token";
    let mut mw = create_cookie_mw(true, None, Some(custom_name));
    mw.init().unwrap();

    assert_eq!(mw.refresh_token_cookie_name, custom_name);

    let jwt = Arc::new(mw);
    let app = create_cookie_app(&jwt).await;

    // Login and check custom cookie name
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(serde_json::json!({"username": "admin", "password": "admin"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 200);

    let cookies = collect_set_cookie_headers(&resp);
    let has_custom = cookies
        .iter()
        .any(|c| c.contains(&format!("{custom_name}=")));
    assert!(has_custom, "Should use custom refresh token cookie name");
}

// 11. TestRefreshTokenCookieDefault
#[actix_web::test]
async fn test_refresh_token_cookie_default() {
    let mut mw = create_cookie_mw(true, None, None);
    // Don't set refresh_token_cookie_name - test the default
    mw.init().unwrap();

    assert_eq!(mw.refresh_token_cookie_name, "refresh_token");
}

// 12. TestTokenGeneratorSetsRefreshToken
#[actix_web::test]
async fn test_token_generator_sets_refresh_token() {
    let mut mw = create_cookie_mw(false, None, None);
    mw.init().unwrap();

    let user_data = serde_json::json!("admin");
    let token_pair = mw.token_generator(&user_data).await.unwrap();

    assert!(!token_pair.access_token.is_empty());
    assert!(token_pair.refresh_token.is_some());
    assert!(!token_pair.refresh_token.as_ref().unwrap().is_empty());
    assert_eq!(token_pair.token_type, "Bearer");
    assert!(token_pair.expires_at > 0);
    assert!(token_pair.created_at > 0);
}

// 13. TestExtractRefreshTokenContentType
#[actix_web::test]
async fn test_extract_refresh_token_json_body() {
    let mut mw = create_cookie_mw(false, None, Some("refresh_token"));
    mw.init().unwrap();

    let req = test::TestRequest::post()
        .uri("/test")
        .insert_header(("Content-Type", "application/json"))
        .to_http_request();

    let body = br#"{"refresh_token":"from-json-body"}"#;
    let token = mw.extract_refresh_token(&req, body);
    assert_eq!(
        token,
        Some("from-json-body".to_string()),
        "Should extract from JSON body with application/json Content-Type"
    );
}

#[actix_web::test]
async fn test_extract_refresh_token_form_body() {
    let mut mw = create_cookie_mw(false, None, Some("refresh_token"));
    mw.init().unwrap();

    let req = test::TestRequest::post()
        .uri("/test")
        .insert_header(("Content-Type", "application/x-www-form-urlencoded"))
        .to_http_request();

    let body = b"refresh_token=from-form-body";
    let token = mw.extract_refresh_token(&req, body);
    assert_eq!(
        token,
        Some("from-form-body".to_string()),
        "Should extract from form body"
    );
}

#[actix_web::test]
async fn test_extract_refresh_token_query_ignored() {
    let mut mw = create_cookie_mw(false, None, Some("refresh_token"));
    mw.init().unwrap();

    // Query params should be ignored for security; JSON body should win
    let req = test::TestRequest::post()
        .uri("/test?refresh_token=from-query")
        .insert_header(("Content-Type", "application/json"))
        .to_http_request();

    let body = br#"{"refresh_token":"from-json-body"}"#;
    let token = mw.extract_refresh_token(&req, body);
    assert_eq!(
        token,
        Some("from-json-body".to_string()),
        "Query parameter should be ignored for security, JSON body should be used"
    );
}

#[actix_web::test]
async fn test_extract_refresh_token_cookie_precedence() {
    let mut mw = create_cookie_mw(false, None, Some("refresh_token"));
    mw.init().unwrap();

    let req = test::TestRequest::post()
        .uri("/test")
        .insert_header(("Content-Type", "application/json"))
        .cookie(Cookie::new("refresh_token", "from-cookie"))
        .to_http_request();

    let body = br#"{"refresh_token":"from-json-body"}"#;
    let token = mw.extract_refresh_token(&req, body);
    assert_eq!(
        token,
        Some("from-cookie".to_string()),
        "Cookie should have highest precedence"
    );
}
