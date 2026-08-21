use std::ffi::{CStr, NulError};
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The Adamo API rejected the credentials (bad or inactive API key).
    Auth(String),
    /// The Adamo API or a relay could not be reached.
    Network(String),
    /// Invalid argument: bad option value, closed handle, or bad config.
    Invalid(String),
    /// Video pipeline failure: encoder/decoder unavailable, capture or
    /// shared-memory source failed.
    Video(String),
    /// An uncategorized error reported by the underlying Adamo library.
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
            Error::Auth(m)
            | Error::Network(m)
            | Error::Invalid(m)
            | Error::Video(m)
            | Error::Ffi(m) => write!(f, "adamo: {m}"),
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

/// Pull the last error out of thread-local storage on the C side and map
/// its category code onto a typed variant. Always returns a populated
/// error — if libadamo didn't leave a message we synthesize a placeholder
/// so callers don't get an empty error.
pub(crate) fn last_ffi_error() -> Error {
    // SAFETY: adamo_last_error either returns NULL or a pointer to a
    // static/thread-local NUL-terminated C string (and adamo_last_error_code
    // is a plain thread-local read). We copy the message into an owned Rust
    // string before returning.
    let (msg, code) = unsafe {
        let ptr = adamo_sys::adamo_last_error();
        if ptr.is_null() {
            return Error::Ffi("(no error message)".into());
        }
        match CStr::from_ptr(ptr).to_str() {
            Ok(s) => (s.to_owned(), adamo_sys::adamo_last_error_code()),
            Err(_) => return Error::InvalidUtf8,
        }
    };
    match code {
        1 => Error::Auth(msg),
        2 => Error::Network(msg),
        3 => Error::Timeout,
        4 => Error::Invalid(msg),
        5 => Error::Video(msg),
        _ => Error::Ffi(msg),
    }
}
