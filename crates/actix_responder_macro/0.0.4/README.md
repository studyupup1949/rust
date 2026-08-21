# actix-responder-macro

An attribute macro to `transform` a handler response struct to an `actix responder`.
Keeps flexibility while adding more type safety.

The `actix_responder` adds 2 additional fields to your struct
`content_type` and `status_code`.
The `meta_attr` allows for arbitrary attributes to both fields.

`status_attr` applies only to `status_code` and `content_attr` applies only to `content_type`


The reason for this is like in the example below, if you use 
a crate like `TypedBuilder`, you might want to apply options like
`#[builder(default)]` to the generated field.

The macro always applies `#[serde(skip)]` to both generated fields 
so they won't show up in the request response. 

```rust
#[actix_responder(meta_attr = "builder(default)")]
#[derive(TypedBuilder, Serialize, Deserialize)]
pub struct SuccessResp {
    success: bool,
}
```


From this

```rust
#[get("/health_check")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok()
    .set_header(header::CONTENT_TYPE, mime::APPLICATION_JSON)
    .json(SuccessResp { success: true })
}
```

to this

```rust
#[get("/health_check")]
pub async fn health_check() -> SuccessResp {
    SuccessResp::builder()
    .success(true)
    .content_type(mime::APPLICATION_JSON::to_string())
    .build()
}
```


A more complicated example with setting default values

```rust
extern crate actix_responder_macro;
extern crate mime;
extern crate typed_builder;

use actix_responder_macro::actix_responder;
use actix_web::http::StatusCode;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[actix_responder(
    status_attr = "builder(default = StatusCode::INTERNAL_SERVER_ERROR)",
    content_attr = "builder(default = mime::IMAGE_BMP.to_string())"
)]
#[derive(Serialize, Deserialize, Debug, TypedBuilder, Default)]
pub struct ImageResp {...}
```
