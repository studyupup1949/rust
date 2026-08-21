[![crates.io](https://img.shields.io/crates/v/actix-session-ext.svg)](https://crates.io/crates/actix-session-ext)
[![MIT licensed](https://img.shields.io/crates/l/actix-session-ext.svg)](./LICENSE)
[![Documentation](https://docs.rs/actix-session-ext/badge.svg)](https://docs.rs/actix-session-ext)
[![CI](https://github.com/alekece/actix-session-ext/actions/workflows/ci.yaml/badge.svg)](https://github.com/alekece/actix-session-ext/actions/workflows/ci.yaml)

<!-- cargo-sync-readme start -->

# actix-session-ext

The `actix-session-ext` crate provides a safer `actix_session::Session` interface thanks to typed key.

## Examples
```rust,no_run
use actix_web::{Error, Responder, HttpResponse};
use actix_session::Session;
use actix_session_ext::{SessionKey, SessionExt};

// create an actix application and attach the session middleware to it

const USER_KEY: SessionKey<String> = SessionKey::new("user");
const TIMESTAMP_KEY: SessionKey<u64> = SessionKey::new("timestamp");

#[actix_web::post("/login")]
async fn login(session: Session) -> Result<String, Error> {
    session.insert_by_key(USER_KEY, "Dupont".to_owned())?;
    session.insert_by_key(TIMESTAMP_KEY, 1234567890)?;

   Ok("logged in".to_owned())
}

#[actix_web::get("/logged_at")]
async fn logged_at(session: Session) -> Result<String, Error> {
   let timestamp = session.get_by_key(TIMESTAMP_KEY)?.unwrap_or_default();

   Ok(format!("logged at {}", timestamp))
}
```

<!-- cargo-sync-readme end -->

## License

Licensed under MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)
