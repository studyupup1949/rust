# actix-firebase-auth

Lightweight Firebase Authentication integration for [Actix Web](https://actix.rs/docs).

## ✨ Features

This crate provides an easy way to **verify Firebase ID tokens** and **extract authenticated users** in Actix Web applications. It includes:

- An Actix-compatible extractor to **automatically validate** and inject `FirebaseUser` into request handlers

- A strongly-typed interface to access decoded Firebase claims

- Optional feature flags for **Identity Provider** (**IdP**) helpers, such as support for extracting Google-specific identity claims (`idp-google`)

- Errors are mapped to appropriate HTTP status codes using Actix’s error conventions

- Authentication failures include the [WWW-Authenticate](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/WWW-Authenticate) header in the response, as specified by [RFC 7235](https://datatracker.ietf.org/doc/html/rfc7235#section-4.1), to ensure compatibility with HTTP authentication standards.

## 📦 Installation

```bash
cargo add actix-firebase-auth
```

## 🚀 Usage

The `FirebaseUser` struct implements Actix Web’s [FromRequest](https://docs.rs/actix-web/latest/actix_web/trait.FromRequest.html) trait, allowing seamless extraction directly within route handlers. When a route expects a `FirebaseUser`, the middleware automatically attempts to verify the Firebase ID token from the `Authorization` header.

If verification fails - due to a missing token, expiration, or invalid signature - the request is rejected with a `401 Unauthorized` response, ensuring protected routes remain secure by default.

### 💡 Example

See the [examples/server.rs](/examples/server.rs) for a minimal Actix Web server.

To run this example:

```bash
cargo run --example server
```

Make sure to include a valid Firebase ID token in the `Authorization` header when calling protected endpoints:

```http
GET /protected HTTP/1.1
Host: api.example.com
Authorization: Bearer <Firebase_ID_Token>
```

## 🧪 Testing

The crate includes a test suite covering:

- Emulator behavior
- Invalid tokens and malformed input
- Valid RS256 JWTs with mocked keys

To run the tests:

```bash
cargo test
```

## 🔗 Similar Projects

This crate is a hard-fork of [firebase-auth](https://github.com/trchopan/firebase-auth), rewritten for better compatibility within the [Actix Web](https://actix.rs/docs) ecosystem.

## ⚖️ License

Licensed under either of

- [MIT license](https://spdx.org/licenses/MIT.html) (see [LICENSE-MIT](/LICENSE-MIT)) or
- [Apache License, Version 2.0](https://spdx.org/licenses/Apache-2.0.html) (see [LICENSE-APACHE](/LICENSE-APACHE))

at your discretion.

## 🤝 Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
