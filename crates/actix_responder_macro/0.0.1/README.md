# actix-responder-macro

An attribute macro to `transform` a handler response struct to an `actix responder`.
Keeps flexibility while adding more type safety.

```
//the meta_attr allows for arbitrary copy paste to the meta field
//the macro is adding to the struct
#[actix_responder(meta_attr = "builder(default)")]
#[derive(TypedBuilder, Serialize, Deserialize)]
struct SuccessResp {
    success: bool,
}
```


From this

```
#[get("/health_check")]
    pub async fn health_check() -> impl Responder {
        HttpResponse::Ok()
        .set_header(header::CONTENT_TYPE, mime::APPLICATION_JSON)
        .json(SuccessResp { success: true })
    }
```

to this

```
#[get("/health_check")]
    pub async fn health_check() -> SuccessResp {
        SuccessResp::builder().success(true).build()
    }
```

