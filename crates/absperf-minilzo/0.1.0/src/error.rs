use std::os::raw::c_int;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Error,
    OutOfMemory,
    NotCompressible,
    InputOverrun,
    OutputOverrun,
    LookbehindOverrun,
    EOFNotFound,
    InputNotConsumed,
    NotYetImplemented,
    InvalidArgument,
    InvalidAlignment,
    OutputNotConsumed,
    InternalError,
    Unknown,
}

impl From<c_int> for Error {
    fn from(value: c_int) -> Self {
        match value {
            -1 => Error::Error,
            -2 => Error::OutOfMemory,
            -3 => Error::NotCompressible,
            -4 => Error::InputOverrun,
            -5 => Error::OutputOverrun,
            -6 => Error::LookbehindOverrun,
            -7 => Error::EOFNotFound,
            -8 => Error::InputNotConsumed,
            -9 => Error::NotYetImplemented,
            -10 => Error::InvalidArgument,
            -11 => Error::InvalidAlignment,
            -12 => Error::OutputNotConsumed,
            -99 => Error::InternalError,
            _ => Error::Unknown,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn check(value: c_int) -> Result<()> {
    if value == 0 {
        Ok(())
    } else {
        Err(value.into())
    }
}
