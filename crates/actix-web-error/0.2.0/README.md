# actix-web-error

Error responses for [actix-web](https://actix.rs) made easy.
This crate will make it easy implementing [`actix_web::ResponseError`](https://docs.rs/actix-web/latest/actix_web/trait.ResponseError.html) for errors
by providing a `thiserror`-like API for specifying an HTTP status.
It's best used in combination with [thiserror](https://docs.rs/thiserror).

Currently, only a JSON response is supported.

Thanks to the aforementioned [thiserror](https://github.com/dtolnay/thiserror) project,
I used the core structure and core utilities.

## Error Responses

- `Json` will respond with JSON in the form of `{ "error": <Display representation> }` (`application/json`).
- `Text` will respond with the `Display` representation of the error (`text/plain`).

## Example

```rust
#[derive(Debug, thiserror::Error, actix_web_error::Json)]
#[status(BAD_REQUEST)] // default status for all variants
enum MyError {
    #[error("Missing: {0}")]
    MissingField(&'static str),
    #[error("Malformed Date")]
    MalformedDate,
    #[error("Internal Server Error")]
    #[status(500)] // specific override
    Internal,
}

#[derive(Debug, thiserror::Error, actix_web_error::Text)]
#[error("Item not found")]
#[status(404)]
struct MyOtherError;
```

This will roughly expand to:

```rust
use actix_web::{ResponseError, HttpResponse, HttpResponseBuilder, http::StatusCode};

#[derive(Debug, thiserror::Error)]
enum MyError {
    #[error("Missing: {0}")]
    MissingField(&'static str),
    #[error("Malformed Date")]
    MalformedDate,
    #[error("Internal Server Error")]
    Internal,
}

#[derive(Debug, thiserror::Error)]
#[error("Item not found")]
struct MyOtherError;

impl ResponseError for MyError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Internal => StatusCode::from_u16(500).unwrap(),
            _ => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .json(serde_json::json!({ "error": self.to_string() }))
    }
}

impl ResponseError for MyOtherError {
    fn status_code(&self) -> StatusCode {
        // With at least opt-level=1, this unwrap will be removed,
        // so this function will essentially return a constant.
        StatusCode::from_u16(404).unwrap()
    }
}
```
