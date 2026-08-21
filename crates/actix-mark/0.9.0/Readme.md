# actix-mark

[![Crates.io](https://img.shields.io/crates/v/actix-mark.svg)](https://crates.io/crates/actix-mark)
[![docs.rs](https://docs.rs/actix-mark/badge.svg)](https://docs.rs/actix-mark)
[![License](https://img.shields.io/crates/l/actix-mark.svg)](LICENSE)

An [actix-web](https://github.com/actix/actix-web) service for serving Markdown files as HTML. Drop it into your app the same way you would `actix-files::Files`: point it at a directory, give it an HTML template, and it will render every `.md` file on the fly. Non-Markdown files (CSS, images, …) are passed through unchanged.

## Features

- **Template-based rendering** — provide any HTML file with a `<markdown>` tag; the rendered content is injected there
- **Clean URLs** — `/docs/guide` resolves to `guide.md`; the `.md` extension also works directly
- **Nested directories** — subdirectories are handled recursively; `/docs/` resolves to `index.md`
- **Static asset pass-through** — CSS, images, and other files in the same directory are served as-is
- **Markdown extensions** — tables, strikethrough, footnotes, and task lists enabled via [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark)
- **Path traversal protection** — requests cannot escape the served directory

## Installation

```toml
[dependencies]
actix-web = "4"
actix-mark = "0.9"
```

## Usage

### Minimal example

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

### Loading the template from a file at startup

```rust
MarkdownFiles::new("/docs", "./content")
    .template_file("./template.html")?
```

`template_file` reads the file eagerly, so a missing or unreadable template is reported at startup rather than on the first request.

## Template format

The template is any valid HTML file containing a `<markdown>` tag (self-closing variants `<markdown/>` and `<markdown />` are also accepted). The tag is replaced with the rendered HTML.

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>My docs</title>
    <link rel="stylesheet" href="/docs/style.css">
</head>
<body>
    <main>
        <markdown>
    </main>
</body>
</html>
```

The stylesheet can live in the same content directory and will be served as a static file.

## URL mapping

| Request URL         | File served             |
|---------------------|-------------------------|
| `/docs/`            | `<dir>/index.md`        |
| `/docs/guide`       | `<dir>/guide.md`        |
| `/docs/guide.md`    | `<dir>/guide.md`        |
| `/docs/api/intro`   | `<dir>/api/intro.md`    |
| `/docs/style.css`   | `<dir>/style.css` (static) |
| anything else       | 404                     |

## Running the example

The repository includes a working example with sample Markdown pages and a stylesheet:

```sh
cargo run --example basic
```

Then open [http://localhost:8080/docs/](http://localhost:8080/docs/).

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
