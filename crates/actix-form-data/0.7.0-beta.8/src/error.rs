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

use std::{
    num::{ParseFloatError, ParseIntError},
    str::Utf8Error,
};

use actix_web::{
    error::{ParseError, PayloadError, ResponseError},
    http::StatusCode,
    HttpResponse,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error parsing payload")]
    Payload(#[from] PayloadError),
    #[error("Error in multipart creation")]
    Multipart(#[from] MultipartError),
    #[error("Failed to parse field")]
    ParseField(#[from] Utf8Error),
    #[error("Failed to parse int")]
    ParseInt(#[from] ParseIntError),
    #[error("Failed to parse float")]
    ParseFloat(#[from] ParseFloatError),
    #[error("Bad Content-Type")]
    ContentType,
    #[error("Bad Content-Disposition")]
    ContentDisposition,
    #[error("Failed to parse field name")]
    Field,
    #[error("Too many fields in request")]
    FieldCount,
    #[error("Field too large")]
    FieldSize,
    #[error("Found field with unexpected name or type")]
    FieldType,
    #[error("Failed to parse filename")]
    Filename,
    #[error("Too many files in request")]
    FileCount,
    #[error("File too large")]
    FileSize,
    #[error("Task panicked")]
    Panicked,
}

#[derive(Debug, thiserror::Error)]
pub enum MultipartError {
    #[error("No Content-Disposition `form-data` header")]
    NoContentDisposition,
    #[error("No Content-Type header found")]
    NoContentType,
    #[error("Cannot parse Content-Type header")]
    ParseContentType,
    #[error("Multipart boundary is not found")]
    Boundary,
    #[error("Nested multipart is not supported")]
    Nested,
    #[error("Multipart stream is incomplete")]
    Incomplete,
    #[error("Failed parsing")]
    Parse(#[source] ParseError),
    #[error("Multipart stream is not consumed")]
    NotConsumed,
    #[error("An error occured processing field `{field_name}`: `{zource}`")]
    Field { field_name: String, zource: String },
    #[error("Duplicate field found for: `{0}")]
    DuplicateField(String),
    #[error("Field with name `{0}` is required")]
    MissingField(String),
    #[error("Unsupported field `{0}`")]
    UnsupportedField(String),
    #[error("Unknown error occured: {0}")]
    Unknown(String),
}

impl From<actix_multipart::MultipartError> for Error {
    fn from(value: actix_multipart::MultipartError) -> Self {
        match value {
            actix_multipart::MultipartError::ContentDispositionMissing => {
                Error::Multipart(MultipartError::NoContentDisposition)
            }
            actix_multipart::MultipartError::ContentTypeMissing => {
                Error::Multipart(MultipartError::NoContentType)
            }
            actix_multipart::MultipartError::ContentTypeParse => {
                Error::Multipart(MultipartError::ParseContentType)
            }
            actix_multipart::MultipartError::BoundaryMissing => {
                Error::Multipart(MultipartError::Boundary)
            }
            actix_multipart::MultipartError::Nested => Error::Multipart(MultipartError::Nested),
            actix_multipart::MultipartError::Incomplete => {
                Error::Multipart(MultipartError::Incomplete)
            }
            actix_multipart::MultipartError::Parse(e) => Error::Multipart(MultipartError::Parse(e)),
            actix_multipart::MultipartError::Payload(e) => Error::Payload(e),
            actix_multipart::MultipartError::NotConsumed => {
                Error::Multipart(MultipartError::NotConsumed)
            }
            actix_multipart::MultipartError::Field { name, source } => {
                Error::Multipart(MultipartError::Field {
                    field_name: name,
                    zource: source.to_string(),
                })
            }
            actix_multipart::MultipartError::DuplicateField(s) => {
                Error::Multipart(MultipartError::DuplicateField(s))
            }
            actix_multipart::MultipartError::MissingField(s) => {
                Error::Multipart(MultipartError::MissingField(s))
            }
            actix_multipart::MultipartError::UnknownField(s) => {
                Error::Multipart(MultipartError::UnsupportedField(s))
            }
            e => Error::Multipart(MultipartError::Unknown(e.to_string())),
        }
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(_: tokio::task::JoinError) -> Self {
        Self::Panicked
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match *self {
            Error::Payload(ref e) => e.status_code(),
            _ => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match *self {
            Error::Payload(ref e) => e.error_response(),
            Error::Panicked => HttpResponse::InternalServerError().finish(),
            Error::Multipart(_)
            | Error::ParseField(_)
            | Error::ParseInt(_)
            | Error::ParseFloat(_) => HttpResponse::BadRequest().finish(),
            Error::ContentType
            | Error::ContentDisposition
            | Error::Field
            | Error::FieldCount
            | Error::FieldSize
            | Error::FieldType
            | Error::Filename
            | Error::FileCount
            | Error::FileSize => HttpResponse::BadRequest().finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn assert_send() {
        fn is_send<E: Send>() {}
        is_send::<Error>();
    }
}
