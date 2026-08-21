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

use actix_web::{
    error::{PayloadError, ResponseError},
    http::StatusCode,
    HttpResponse,
};

pub(crate) trait ResultExt<T, E> {
    fn or_raise(self, kind: ErrorKind) -> Result<T, Error>
    where
        Self: Sized,
        E: std::error::Error + Send + Sync + 'static;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn or_raise(self, kind: ErrorKind) -> Result<T, Error>
    where
        Self: Sized,
        E: std::error::Error + Send + Sync + 'static,
    {
        self.map_err(|error| Error::new_with(kind, error))
    }
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl Error {
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Self { kind, source: None }
    }

    pub(crate) fn new_with<E>(kind: ErrorKind, error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self {
            kind,
            source: Some(Box::new(error)),
        }
    }

    pub(crate) fn new_with_unsend<E>(kind: ErrorKind, error: E) -> Self
    where
        E: std::error::Error + 'static,
    {
        Self {
            kind,
            source: Some(Box::new(StringError::new(&error))),
        }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    Multipart,
    ParseField,
    ParseInt,
    ParseFloat,
    ContentDisposition,
    Field,
    FieldCount,
    FieldSize,
    FieldType,
    Filename,
    FileCount,
    FileSize,
    Panicked,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ErrorKind::Multipart => f.write_str("Failed to read multipart body"),
            ErrorKind::ParseField => f.write_str("Failed to parse multipart field"),
            ErrorKind::ParseInt => f.write_str("Failed to parse int"),
            ErrorKind::ParseFloat => f.write_str("Failed to parse float"),
            ErrorKind::ContentDisposition => f.write_str("Failed to parse Content-Disposition"),
            ErrorKind::Field => f.write_str("Failed to parse field name"),
            ErrorKind::FieldCount => f.write_str("Too many fields in request"),
            ErrorKind::FieldSize => f.write_str("Field too large"),
            ErrorKind::FieldType => f.write_str("Found field with unexpected name or type"),
            ErrorKind::Filename => f.write_str("Filename is missing"),
            ErrorKind::FileCount => f.write_str("Too many files in request"),
            ErrorKind::FileSize => f.write_str("File too large"),
            ErrorKind::Panicked => f.write_str("Task panicked"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as _)
    }
}

#[derive(Debug)]
pub enum MultipartError {
    NoContentDisposition,
    NoContentType,
    ParseContentType,
    Boundary,
    Nested,
    Incomplete,
    NotConsumed,
    DuplicateField(String),
    MissingField(String),
    UnsupportedField(String),
}

impl std::fmt::Display for MultipartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoContentDisposition => f.write_str("No Content-Disposition `form-data` header"),
            Self::NoContentType => f.write_str("No Content-Type `form-data` header"),
            Self::ParseContentType => {
                f.write_str("Failed to parse Content-Type `form-data` header")
            }
            Self::Boundary => f.write_str("Multipart boundary is missing"),
            Self::Nested => f.write_str("Nested multipart is not supported"),
            Self::Incomplete => f.write_str("Multipart stream is incomplete"),
            Self::NotConsumed => f.write_str("Multipart stream was not consumed"),
            Self::DuplicateField(field) => write!(f, "Duplicate field with name `{field}` found"),
            Self::MissingField(field) => write!(f, "Field with name `{field}` is required"),
            Self::UnsupportedField(field) => {
                write!(f, "Found unsupported field with name `{field}`")
            }
        }
    }
}

impl std::error::Error for MultipartError {}

#[derive(Debug)]
pub(crate) struct StringError {
    display: String,
    source: Option<Box<StringError>>,
}

impl StringError {
    fn new(error: &(dyn std::error::Error + 'static)) -> Self {
        let display = format!("{error}");

        let source = error.source().map(StringError::new).map(Box::new);

        Self { display, source }
    }
}

impl std::fmt::Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display)
    }
}

impl std::error::Error for StringError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e as _)
    }
}

#[derive(Debug)]
pub struct FieldError {
    name: String,
    source: StringError,
}

impl FieldError {
    fn new(name: String, source: actix_web::error::Error) -> Self {
        Self {
            name,
            source: StringError::new(&source),
        }
    }
}

impl std::fmt::Display for FieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed processing field with name `{}`", self.name)
    }
}

impl std::error::Error for FieldError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

impl From<actix_multipart::MultipartError> for Error {
    fn from(value: actix_multipart::MultipartError) -> Self {
        match value {
            actix_multipart::MultipartError::ContentDispositionMissing => {
                Error::new_with(ErrorKind::Multipart, MultipartError::NoContentDisposition)
            }
            actix_multipart::MultipartError::ContentTypeMissing => {
                Error::new_with(ErrorKind::Multipart, MultipartError::NoContentType)
            }
            actix_multipart::MultipartError::ContentTypeParse => {
                Error::new_with(ErrorKind::Multipart, MultipartError::ParseContentType)
            }
            actix_multipart::MultipartError::BoundaryMissing => {
                Error::new_with(ErrorKind::Multipart, MultipartError::Boundary)
            }
            actix_multipart::MultipartError::Nested => {
                Error::new_with(ErrorKind::Multipart, MultipartError::Nested)
            }
            actix_multipart::MultipartError::Incomplete => {
                Error::new_with(ErrorKind::Multipart, MultipartError::Incomplete)
            }
            actix_multipart::MultipartError::Parse(e) => Error::new_with(ErrorKind::Multipart, e),
            actix_multipart::MultipartError::Payload(e) => Error::new_with(ErrorKind::Multipart, e),
            actix_multipart::MultipartError::NotConsumed => {
                Error::new_with(ErrorKind::Multipart, MultipartError::NotConsumed)
            }
            actix_multipart::MultipartError::Field { name, source } => {
                Error::new_with(ErrorKind::Multipart, FieldError::new(name, source))
            }
            actix_multipart::MultipartError::DuplicateField(s) => {
                Error::new_with(ErrorKind::Multipart, MultipartError::DuplicateField(s))
            }
            actix_multipart::MultipartError::MissingField(s) => {
                Error::new_with(ErrorKind::Multipart, MultipartError::MissingField(s))
            }
            actix_multipart::MultipartError::UnknownField(s) => {
                Error::new_with(ErrorKind::Multipart, MultipartError::UnsupportedField(s))
            }
            e => Error::new_with_unsend(ErrorKind::Multipart, e),
        }
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        if let Some(source) = &self.source {
            if let Some(payload) = source.downcast_ref::<PayloadError>() {
                return payload.status_code();
            }
        }

        StatusCode::BAD_REQUEST
    }

    fn error_response(&self) -> HttpResponse {
        if let Some(source) = &self.source {
            if source.is::<tokio::task::JoinError>() {
                return HttpResponse::InternalServerError().finish();
            }

            if let Some(payload) = source.downcast_ref::<PayloadError>() {
                return payload.error_response();
            }
        }

        HttpResponse::BadRequest().finish()
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

    #[test]
    fn assert_sync() {
        fn is_sync<E: Sync>() {}
        is_sync::<Error>()
    }

    #[test]
    fn assert_error() {
        fn is_error<E: std::error::Error>() {}
        is_error::<Error>()
    }
}
