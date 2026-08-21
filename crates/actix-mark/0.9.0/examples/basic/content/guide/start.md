# Getting Started

Add `actix_mark` to your `Cargo.toml`:

```toml
[dependencies]
actix-web = "4"
actix_mark = "0.1"
```

## Minimal example

```rust
use actix_web::{App, HttpServer};
use actix_mark::MarkdownFiles;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(
            MarkdownFiles::new("/docs", "./content")
                .template(include_str!("template.html")),
        )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Template format

Your template must contain a `<markdown>` tag:

```html
<!DOCTYPE html>
<html>
  <head><link rel="stylesheet" href="/docs/style.css"></head>
  <body>
    <main><markdown></main>
  </body>
</html>
```

[← Back to index](..)
