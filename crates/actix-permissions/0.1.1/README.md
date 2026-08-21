# Actix Permissions [![Continuous Integration](https://github.com/eisberg-labs/actix-permissions/actions/workflows/ci.yml/badge.svg)](https://github.com/eisberg-labs/actix-permissions/actions/workflows/ci.yml) [![cargo-badge][]][cargo] [![license-badge][]][license]

Permission and input validation extension for Actix Web. Alternative to actix guard, with access to app data injections, HttpRequest and Payload.
Permissions are flexible, take a look at [Examples directory](./examples) for some of the use cases.

You could write a permission check like a function or like a struct.  
This code:
```rust
fn is_allowed(
    req: &HttpRequest,
    payload: &mut Payload,
) -> Ready<actix_web::Result<bool, actix_web::Error>> {
    todo!();
}
``` 
is same as writing:
```rust
struct IsAllowed;

impl Permission for IsAllowed {
    fn call(&self, req: &HttpRequest, _payload: &mut Payload) -> Ready<actix_web::Result<bool>> {
        todo!();
    }
}
```

# Example
Dependencies:  
```toml
[dependencies]
actix-permissions = "0.1.1"
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
## Use Cases
Take a look at [Examples directory](./examples).
You could use actix-permissions for role based authorization check, like in *role-based-authorization* example.  
*hello-world* example is just a proof of concept, showing how you can compose a list of permissions,
access service request, payload and injected services.

## Permission Deny
By default, 403 is returned for failed permission checks. You may want to toggle between `Unauthorized` and `Forbidden`,
maybe customize 403 forbidden messages. That's why `check_with_custom_deny` is for.
Take a look at [role based authorization example](./examples/role-based-authorization) for more info.

## Contributing

This project welcomes all kinds of contributions. No contribution is too small!

If you want to contribute to this project but don't know how to begin or if you need help with something related to this project, 
feel free to send me an email <https://www.eisberg-labs.com/> (contact form at the bottom).

Some pointers on contribution are in [Contributing.md](./CONTRIBUTING.md)

## Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).


# License

Distributed under the terms of [MIT license](./LICENSE-MIT) and [Apache license](./LICENSE-APACHE).

[cargo-badge]: https://img.shields.io/crates/v/actix-permissions.svg?style=flat-square
[cargo]: https://crates.io/crates/actix-permissions
[license-badge]: https://img.shields.io/badge/license-MIT/Apache--2.0-lightgray.svg?style=flat-square
[license]: #license
