# `actix-web-rest` - Opiniated toolkit to build restful API using actix-web.

## Why ?

:zap: Gotta go fast!

## Getting started

```shell
# Create a new project.
cargo init my_api

# Add actix-web-rest to your project.
cargo add actix-web-rest actix-web serde anyhow thiserror
```

Then overwrite `main.rs` with the following content:

```rust
use actix_web::{web, App, HttpResponse, HttpServer, ResponseError};
use actix_web_rest::{actix_web, http::StatusCode, rest_error};
use anyhow::anyhow;

#[allow(clippy::enum_variant_names)]
#[rest_error]
enum MyEndpointError {
    #[rest(status_code = StatusCode::OK)]
    #[error("error foo")]
    FooError,

    #[rest(status_code = StatusCode::OK)]
    #[error("error bar")]
    BarError,

    #[rest(status_code = StatusCode::INTERNAL_SERVER_ERROR)]
    #[error(transparent)]
    #[serde(skip)]
    UnexpectedError(#[from] anyhow::Error),
}

async fn handler(path: web::Path<String>) -> Result<HttpResponse, impl ResponseError> {
    let path_param = path.into_inner();
    match path_param.as_ref() {
        "foo" => Err(MyEndpointError::FooError),
        "bar" => Err(MyEndpointError::BarError),
        _ => Err(MyEndpointError::from(anyhow!(
            "unexpected path params: {path_param}"
        ))),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/{error}", web::get().to(handler)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
```

```shell
curl -i http://localhost:8080/foo
curl -i http://localhost:8080/bar
curl -i http://localhost:8080/baz
```

## Contributing

If you want to contribute to `actix-web-rest` to add a feature or improve the code contact
me at [negrel.dev@protonmail.com](mailto:negrel.dev@protonmail.com), open an
[issue](https://github.com/negrel/actix-web-rest/issues) or make a
[pull request](https://github.com/negrel/actix-web-rest/pulls).

## :stars: Show your support

Please give a :star: if this project helped you!

[![buy me a coffee](.github/images/bmc-button.png)](https://www.buymeacoffee.com/negrel)

## :scroll: License

MIT © [Alexandre Negrel](https://www.negrel.dev/)
