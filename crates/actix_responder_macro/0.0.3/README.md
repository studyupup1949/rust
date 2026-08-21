# actix-responder-macro

An attribute macro to `transform` a handler response struct to an `actix responder`.
Keeps flexibility while adding more type safety.

The `actix_responder` adds an additional field to your struct which uses to set things
like `content_type` and `response_code`.
The `meta_attr` allows for arbitrary copy paste to the meta field
the macro is adding to the struct.

The reason for this is like in the example below, if you use 
a crate like `TypedBuilder`, you might want to apply options like
`#[builder(default)]` to the generated field.

The macro always applies `#[serde(skip)]` to the generated field 
so it won't show up in the request response. 

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
    SuccessResp::builder().success(true).build()
}
```


A more complicated example with setting default values

```rust
extern crate actix_responder_macro;
extern crate typed_builder;

use actix_responder_macro::actix_responder;
use actix_web::http::StatusCode;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[actix_responder(meta_attr = r#"builder(
        default = SuccessRespMetadata{
                status_code: Some(StatusCode::INTERNAL_SERVER_ERROR),
                content_type: Some("image/bmp".to_string())
            }
        )"#)]
#[derive(Serialize, Deserialize, Debug, TypedBuilder, Default)]
pub struct ImageResp {...}
```
