![docs.rs](https://img.shields.io/docsrs/actix-jwt-session)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![builds.sr.ht status](https://builds.sr.ht/~tsumanu.svg)](https://builds.sr.ht/~tsumanu?)

All in one creating session and session validation library for actix.

It's designed to extract session using middleware and validate endpoint simply by using actix-web extractors.
Currently you can extract tokens from Header or Cookie. It's possible to implement Path, Query
or Body using `[ServiceRequest::extract]` but you must have struct to which values will be
extracted so it's easy to do if you have your own fields.

Example:

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct MyJsonBody {
    jwt: Option<String>,
    refresh: Option<String>,
}
```

To start with this library you need to create your own `AppClaims` structure and implement
`actix_jwt_session::Claims` trait for it.

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Audience {
    Web,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub struct Claims {
    #[serde(rename = "exp")]
    pub expiration_time: u64,
    #[serde(rename = "iat")]
    pub issues_at: usize,
    /// Account login
    #[serde(rename = "sub")]
    pub subject: String,
    #[serde(rename = "aud")]
    pub audience: Audience,
    #[serde(rename = "jti")]
    pub jwt_id: uuid::Uuid,
    #[serde(rename = "aci")]
    pub account_id: i32,
    #[serde(rename = "nbf")]
    pub not_before: u64,
}

impl actix_jwt_session::Claims for Claims {
    fn jti(&self) -> uuid::Uuid {
        self.jwt_id
    }

    fn subject(&self) -> &str {
        &self.subject
    }
}

impl Claims {
    pub fn account_id(&self) -> i32 {
        self.account_id
    }
}
```

Then you must create middleware factory with session storage. Currently there's adapter only
for redis so we will goes with it in this example.

* First create connection pool to redis using `redis_async_pool`.
* Next generate or load create jwt signing keys. They are required for creating JWT from
  claims.
* Finally pass keys and algorithm to builder, pass pool and add some extractors

```rust
use std::sync::Arc;
use actix_jwt_session::*;

async fn create<AppClaims: actix_jwt_session::Claims>() {
    // create redis connection
    let redis = 
        deadpool_redis::Config::from_url("redis://localhost:6379").create_pool(None).expect("Fail to connect to redis");
 
    // load or create new keys in `./config`
    let keys = JwtSigningKeys::load_or_create();

    // create new [SessionStorage] and [SessionMiddlewareFactory]
    let (storage, factory) = SessionMiddlewareFactory::<AppClaims>::build(
        Arc::new(keys.encoding_key),
        Arc::new(keys.decoding_key),
        Algorithm::EdDSA
    )
    // pass redis connection
    .with_redis_pool(redis.clone())
    .with_extractors(Extractors::default()
        // Check if header "Authorization" exists and contains Bearer with encoded JWT
        .with_jwt_header("Authorization")
        // Check if cookie "jwt" exists and contains encoded JWT
        .with_jwt_cookie("acx-a")
        .with_refresh_header("ACX-Refresh")
        // Check if cookie "jwt" exists and contains encoded JWT
        .with_refresh_cookie("acx-r")
    )
    .finish();
}
```

As you can see we have there [SessionMiddlewareBuilder::with_refresh_cookie] and [SessionMiddlewareBuilder::with_refresh_header]. Library uses
internal structure [RefreshToken] which is created and managed internally without any additional user work.

This will be used to extend JWT lifetime. This lifetime comes from 2 structures which describe
time to live. [JwtTtl] describes how long access token should be valid, [RefreshToken]
describes how long refresh token is valid. [SessionStorage] allows to extend livetime of both
with single call of [SessionStorage::refresh] and it will change time of creating tokens to
current time.

```rust
use actix_jwt_session::{JwtTtl, RefreshTtl, Duration};

fn example_ttl() {
    let jwt_ttl = JwtTtl(Duration::days(14));
    let refresh_ttl = RefreshTtl(Duration::days(3 * 31));
}
```

Now you just need to add those structures to [actix_web::App] using `.app_data` and `.wrap` and
you are ready to go. Bellow you have full example of usage.

Examples usage:

```rust
use std::sync::Arc;
use actix_jwt_session::*;
use actix_web::{get, post};
use actix_web::web::{Data, Json};
use actix_web::{HttpResponse, App, HttpServer};
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
 
    let jwt_ttl = JwtTtl(Duration::days(14));
    let refresh_ttl = RefreshTtl(Duration::days(3 * 31));
 
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new( jwt_ttl ))
            .app_data(Data::new( refresh_ttl ))
            .use_jwt(
                Extractors::default()
                // Check if header "Authorization" exists and contains Bearer with encoded JWT
                .with_jwt_header(JWT_HEADER_NAME)
                // Check if cookie JWT exists and contains encoded JWT
                .with_jwt_cookie(JWT_COOKIE_NAME)
                .with_refresh_header(REFRESH_HEADER_NAME)
                // Check if cookie JWT exists and contains encoded JWT
                .with_refresh_cookie(REFRESH_COOKIE_NAME)
            )
            .app_data(Data::new(redis.clone()))
            .service(must_be_signed_in)
            .service(may_be_signed_in)
            .service(register)
            .service(sign_in)
            .service(sign_out)
            .service(refresh_session)
            .service(session_info)
            .service(root)
    })
    .bind(("0.0.0.0", 8080)).unwrap()
    .run()
    .await.unwrap();
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionData {
    id: uuid::Uuid,
    subject: String,
}

#[get("/authorized")]
async fn must_be_signed_in(session: Authenticated<AppClaims>) -> HttpResponse {
    use crate::actix_jwt_session::Claims;
    let jit = session.jti();
    HttpResponse::Ok().finish()
}

#[get("/maybe-authorized")]
async fn may_be_signed_in(session: MaybeAuthenticated<AppClaims>) -> HttpResponse {
    if let Some(session) = session.into_option() {
    }
    HttpResponse::Ok().finish()
}

#[derive(Deserialize)]
struct SignUpPayload {
    login: String,
    password: String,
    password_confirmation: String,
}

#[post("/session/sign-up")]
async fn register(payload: Json<SignUpPayload>) -> Result<HttpResponse, actix_web::Error> {
    let payload = payload.into_inner();
    
    // Validate payload
    
    // Save model and return HttpResponse
    let model = AccountModel {
        id: -1,
        login: payload.login,
        // Encrypt password before saving to database
        pass_hash: Hashing::encrypt(&payload.password).unwrap(),
    };
    // Save model

    todo!()
}

#[derive(Deserialize)]
struct SignInPayload {
    login: String,
    password: String,
}

#[post("/session/sign-in")]
async fn sign_in(
    store: Data<SessionStorage>,
    payload: Json<SignInPayload>,
    jwt_ttl: Data<JwtTtl>,
    refresh_ttl: Data<RefreshTtl>,
) -> Result<HttpResponse, actix_web::Error> {
    let payload = payload.into_inner();
    let store = store.into_inner();
    let account: AccountModel = {
        /* load account using login */
         todo!()
    };
    if let Err(e) = Hashing::verify(account.pass_hash.as_str(), payload.password.as_str()) {
        return Ok(HttpResponse::Unauthorized().finish());
    }
    let claims = AppClaims {
         issues_at: OffsetDateTime::now_utc().unix_timestamp() as usize,
         subject: account.login.clone(),
         expiration_time: jwt_ttl.0.as_seconds_f64() as u64,
         audience: Audience::Web,
         jwt_id: uuid::Uuid::new_v4(),
         account_id: account.id,
         not_before: 0,
    };
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
async fn sign_out(store: Data<SessionStorage>, auth: Authenticated<AppClaims>) -> HttpResponse {
    let store = store.into_inner();
    store.erase::<AppClaims>(auth.jwt_id).await.unwrap();
    HttpResponse::Ok()
        .append_header((JWT_HEADER_NAME, ""))
        .append_header((REFRESH_HEADER_NAME, ""))
        .cookie(
            actix_web::cookie::Cookie::build(JWT_COOKIE_NAME, "")
                .expires(OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(REFRESH_COOKIE_NAME, "")
                .expires(OffsetDateTime::now_utc())
                .finish(),
        )
        .finish()
}

#[get("/session/info")]
async fn session_info(auth: Authenticated<AppClaims>) -> HttpResponse {
    HttpResponse::Ok().json(&*auth)
}

#[get("/session/refresh")]
async fn refresh_session(
    auth: Authenticated<RefreshToken>,
    storage: Data<SessionStorage>,
) -> HttpResponse {
    let storage = storage.into_inner();
    storage.refresh(auth.refresh_jti).await.unwrap();
    HttpResponse::Ok().json(&*auth)
}

#[get("/")]
async fn root() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Audience {
    Web,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub struct AppClaims {
    #[serde(rename = "exp")]
    pub expiration_time: u64,
    #[serde(rename = "iat")]
    pub issues_at: usize,
    /// Account login
    #[serde(rename = "sub")]
    pub subject: String,
    #[serde(rename = "aud")]
    pub audience: Audience,
    #[serde(rename = "jti")]
    pub jwt_id: uuid::Uuid,
    #[serde(rename = "aci")]
    pub account_id: i32,
    #[serde(rename = "nbf")]
    pub not_before: u64,
}

impl actix_jwt_session::Claims for AppClaims {
    fn jti(&self) -> uuid::Uuid {
        self.jwt_id
    }

    fn subject(&self) -> &str {
        &self.subject
    }
}

impl AppClaims {
    pub fn account_id(&self) -> i32 {
        self.account_id
    }
}

struct AccountModel {
    id: i32,
    login: String,
    pass_hash: String,
}
```

# Changelog:

1.0.0

* Factory is created using builder pattern
* JSON Web Token has automatically created Refresh Token
* Higher abstraction layers for Middleware, MiddlewareFactory and SessionStorage
* Build-in hashing functions
* Build-in TTL structures
* Documentation

1.0.1

* Returns new pair after refresh lifetime

1.0.2

* License file
* Categories and tags
* Repository
* Bug tracker
* Test builds
* Badges

# Bug tracker

https://todo.sr.ht/~tsumanu/actix-jwt-session
