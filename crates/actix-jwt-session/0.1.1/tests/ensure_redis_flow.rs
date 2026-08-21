use std::sync::Arc;

use actix_jwt_session::{
    Authenticated, HeaderExtractor, RedisMiddlewareFactory, SessionStorage, DEFAULT_HEADER_NAME,
};
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use actix_web::{get, post};
use actix_web::{http::header::ContentType, test, App};
use jsonwebtoken::*;
use ring::rand::SystemRandom;
use ring::signature::{Ed25519KeyPair, KeyPair};
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
async fn not_authenticated() {
    let redis = {
        use redis_async_pool::{RedisConnectionManager, RedisPool};
        RedisPool::new(
            RedisConnectionManager::new(
                redis::Client::open("redis://localhost:6379").expect("Fail to connect to redis"),
                true,
                None,
            ),
            5,
        )
    };

    let keys = JwtSigningKeys::generate().unwrap();
    let factory = RedisMiddlewareFactory::<Claims>::new(
        Arc::new(keys.encoding_key),
        Arc::new(keys.decoding_key),
        Algorithm::EdDSA,
        redis.clone(),
        vec![Box::new(HeaderExtractor::new(DEFAULT_HEADER_NAME))],
    );

    let app = App::new()
        .app_data(Data::new(factory.storage()))
        .wrap(factory.clone())
        .app_data(Data::new(redis.clone()))
        .service(sign_in)
        .service(sign_out)
        .service(session)
        .service(root);

    let app = actix_web::test::init_service(app).await;

    let res = test::call_service(
        &app,
        test::TestRequest::default()
            .insert_header(ContentType::plaintext())
            .to_request(),
    )
    .await;
    assert!(res.status().is_success());

    let res = test::call_service(
        &app,
        test::TestRequest::default()
            .uri("/s")
            .insert_header(ContentType::plaintext())
            .to_request(),
    )
    .await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let origina_claims = Claims {
        id: Uuid::new_v4(),
        subject: "foo".to_string(),
    };
    let res = test::call_service(
        &app,
        test::TestRequest::default()
            .uri("/in")
            .method(actix_web::http::Method::POST)
            .insert_header(ContentType::json())
            .set_json(&origina_claims)
            .to_request(),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[post("/in")]
async fn sign_in(
    store: Data<SessionStorage<Claims>>,
    claims: Json<Claims>,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = claims.into_inner();
    let store = store.into_inner();
    store
        .clone()
        .set_by_jti(claims, std::time::Duration::from_secs(300))
        .await
        .unwrap();
    Ok(HttpResponse::Ok().body(""))
}

#[post("/out")]
async fn sign_out(_store: Data<SessionStorage<Claims>>) -> HttpResponse {
    HttpResponse::Ok().body("")
}

#[get("/s")]
async fn session(auth: Authenticated<Claims>) -> HttpResponse {
    HttpResponse::Ok().json(&*auth)
}

#[get("/")]
async fn root() -> HttpResponse {
    HttpResponse::Ok().body("")
}

pub struct JwtSigningKeys {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtSigningKeys {
    fn generate() -> Result<Self, Box<dyn std::error::Error>> {
        let doc = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new())?;
        let keypair = Ed25519KeyPair::from_pkcs8(doc.as_ref())?;
        let encoding_key = EncodingKey::from_ed_der(doc.as_ref());
        let decoding_key = DecodingKey::from_ed_der(keypair.public_key().as_ref());
        Ok(JwtSigningKeys {
            encoding_key,
            decoding_key,
        })
    }
}
