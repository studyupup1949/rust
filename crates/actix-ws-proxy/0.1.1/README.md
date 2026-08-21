[![builds.sr.ht status](https://builds.sr.ht/~liz/actix-ws-proxy.svg)](https://builds.sr.ht/~liz/actix-ws-proxy?)
[![crates.io](https://img.shields.io/crates/v/actix-web?label=latest)](https://crates.io/crates/actix-ws-proxy)
[![Documentation](https://docs.rs/actix-web/badge.svg)](https://docs.rs/actix-web/4.4.1)

# actix-ws-proxy
A companion to [actix-proxy] that handles websockets.

## Example

```rust
use actix_web::{Error, get, HttpRequest, HttpResponse, web};
#[get("/proxy/{port}")]
async fn proxy(
    req: HttpRequest,
    stream: web::Payload,
    port: web::Path<u16>,
) -> Result<HttpResponse, Error> {
    actix_ws_proxy::start(&req, format!("ws://127.0.0.1:{}", port), stream).await
}
```

Any errors will result in a disconnect on both sides with the error message as the reason.

[actix-proxy]: https://crates.io/crates/actix-proxy
