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
    string::FromUtf8Error,
};

use actix_multipart::MultipartError;
use actix_web::{
    error::{PayloadError, ResponseError},
    http::StatusCode,
    HttpResponse,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error parsing payload")]
    Payload(#[from] PayloadError),
    #[error("Error in multipart creation")]
    Multipart(MultipartError),
    #[error("Failed to parse field")]
    ParseField(#[from] FromUtf8Error),
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
}

impl From<MultipartError> for Error {
    fn from(m: MultipartError) -> Self {
        Error::Multipart(m)
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
