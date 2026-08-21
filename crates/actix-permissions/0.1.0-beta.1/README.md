# Actix Permissions [![Continuous Integration](https://github.com/eisberg-labs/actix-permissions/actions/workflows/ci.yml/badge.svg)](https://github.com/eisberg-labs/actix-permissions/actions/workflows/ci.yml) [![cargo-badge][]][cargo] [![license-badge][]][license] [![rust-version-badge][]][rust-version]

Permission and input validation extension for Actix Web. Alternative to actix guard, with access to app data injections, HttpRequest and Payload.

# Example
Dependencies:  
```toml
[dependencies]
actix-permissions = "0.1.0-beta.1"
```
Code:
```rust
use actix_permissions::{check, with};
use actix_web::dev::*;
use actix_web::web::Data;
use actix_web::*;
use serde::Serialize;
use std::future::{ready, Ready};

fn dummy_permission_check(
    req: &HttpRequest,
    _payload: &mut Payload,
) -> Ready<actix_web::Result<bool, actix_web::Error>> {
    let checker_service: Option<&Data<DummyService>> = req.app_data::<Data<DummyService>>();
    ready(Ok(checker_service.unwrap().check(req.query_string())))
}

fn another_dummy_permission_check(
    req: &HttpRequest,
    _payload: &mut Payload,
) -> Ready<actix_web::Result<bool, actix_web::Error>> {
    // Unecessary complicating permission check to show what it can do.
    // You have access to request, payload, and all injected dependencies through app_data.
    let checker_service: Option<&Data<DummyService>> = req.app_data::<Data<DummyService>>();
    // TODO: do not unwrap here
    ready(Ok(checker_service.unwrap().check(req.query_string())))
}

struct DummyService;

impl DummyService {
    pub fn check(&self, value: &str) -> bool {
        value.contains('q')
    }
}

async fn index() -> Result<String, Error> {
    Ok("Hi there!".to_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(Data::new(DummyService))
            .service(web::scope("").route(
                "/",
                check(
                    web::get(),
                    with(dummy_permission_check).and(another_dummy_permission_check),
                    index,
                ),
            ))
    })
    .bind("127.0.0.1:8888")?
    .run()
    .await
}
```


[cargo-badge]: https://img.shields.io/crates/v/actix-permissions.svg?style=flat-square
[cargo]: https://crates.io/crates/actix-permissions
[license-badge]: https://img.shields.io/badge/license-MIT/Apache--2.0-lightgray.svg?style=flat-square
[license]: #license
[rust-version-badge]: https://img.shields.io/badge/rust-1.15+-blue.svg?style=flat-square
[rust-version]: .travis.yml#L5
