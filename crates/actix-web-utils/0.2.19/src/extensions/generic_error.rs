use std::fmt::Display;

/// This is a wrapper for all errors. The error must implement display at least.
/// Usage:
/// ```
/// use actix_web_utils::enums::error::Error;
/// use actix_web_utils::extensions::generic_error::GenericError;
/// let generic_error = GenericError::wrap(Error::Unspecified);
/// //Use it as you please
/// ```
pub struct GenericError<E: Display> {
    pub error: E
}
impl<E: Display> GenericError<E> {
    pub fn wrap(e: E) -> GenericError<E>{
        GenericError { error: e }
    }
}