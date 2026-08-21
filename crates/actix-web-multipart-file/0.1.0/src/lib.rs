//! ```rust
//! # extern crate actix_web;
//! # extern crate actix_web_multipart_file;
//! # extern crate futures;
//! # use actix_web::{HttpRequest, HttpMessage, FutureResponse, HttpResponse};
//! # use actix_web_multipart_file::save_files;
//! # use futures::Stream;
//!
//! fn handle_multipart(req: HttpRequest) -> FutureResponse<HttpResponse> {
//!     save_files(req.multipart())
//!         .and_then(|field| {
//!             // do something with field
//! #           ::futures::future::ok(())
//!         })
//! # ; unimplemented!()
//!     // ...
//! }
//!
//! ```

extern crate actix_web;
extern crate bytes;
extern crate futures;
#[macro_use]
extern crate log;
extern crate tempfile;
#[macro_use]
extern crate failure;
extern crate mime;

use actix_web::error::PayloadError;
use actix_web::http::header::ContentDisposition;
use actix_web::http::HeaderMap;
use actix_web::multipart::Multipart;
use actix_web::multipart::MultipartItem;
use actix_web::{HttpResponse, ResponseError};
use bytes::Bytes;
use futures::future::Either;
use futures::prelude::*;
use futures::stream;
use mime::Mime;
use std::fs::File;
use std::io::Write;
use tempfile::tempfile;

#[derive(Debug)]
pub struct Field {
    pub headers: HeaderMap,
    pub content_type: Mime,
    pub content_disposition: ContentDisposition,
    pub form_data: FormData,
}

impl Field {
    /// for compatibility with multipart's Field
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// for compatibility with multipart's Field
    pub fn content_type(&self) -> &Mime {
        &self.content_type
    }

    /// for compatibility with multipart's Field.
    /// always return Some(data)
    pub fn content_disposition(&self) -> Option<ContentDisposition> {
        Some(self.content_disposition.clone())
    }
}

#[derive(Debug)]
pub enum FormData {
    /// form data that don't have filename
    Data {
        /// name field
        name: String,
        /// body
        value: Bytes,
    },
    /// form data thathave filename
    File {
        /// name field
        name: String,
        /// filename field
        filename: String,
        /// a temporary file that contains the body
        file: File,
    },
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Multipart error: {}", _0)]
    Multipart(#[cause] ::actix_web::error::MultipartError),
    #[fail(display = "Given data is not a form data")]
    NotFormData,
    #[fail(display = " File handling error: {}", _0)]
    Io(#[cause] ::std::io::Error),
}

impl From<::actix_web::error::MultipartError> for Error {
    fn from(e: ::actix_web::error::MultipartError) -> Self {
        Error::Multipart(e)
    }
}

impl From<::std::io::Error> for Error {
    fn from(e: ::std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        match self {
            Error::Multipart(_) => HttpResponse::BadRequest().body("multipart error"),
            Error::NotFormData => HttpResponse::BadRequest().body("not a form data"),
            Error::Io(_) => HttpResponse::InternalServerError().body("file error"),
        }
    }
}

pub type BoxStream<'a, T, E> = Box<Stream<Item = T, Error = E> + 'a>;

fn save_into_tempfile<S, I, E>(input: S) -> impl Future<Item = File, Error = Error>
where
    S: Stream<Item = I, Error = E>,
    I: AsRef<[u8]>,
    Error: From<E>,
{
    tempfile()
        .into_future()
        .from_err()
        .and_then(|tempfile| {
            let file = tempfile.try_clone()?;
            Ok((tempfile, file))
        })
        .and_then(|(mut tempfile, mut file)| {
            input
                .from_err()
                .and_then(move |bytes| {
                    // FIXME: make it async
                    tempfile.write_all(bytes.as_ref()).map_err(|err| err.into())
                })
                .collect()
                .and_then(move |_| {
                    use std::io::{Seek, SeekFrom};
                    // FIXME: make it async
                    file.seek(SeekFrom::Start(0))?;
                    Ok(file)
                })
        })
}

/// save form data with filename into tempfiles
pub fn save_files<S>(multipart: Multipart<S>) -> BoxStream<'static, Field, Error>
where
    S: Stream<Item = Bytes, Error = PayloadError> + 'static,
{
    let st = multipart
        .map_err(Error::from)
        .map(|item| -> BoxStream<Field, Error> {
            match item {
                MultipartItem::Nested(m) => save_files(m),
                MultipartItem::Field(field) => {
                    debug!("multipart field: {:?}", field);
                    // form data must have content disposition
                    let content_disposition = match field.content_disposition() {
                        Some(d) => d,
                        None => return Box::new(stream::once(Err(Error::NotFormData))),
                    };

                    if !content_disposition.is_form_data() {
                        return Box::new(stream::once(Err(Error::NotFormData)));
                    }

                    // currently fields that don't have names are ignored
                    let name = match content_disposition.get_name() {
                        Some(n) => n.to_string(),
                        None => return Box::new(stream::empty()),
                    };

                    let content_type = field.content_type().clone();
                    let headers = field.headers().clone();

                    let form_data = match content_disposition.get_filename() {
                        None => {
                            let fut = field
                                .from_err()
                                .concat2()
                                .map(|value| FormData::Data { name, value });
                            Either::A(fut)
                        }

                        Some(filename) => {
                            let filename = filename.to_string();
                            let fut = save_into_tempfile(field).map(move |file| FormData::File {
                                name,
                                filename,
                                file,
                            });
                            Either::B(fut)
                        }
                    };
                    let fut = form_data.map(move |form_data| Field {
                        content_type,
                        headers,
                        content_disposition,
                        form_data,
                    });
                    Box::new(fut.into_stream())
                }
            }
        })
        .flatten();
    Box::new(st)
}
