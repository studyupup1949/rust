# actix-logger

drop-in replacement for the default actix web logger middleware 

this is a fork of [actix-contrib-logger](https://crates.io/crates/actix-contrib-logger) that changes the log level to `Warn` on redirection messages and client errors, and `Error` on server errors by default.

## Example

```rs
use actix_web::{App, HttpServer};
use actix_logger::Logger;

#[tokio::main]
async fn main() -> actix_web::Result<()> {
    femme::start();

    HttpServer::new(|| {
        App::new().wrap(Logger::default().service(/* */))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

for more info check the readme of the original project [here](https://github.com/mrsarm/rust-actix-contrib-logger#readme).
