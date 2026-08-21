/*
 * This file is part of Actix Form Data.
 *
 * Copyright © 2020 Riley Trautman
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
//! use actix_form_data::{Error, Field, Form, Value};
//! use actix_web::{
//!     web::{post, resource},
//!     App, HttpResponse, HttpServer,
//! };
//! use futures_util::stream::StreamExt;
//!
//! async fn upload(uploaded_content: Value<()>) -> HttpResponse {
//!     println!("Uploaded Content: {:#?}", uploaded_content);
//!     HttpResponse::Created().finish()
//! }
//!
//! #[actix_rt::main]
//! async fn main() -> Result<(), anyhow::Error> {
//!     let form = Form::new()
//!         .field("Hey", Field::text())
//!         .field(
//!             "Hi",
//!             Field::map()
//!                 .field("One", Field::int())
//!                 .field("Two", Field::float())
//!                 .finalize(),
//!         )
//!         .field(
//!             "files",
//!             Field::array(Field::file(|_, _, mut stream| async move {
//!                 while let Some(_) = stream.next().await {
//!                     // do something
//!                 }
//!                 Ok(()) as Result<(), Error>
//!             })),
//!         );
//!
//!     println!("{:?}", form);
//!
//!     HttpServer::new(move || {
//!         App::new()
//!             .wrap(form.clone())
//!             .service(resource("/upload").route(post().to(upload)))
//!     })
//!     .bind("127.0.0.1:8082")?;
//!     // commented out to prevent infinite doctest
//!     // .run()
//!     // .await?;
//!
//!     Ok(())
//! }
//!```

mod error;
mod middleware;
mod types;
mod upload;

pub use self::{
    error::Error,
    types::{Field, FileMeta, Form, Value},
    upload::handle_multipart,
};
