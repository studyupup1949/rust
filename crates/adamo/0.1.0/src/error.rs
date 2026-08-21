use std::ffi::{CStr, NulError};
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// An error reported by the underlying Adamo library.
    Ffi(String),
    /// A Rust string contained an interior NUL, so it could not be passed
    /// across the C boundary.
    InteriorNul,
    /// The FFI returned a handle/payload that could not be decoded (e.g.
    /// non-UTF-8 key). The byte form is preserved.
    InvalidUtf8,
    /// A blocking receive timed out.
    Timeout,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Ffi(m) => write!(f, "adamo: {m}"),
            Error::InteriorNul => f.write_str("adamo: string contained an interior NUL byte"),
            Error::InvalidUtf8 => f.write_str("adamo: FFI returned non-UTF-8 data"),
            Error::Timeout => f.write_str("adamo: receive timed out"),
        }
    }
}

impl std::error::Error for Error {}

impl From<NulError> for Error {
    fn from(_: NulError) -> Self {
        Error::InteriorNul
    }
}

/// Pull the last-error string out of thread-local storage on the C side.
/// Always returns a populated `Error::Ffi` — if libadamo didn't leave a
/// message we synthesize a placeholder so callers don't get an empty
/// error.
pub(crate) fn last_ffi_error() -> Error {
    // SAFETY: adamo_last_error either returns NULL or a pointer to a
    // static/thread-local NUL-terminated C string. We copy it into an
    // owned Rust string before returning.
    unsafe {
        let ptr = adamo_sys::adamo_last_error();
        if ptr.is_null() {
            return Error::Ffi("(no error message)".into());
        }
        match CStr::from_ptr(ptr).to_str() {
            Ok(s) => Error::Ffi(s.to_owned()),
            Err(_) => Error::InvalidUtf8,
        }
    }
}
