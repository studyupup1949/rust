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
    dev::Body,
    error::{PayloadError, ResponseError},
    http::StatusCode,
    BaseHttpResponse,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    FileFn(#[from] actix_web::Error),
    #[error("Error parsing payload, {0}")]
    Payload(#[from] PayloadError),
    #[error("Error in multipart creation, {0}")]
    Multipart(MultipartError),
    #[error("Failed to parse field, {0}")]
    ParseField(#[from] FromUtf8Error),
    #[error("Failed to parse int, {0}")]
    ParseInt(#[from] ParseIntError),
    #[error("Failed to parse float, {0}")]
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
    #[error("Uploaded guard used without Multipart middleware")]
    MissingMiddleware,
    #[error("Impossible Error! Middleware exists, didn't fail, and didn't send value")]
    TxDropped,
}

impl From<MultipartError> for Error {
    fn from(m: MultipartError) -> Self {
        Error::Multipart(m)
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match *self {
            Error::FileFn(ref e) => ResponseError::status_code(e.as_response_error()),
            Error::Payload(ref e) => ResponseError::status_code(e),
            Error::MissingMiddleware | Error::TxDropped => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> BaseHttpResponse<Body> {
        match *self {
            Error::FileFn(ref e) => ResponseError::error_response(e.as_response_error()),
            Error::Payload(ref e) => ResponseError::error_response(e),
            Error::Multipart(_)
            | Error::ParseField(_)
            | Error::ParseInt(_)
            | Error::ParseFloat(_) => BaseHttpResponse::bad_request(),
            Error::ContentType
            | Error::ContentDisposition
            | Error::Field
            | Error::FieldCount
            | Error::FieldSize
            | Error::FieldType
            | Error::Filename
            | Error::FileCount
            | Error::FileSize => BaseHttpResponse::bad_request(),
            Error::MissingMiddleware | Error::TxDropped => {
                BaseHttpResponse::internal_server_error()
            }
        }
    }
}
