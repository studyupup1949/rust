# actix-firebase-auth

A minimal, hard-fork of [firebase-auth](https://github.com/trchopan/firebase-auth), restructured for better compatibility within an [Actix Web](https://actix.rs/docs) ecosystem.

**NOTICE**: For most use cases, you're likely better served by using the [original firebase-auth crate](https://github.com/trchopan/firebase-auth), which has an active community and provides broader ecosystem support.

## Overview

This crate lets you verify Firebase ID tokens in Actix Web apps. It’s built to work smoothly with Actix’s async runtime, so you can easily protect your routes by checking that incoming requests carry valid Firebase authentication tokens.

## Installation

```bash
cargo add actix-firebase-auth
```

## Usage

The `FirebaseUser` struct implements Actix Web’s [FromRequest](https://docs.rs/actix-web/latest/actix_web/trait.FromRequest.html) trait, allowing seamless extraction directly within route handlers. When a route expects a `FirebaseUser`, the middleware automatically attempts to verify the Firebase ID token from the `Authorization` header.

If verification fails - due to a missing token, expiration, or invalid signature - the request is rejected with a `401 Unauthorized` response, ensuring protected routes remain secure by default.

### Example

See [/examples/server.rs](/examples/server.rs) for a minimal Actix Web server.

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
