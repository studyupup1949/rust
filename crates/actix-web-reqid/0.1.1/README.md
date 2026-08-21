# actix-web-reqid

Tool to work with Request ID(UUID).

This crate is based on [timtonk/actix-web-middleware-requestid](https://github.com/timtonk/actix-web-middleware-requestid).

## Usage

Add the middleware to your Actix Web application to automatically generate and attach a UUID request ID to each incoming request.

### 1. Add the middleware

```rust
use actix_web::{App, HttpServer};
use actix_web_reqid::RequestIDWrapper;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(RequestIDWrapper)
            // your routes here
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

### 2. Extract the request ID in your handler

```rust
use actix_web::{get, HttpResponse, Responder};
use actix_web_reqid::RequestID;

#[get("/")]
async fn index(request_id: RequestID) -> impl Responder {
    HttpResponse::Ok().body(format!("Request ID: {}", request_id.0))
}
```

### 3. Error handling

If the request ID is missing, the extractor will return a `400 Bad Request` error.

---

This middleware helps you trace requests by assigning a unique UUID to each request and making it accessible in your handlers.