# actix-firebase-auth

A minimal, hard-fork of [firebase-auth](https://github.com/trchopan/firebase-auth), restructured for better compatibility within an [Actix Web](https://actix.rs/docs) ecosystem.

**NOTICE**: For most use cases, you're likely better served by using the [original firebase-auth crate](https://github.com/trchopan/firebase-auth), which has an active community and provides broader ecosystem support.

## Overview

This crate lets you verify Firebase ID tokens in Actix Web apps. It’s built to work smoothly with Actix’s async runtime, so you can easily protect your routes by checking that incoming requests carry valid Firebase authentication tokens.

## Installation

Manually add the crate to your `Cargo.toml`:

```toml
actix-firebase-auth = { version = "0.1.0" }
```

Using **cargo**:

```bash
cargo add actix-firebase-auth
```

## Usage

The `FirebaseUser` struct implements Actix Web’s [FromRequest](https://docs.rs/actix-web/latest/actix_web/trait.FromRequest.html) trait, allowing seamless extraction directly within route handlers. When a route expects a `FirebaseUser`, the middleware automatically attempts to verify the Firebase ID token from the `Authorization` header.

If verification fails - due to a missing token, expiration, or invalid signature - the request is rejected with a `401 Unauthorized` response, ensuring protected routes remain secure by default.

### Example

#### Client-side

A web client must send requests in the following format:

```http
GET /whoami HTTP/1.1
Host: api.example.com
Authorization: Bearer <Firebase_ID_Token>
```

#### Server-side

```rust
use actix_web::{web, get, App, HttpServer, HttpResponse, Responder};
use actix_firebase_auth::{FirebaseAuth, FirebaseUser};

#[get("/whoami")]
async fn whoami(user: FirebaseUser) -> HttpResponse {
    HttpResponse::Ok().json(user)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let auth = FirebaseAuth::new("your-project-id").await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(auth.clone()))
            .service(whoami)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

## Testing

The crate includes a test suite covering:

- Emulator behavior
- Invalid tokens and malformed input
- Valid RS256 JWTs with mocked keys

To run the tests:

```bash
cargo test
```

## License

Licensed under either of

- [MIT license](https://spdx.org/licenses/MIT.html) (see [LICENSE-MIT](/LICENSE-MIT)) or
- [Apache License, Version 2.0](https://spdx.org/licenses/Apache-2.0.html) (see [LICENSE-APACHE](/LICENSE-APACHE))

at your discretion.

## Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
