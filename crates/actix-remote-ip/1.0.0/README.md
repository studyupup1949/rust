# Actix Remote IP extractor
[![Build Status](https://drone.communiquons.org/api/badges/pierre/actix-remote-ip/status.svg)](https://drone.communiquons.org/pierre/actix-remote-ip)
[![Crate](https://img.shields.io/crates/v/actix-remote-ip.svg)](https://crates.io/crates/actix-remote-ip)

Tiny extractor of remote user IP address, that handles reverse proxy.

The `X-Forwarded-For` header is automatically parsed when the request comes from a defined proxy, to determine the real remote client IP Address.

Note : regarding IPv6 addresses, the local part of the address is discarded. For example, the  IPv6 client `2001:0db8:85a3:0000:0000:8a2e:0370:7334` will be returned as `2001:0db8:85a3:0000:0000:0000:0000:0000`

## Configuration
Configure it when you configure your Actix server:

```rust
HttpServer::new(move || {
    App::new()
        .app_data(web::Data::new(RemoteIPConfig::parse_opt(Some("IP"))))
    // ...
})
```

## Usage
In your route, add a `RemoteIP` parameter:

```rust
#[get("/")]
async fn home(remote_ip: RemoteIP) -> HttpResponse {
    let ip: IpAddr = remote_ip.0;
    // ...
}
```