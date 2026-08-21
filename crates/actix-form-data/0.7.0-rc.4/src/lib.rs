/*
 * This file is part of Actix Form Data.
 *
 * Copyright © 2026 asonix
 *
 * Actix Form Data is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Actix Form Data is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Actix Form Data.  If not, see <http://www.gnu.org/licenses/>.
 */

//! # Actix Form Data
//! A library for retrieving form data from Actix Web's multipart streams. It can stream
//! uploaded files onto the filesystem (its main purpose), but it can also parse associated
//! form data.
//!
//! # Example
//!
//!```rust
//! use actix_form_data::{Error, Field, Form, FormData, Multipart, Value};
//! use actix_web::{
//!     web::{post, resource},
//!     App, HttpRequest, HttpResponse, HttpServer,
//! };
//! use futures_util::stream::StreamExt;
//!
//! struct UploadedContent(Value<()>);
//!
//! impl FormData for UploadedContent {
//!     type Item = ();
//!     type Error = Error;
//!
//!     fn form(_: &HttpRequest) -> Result<Form<Self::Item, Self::Error>, Self::Error> {
//!         Ok(Form::new()
//!             .field("Hey", Field::text())
//!             .field(
//!                 "Hi",
//!                 Field::map()
//!                     .field("One", Field::int())
//!                     .field("Two", Field::float())
//!                     .finalize(),
//!             )
//!             .field(
//!                 "files",
//!                 Field::array(Field::file(async move |_, _, mut stream| {
//!                     while let Some(res) = stream.next().await {
//!                         res?;
//!                     }
//!                     Ok(()) as Result<(), Error>
//!                 })),
//!             ))
//!     }
//!
//!     fn extract(value: Value<Self::Item>) -> Result<Self, Self::Error>
//!     where
//!         Self: Sized,
//!     {
//!         Ok(UploadedContent(value))
//!     }
//! }
//!
//! async fn upload(Multipart(UploadedContent(value)): Multipart<UploadedContent>) -> HttpResponse {
//!     println!("Uploaded Content: {:#?}", value);
//!     HttpResponse::Created().finish()
//! }
//!
//! #[actix_rt::main]
//! async fn main() -> Result<(), anyhow::Error> {
//!     HttpServer::new(move || App::new().service(resource("/upload").route(post().to(upload))))
//!         .bind("127.0.0.1:8080")?
//!         .run();
//!         // .await?;
//!
//!     Ok(())
//! }
//!```

mod error;
mod extractor;
mod types;
mod upload;

pub use self::{
    error::{Error, ErrorKind},
    extractor::{FormData, Multipart},
    types::{Field, FileMeta, Form, Value},
    upload::handle_multipart,
};
