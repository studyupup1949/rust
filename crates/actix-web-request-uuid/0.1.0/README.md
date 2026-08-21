# actix-web-request-uuid

A Rust library to add request ID functionality with the actix-web framework.

[![CI](https://github.com/YusukeYoshida8849/actix-web-request-uuid/workflows/CI/badge.svg)](https://github.com/YusukeYoshida8849/actix-web-request-uuid/actions?query=workflow%3ACI)
[![crates.io](https://img.shields.io/crates/v/actix-web-request-uuid)](https://crates.io/crates/actix-web-request-uuid)
[![Documentation](https://docs.rs/actix-web-request-uuid/badge.svg)](https://docs.rs/actix-web-request-uuid)
[![License](https://img.shields.io/crates/l/actix-web-request-uuid)](https://github.com/YusukeYoshida8849/actix-web-request-uuid#license)

## Japanese

Please refer to [README.ja.md](./README.ja.md) .

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
actix-web-request-uuid = "0.1.0"
```

## Usage

Add this to your crate root:

```rust
use actix_web::{web, App, HttpServer, HttpResponse, Error};
use actix_web_request_uuid::RequestIDMiddleware;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(RequestIDMiddleware::new(36))
            .service(web::resource("/").to(|| HttpResponse::Ok()))
    })
    .bind("127.0.0.1:59880")?
    .run()
    .await
}
```

## Features

- Automatic request ID generation for each HTTP request
- Easy integration with actix-web middleware
- Lightweight and efficient implementation
- Compatible with actix-web ecosystem

### Changes from the Original Project

This project is based on [pastjean/actix-web-requestid](https://github.com/pastjean/actix-web-requestid) with significant feature enhancements:

#### 1. **Global Access via Thread-Local Variables**
Added request ID management functionality using Thread-Local variables that the original project lacked:

- `get_current_request_id()`: Access request ID from anywhere during request processing
- `set_current_request_id()`: Manually set request ID
- `clear_current_request_id()`: Clear request ID information

```rust
// Get request ID from any function
async fn my_service() -> Result<(), Error> {
    if let Some(request_id) = get_current_request_id() {
        log::info!("Processing request: {}", request_id);
    }
    Ok(())
}
```

#### 2. **Rich Customization Options**
While the original project only supported UUID v4, this version supports various formats:

- **Full UUID format**: `with_full_uuid()` - 36 characters with hyphens
- **Simple UUID format**: `with_simple_uuid()` - 32 characters without hyphens
- **Custom format**: `with_custom_uuid_format()` - Custom formatters
- **Custom generator**: `generator()` - Completely custom ID generation logic
- **Header name customization**: `header_name()` - Change default `request-id`

```rust
// Configuration example
let middleware = RequestIDMiddleware::new(32)
    .with_simple_uuid()
    .header_name("X-Request-ID")
    .generator(|| format!("req-{}", Uuid::new_v4().simple()));
```

#### 3. **Actix Web Extractor Support**
More intuitive API through FromRequest implementation that the original project lacked:

```rust
// Get request ID directly in handler functions
async fn show_id(request_id: RequestID) -> impl Responder {
    format!("Your request ID: {}", request_id)
}
```

#### 4. **Extension Traits**
Trait for directly getting request ID from HttpMessage:

```rust
pub trait RequestIDMessage {
    fn request_id(&self) -> RequestID;
}

// Usage example
fn my_middleware(req: &HttpRequest) {
    let id = req.request_id(); // Direct access
}
```

#### 5. **Error Handling and Robustness**
- **ID length validation**: Safe feature that panics on lengths ≤ 0
- **Existing ID reuse**: Reuse IDs stored in extensions
- **Automatic cleanup**: Automatic Thread-Local variable cleanup after request processing

#### 6. **Comprehensive Test Suite**
More detailed test coverage than the original project:

- Custom ID length tests
- UUID format-specific tests
- Custom header name tests
- Thread-Local functionality tests
- Error case tests

#### 7. **Rich Documentation**
- Detailed docstrings for all public functions
- Practical code samples
- Use case-specific explanations
- Best practices guide

These enhancements have significantly upgraded the original simple request ID generation middleware into an **enterprise-level request tracking system supporting log collection, distributed tracing, and debugging assistance**.

## Documentation

For detailed documentation and examples, please refer to the [docs.rs page](https://docs.rs/actix-web-request-uuid).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Original Project

This is a fork of [pastjean/actix-web-requestid](https://github.com/pastjean/actix-web-requestid).
We thank the original author for their contribution to the Rust community.
