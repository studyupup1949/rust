use std::{error::Error as StdError, fmt};

#[derive(Debug)]
/// The error type for the cache
pub enum Error {
    /// An error in the database
    Ext(sled_extensions::Error),

    /// An opeation was canceled
    Canceled,

    /// The cache's mailbox is full
    Full,

    /// The cache's mailbox is closed
    Closed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Ext(ref e) => write!(f, "Error in db, {}", e),
            Error::Canceled => write!(f, "Operation was canceled"),
            Error::Full => write!(f, "Actor mailbox is full"),
            Error::Closed => write!(f, "Actor mailbox is closed"),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Ext(_) => "Error in db",
            Error::Canceled => "Operation was canceled",
            Error::Full => "Actor mailbox is full",
            Error::Closed => "Actor mailbox is closed",
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            Error::Ext(ref e) => Some(e),
            Error::Canceled | Error::Full | Error::Closed => None,
        }
    }
}

impl From<sled_extensions::Error> for Error {
    fn from(e: sled_extensions::Error) -> Self {
        Error::Ext(e)
    }
}

impl<E> From<actix_threadpool::BlockingError<E>> for Error
where
    E: std::fmt::Debug,
    Error: From<E>,
{
    fn from(e: actix_threadpool::BlockingError<E>) -> Self {
        match e {
            actix_threadpool::BlockingError::Error(e) => e.into(),
            actix_threadpool::BlockingError::Canceled => Error::Canceled,
        }
    }
}

impl<T> From<actix::prelude::SendError<T>> for Error {
    fn from(e: actix::prelude::SendError<T>) -> Self {
        match e {
            actix::prelude::SendError::Full(_) => Error::Full,
            actix::prelude::SendError::Closed(_) => Error::Closed,
        }
    }
}
