use std::sync::Arc;

use actix_jwt_session::*;
use actix_web::dev::ServiceResponse;
use actix_web::http::header::ContentType;
use actix_web::http::{Method, StatusCode};
use actix_web::web::{Data, Json};
use actix_web::{get, post, test, App, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Claims {
    id: Uuid,
    subject: String,
}

impl actix_jwt_session::Claims for Claims {
    fn jti(&self) -> Uuid {
        self.id
    }
    fn subject(&self) -> &str {
        &self.subject
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn full_flow() {
    let redis = {
        use deadpool_redis::{Config, Runtime};
        Config::from_url("redis://localhost:6379")
            .create_pool(Some(Runtime::Tokio1))
            .unwrap()
    };

    let keys = JwtSigningKeys::generate(false).unwrap();
    let (storage, factory) = SessionMiddlewareFactory::<Claims>::build(
        Arc::new(keys.encoding_key),
        Arc::new(keys.decoding_key),
        Algorithm::EdDSA,
    )
    .with_redis_pool(redis.clone())
    .with_extractors(
        Extractors::default()
            .with_jwt_header(JWT_HEADER_NAME)
            .with_refresh_header(REFRESH_HEADER_NAME)
            .with_jwt_cookie(JWT_COOKIE_NAME)
            .with_refresh_cookie(REFRESH_COOKIE_NAME),
    )
    .finish();

    let app = App::new()
        .app_data(Data::new(storage.clone()))
        .wrap(factory.clone())
        .app_data(Data::new(redis.clone()))
        .app_data(Data::new(JwtTtl(Duration::seconds(1))))
        .app_data(Data::new(RefreshTtl(Duration::seconds(30))))
        .service(sign_in)
        .service(sign_out)
        .service(session)
        .service(refresh_session)
        .service(root);

    let app = actix_web::test::init_service(app).await;

    // -----------------------------------------------------------------------------
    //              Assert authorization is ignored when token is not needed
    // -----------------------------------------------------------------------------
    let res = test::call_service(
        &app,
        test::TestRequest::default()
            .insert_header(ContentType::plaintext())
            .to_request(),
    )
    .await;
    assert!(res.status().is_success());

    // -----------------------------------------------------------------
    //              Assert signed out when active session
    // -----------------------------------------------------------------
    let res = test::call_service(&app, session_request("", "").to_request()).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let origina_claims = Claims {
        id: Uuid::new_v4(),
        subject: "foo".to_string(),
    };

    // ----------------------------------------------
    //              Create session
    // ----------------------------------------------
    println!("-> Creating session");
    let res = test::call_service(
        &app,
        test::TestRequest::default()
            .uri("/session/sign-in")
            .method(actix_web::http::Method::POST)
            .insert_header(ContentType::json())
            .set_json(&origina_claims)
            .to_request(),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    println!("  <- OK");

    let auth_bearer = res
        .headers()
        .get(JWT_HEADER_NAME)
        .unwrap()
        .to_str()
        .unwrap();
    let refresh_bearer = res
        .headers()
        .get(REFRESH_HEADER_NAME)
        .unwrap()
        .to_str()
        .unwrap();

    // ----------------------------------------------
    //              Assert signed in
    // ----------------------------------------------
    println!("-> Assert signed in");
    let res = test::call_service(
        &app,
        session_request(&auth_bearer, &refresh_bearer).to_request(),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    println!("  <- OK");

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // ----------------------------------------------
    //              Access Token TTL expires
    // ----------------------------------------------
    println!("-> Access Token TTL expires");
    let res = test::try_call_service(
        &app,
        session_request(&auth_bearer, &refresh_bearer).to_request(),
    )
    .await;
    expect_invalid_session(res);
    println!("  <- OK");

    // ----------------------------------------------
    //                     Refresh token
    // ----------------------------------------------
    println!("-> Refresh token");
    let res = test::call_service(
        &app,
        test::TestRequest::default()
            .uri("/session/refresh")
            .method(Method::GET)
            .insert_header((REFRESH_HEADER_NAME, refresh_bearer))
            .to_request(),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    println!("  <- OK");

    // ----------------------------------------------
    //                     Logout
    // ----------------------------------------------
    println!("-> Logout");
    let res = test::call_service(
        &app,
        test::TestRequest::default()
            .uri("/session/sign-out")
            .method(Method::POST)
            .insert_header((JWT_HEADER_NAME, auth_bearer))
            .insert_header((REFRESH_HEADER_NAME, refresh_bearer))
            .to_request(),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    println!("  <- OK");

    // --------------------------------------------------------------
    //              Assert signed out - session destroyed
    // --------------------------------------------------------------
    println!("-> Assert signed out - session destroyed");
    let res = test::try_call_service(
        &app,
        session_request(&auth_bearer, &refresh_bearer).to_request(),
    )
    .await;
    expect_invalid_session(res);
    println!("  <- OK");
}

#[post("/session/sign-in")]
async fn sign_in(
    store: Data<SessionStorage>,
    claims: Json<Claims>,
    jwt_ttl: Data<JwtTtl>,
    refresh_ttl: Data<RefreshTtl>,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = claims.into_inner();
    let store = store.into_inner();
    let pair = store
        .clone()
        .store(claims, *jwt_ttl.into_inner(), *refresh_ttl.into_inner())
        .await
        .unwrap();
    Ok(HttpResponse::Ok()
        .append_header((JWT_HEADER_NAME, pair.jwt.encode().unwrap()))
        .append_header((REFRESH_HEADER_NAME, pair.refresh.encode().unwrap()))
        .finish())
}

#[post("/session/sign-out")]
async fn sign_out(store: Data<SessionStorage>, auth: Authenticated<Claims>) -> HttpResponse {
    let store = store.into_inner();
    store.erase::<Claims>(auth.id).await.unwrap();
    HttpResponse::Ok().finish()
}

#[get("/session/info")]
async fn session(auth: Authenticated<Claims>) -> HttpResponse {
    HttpResponse::Ok().json(&*auth)
}

#[get("/session/refresh")]
async fn refresh_session(
    auth: Authenticated<RefreshToken>,
    storage: Data<SessionStorage>,
) -> HttpResponse {
    let storage = storage.into_inner();
    storage.refresh::<Claims>(auth.refresh_jti).await.unwrap();
    HttpResponse::Ok().json(&*auth)
}

#[get("/")]
async fn root() -> HttpResponse {
    HttpResponse::Ok().finish()
}

fn session_request(auth_bearer: &str, refresh_bearer: &str) -> actix_web::test::TestRequest {
    let req = test::TestRequest::default()
        .uri("/session/info")
        .method(Method::GET);
    if !auth_bearer.is_empty() {
        req.insert_header((JWT_HEADER_NAME, auth_bearer))
            .insert_header((REFRESH_HEADER_NAME, refresh_bearer))
    } else {
        req
    }
}

fn expect_invalid_session(res: Result<ServiceResponse, actix_web::Error>) {
    let err = res
        .expect_err("Must be unauthorized")
        .as_error::<actix_jwt_session::Error>()
        .expect("Must be authorization error")
        .clone();
    assert_eq!(err, actix_jwt_session::Error::LoadError);
}
