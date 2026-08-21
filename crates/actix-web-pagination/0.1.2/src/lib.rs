//! Extract pagination from http request
//!
//! # Example
//!
//! ```no_run
//! use actix_web::{App, HttpResponse, HttpServer};
//! use actix_web_pagination::Pagination;
//!
//! #[actix_web::get("/")]
//! async fn list(page: Pagination) -> HttpResponse {
//!     println!("page: {:?}", page);
//!     HttpResponse::Ok().finish()
//! }
//!
//! #[actix_web::main]
//! async fn main() -> actix_web::Result<()> {
//!     HttpServer::new(|| {
//!         App::new()
//!             .data(Pagination::config().default_per_page(50))
//!             .service(list)
//!     })
//!     .bind("0.0.0.0:8080")?
//!     .run()
//!     .await?;
//!
//!     Ok(())
//! }
//! ```

mod pagination;
mod pagination_config;
mod parse;

pub use pagination::Pagination;
pub use pagination_config::PaginationConfig;
