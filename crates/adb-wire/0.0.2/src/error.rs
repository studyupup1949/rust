use std::io;

/// Error type for ADB wire protocol operations.
#[derive(Debug)]
pub enum Error {
    /// TCP connection or I/O failure.
    Io(io::Error),
    /// ADB server returned FAIL with a message.
    Adb(String),
    /// Unexpected response from ADB server.
    Protocol(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "io: {e}"),
            Error::Adb(msg) => write!(f, "adb: {msg}"),
            Error::Protocol(msg) => write!(f, "protocol: {msg}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl Error {
    pub(crate) fn timed_out(msg: &str) -> Self {
        Error::Io(io::Error::new(io::ErrorKind::TimedOut, msg))
    }
}

pub type Result<T> = std::result::Result<T, Error>;
