use std::fmt::Display;

use log::debug;
use serde::Serialize;

use crate::{dtos::message::MessageResource, enums::error::Error, extensions::typed_response::TypedHttpResponse};

/// This trait aims to aid macros defined in this crate so that the macro can take any shape of error and
/// do the same thing for all.
pub trait ReturnableErrorShape {
    fn convert_to_returnable<T: Serialize> (&self, status_code: u16) -> TypedHttpResponse<T>;
}
impl ReturnableErrorShape for MessageResource {
    fn convert_to_returnable<T: Serialize>(&self, status_code: u16) -> TypedHttpResponse<T> {
        TypedHttpResponse::return_standard_error(status_code, self.clone())
    }
}
impl ReturnableErrorShape for Error {
    fn convert_to_returnable<T: Serialize>(&self, status_code: u16) -> TypedHttpResponse<T> {
        debug!("Converted error to returnable. Error: {}", self.to_string());
        match self {
            Error::Unspecified => TypedHttpResponse::return_standard_error(status_code, MessageResource::new_empty()),
            Error::NetworkError(message) => TypedHttpResponse::return_standard_error(status_code, message.clone()),
            Error::UnexpectedStatusCode(_, actual, message) => TypedHttpResponse::return_standard_error(*actual, message.clone()),
            Error::ClientError(message) => TypedHttpResponse::return_standard_error(status_code, message.clone()),
            Error::SerdeError(message, _) => TypedHttpResponse::return_standard_error(status_code, message.clone()),
            Error::DatabaseError(message, _) => TypedHttpResponse::return_standard_error(status_code, message.clone()),
            Error::ComputeError(message) => TypedHttpResponse::return_standard_error(status_code, message.clone()),
        }
    }
}
impl ReturnableErrorShape for Vec<MessageResource> {
    fn convert_to_returnable<T: Serialize>(&self, status_code: u16) -> TypedHttpResponse<T> {
        TypedHttpResponse::return_standard_error_list(status_code, self.to_vec())
    }
}
impl ReturnableErrorShape for dyn Display {
    fn convert_to_returnable<T: Serialize>(&self, status_code: u16) -> TypedHttpResponse<T> {
        TypedHttpResponse::return_standard_error(status_code, MessageResource::new_from_err(self))
    }
}
