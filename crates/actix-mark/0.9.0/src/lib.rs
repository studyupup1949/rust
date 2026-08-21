//! `actix_mark` — serve Markdown files as HTML from an actix-web application.
//!
//! Drop `MarkdownFiles` into your app the same way you would `actix_files::Files`.
//! Requests for `.md` content are intercepted, converted to HTML with
//! [pulldown-cmark](https://docs.rs/pulldown-cmark), and injected into your
//! HTML template at the `<markdown>` tag.  All other files in the directory
//! are served as-is.
//!
//! # Quick start
//!
//! ```no_run
//! use actix_web::{App, HttpServer};
//! use actix_mark::MarkdownFiles;
//!
//! const TEMPLATE: &str = "<html><body><markdown></body></html>";
//!
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//!     HttpServer::new(|| {
//!         App::new().service(
//!             MarkdownFiles::new("/docs", "./content")
//!                 .template(TEMPLATE),
//!         )
//!     })
//!     .bind("127.0.0.1:8080")?
//!     .run()
//!     .await
//! }
//! ```

mod markdown;
mod service;
mod template;

pub use service::{md_url, MarkdownFiles};
