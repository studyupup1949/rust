General purpose JWT session validator for actix_web

It’s designed to extract session using middleware and validate path simply by using extractors.

Examples usage:

```rust
use std::boxed::Box;
use std::sync::Arc;
use actix_jwt_session::*;
use actix_web::get;
use actix_web::web::Data;
use actix_web::{HttpResponse, App, HttpServer};
use ring::rand::SystemRandom;
use ring::signature::{Ed25519KeyPair, KeyPair};
use jsonwebtoken::*;
use serde::{Serialize, Deserialize};

#[tokio::main]
async fn main() {
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
    let factory = RedisMiddlewareFactory::<AppClaims>::new(
        Arc::new(keys.encoding_key),
        Arc::new(keys.decoding_key),
        Algorithm::EdDSA,
        redis.clone(),
        vec![
            // Check if header "Authorization" exists and contains Bearer with encoded JWT
            Box::new(HeaderExtractor::new("Authorization")),
            // Check if cookie "jwt" exists and contains encoded JWT
            Box::new(CookieExtractor::new("jwt")),
        ]
    );
  
    HttpServer::new(move || {
        let factory = factory.clone();
        App::new()
            .app_data(Data::new(factory.storage()))
            .wrap(factory)
            .app_data(Data::new(redis.clone()))
            .service(storage_access)
            .service(must_be_signed_in)
            .service(may_be_signed_in)
    })
    .bind(("0.0.0.0", 8080)).unwrap()
    .run()
    .await.unwrap();
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AppClaims {
    id: uuid::Uuid,
    subject: String,
}

impl Claims for AppClaims {
    fn jti(&self) -> uuid::Uuid { self.id }
    fn subject(&self) -> &str { &self.subject }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionData {
    id: uuid::Uuid,
    subject: String,
}

#[actix_web::post("/access-storage")]
async fn storage_access(
    session_store: Data<SessionStorage<AppClaims>>, 
    p: actix_web::web::Json<SessionData>,
) -> HttpResponse {
    let p = p.into_inner();
    session_store.store(AppClaims {
        id: p.id,
        subject: p.subject,
    }, std::time::Duration::from_secs(60 * 60 * 24 * 14) ).await.unwrap();
    HttpResponse::Ok().body("")
}

#[get("/authorized")]
async fn must_be_signed_in(session: Authenticated<AppClaims>) -> HttpResponse {
    let jit = session.jti();
    HttpResponse::Ok().body("")
}

#[get("/maybe-authorized")]
async fn may_be_signed_in(session: MaybeAuthenticated<AppClaims>) -> HttpResponse {
    if let Some(session) = session.into_option() {
    }
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
```
