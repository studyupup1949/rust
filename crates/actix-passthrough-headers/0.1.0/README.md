# Actix-passthrough-headers

Middleware to passthrough headers. It will be useful when you want to pass header from request to
response without changes.

## Installation

```bash
cargo add actix-passthrough-headers
```

## Usage

```rs
App::new().wrap(PassthroughHeaders::new(vec!["X-Request-Id"]))
```

## Options

By default when route returns some header:

```rs
async fn route() -> HttpResponse {
	HttpResponse::Ok()
		.insert_header(("X-Request-Id", "2"))
		.finish()
}
```

And the same header was passed to middleware:

```rs
App::new().wrap(PassthroughHeaders::new(["X-Request-Id"]))
```

The middleware ignores route's header and returns initial header value. It means when http request
has header `X-Request-Id: 1` and route returns `X-Request-Id: 2`, response header will be
`X-Request-Id: 1`.

### Method preserve_response_headers()

Method `preserve_response_headers` disallow to rewrite route's headers. It means when http request
has header `X-Request-Id: 1` and route returns `X-Request-Id: 2`, response header will be
`X-Request-Id: 2`.

```rs
App::new().wrap(PassthroughHeaders::new(vec!["X-Request-Id"]).preserve_response_headers())
```

## Examples

### Simple Example

```rs
use actix_web::{App, HttpServer};
use actix_passthrough_headers::PassthroughHeaders;

#[actix_web::main]
async fn main() {
	HttpServer::new(|| {
		App::new().wrap(PassthroughHeaders::new(vec!["X-Request-Id"]))
	})
		.bind(("127.0.0.1", 8080)).unwrap()
		.run()
		.await;
}
```

### Preserve response headers example

```rs
use actix_web::{App, HttpServer};
use actix_passthrough_headers::PassthroughHeaders;

async fn route() -> HttpResponse {
	HttpResponse::Ok().insert_header(("X-Request-Id", "1")).finish()
}

#[actix_web::main]
async fn main() {
	HttpServer::new(|| {
		App::new()
			.wrap(PassthroughHeaders::new(vec!["X-Request-Id"]).preserve_response_headers())
			.route("/", web::get().to(route))
	})
		.bind(("127.0.0.1", 8080)).unwrap()
		.run()
		.await;
}
```
