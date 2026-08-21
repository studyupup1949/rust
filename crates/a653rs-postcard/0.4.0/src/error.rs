//! Error Types

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
extern crate alloc;

use a653rs::prelude::*;

#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
pub enum QueuingRecvError {
    Apex(a653rs::prelude::Error),
    /// Postcard deserialization error
    ///
    /// Also returns the data which failed to deserialize
    Postcard(postcard::Error, Vec<u8>),
}

#[cfg(feature = "alloc")]
impl From<a653rs::prelude::Error> for QueuingRecvError {
    fn from(e: a653rs::prelude::Error) -> Self {
        QueuingRecvError::Apex(e)
    }
}

#[derive(Debug, Clone)]
pub enum QueuingRecvBufError<'a> {
    Apex(a653rs::prelude::Error),
    /// Postcard deserialization error
    ///
    /// Also returns the data which failed to deserialize
    Postcard(postcard::Error, &'a [u8]),
}

impl From<a653rs::prelude::Error> for QueuingRecvBufError<'_> {
    fn from(e: a653rs::prelude::Error) -> Self {
        QueuingRecvBufError::Apex(e)
    }
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
pub enum SamplingRecvError {
    Apex(a653rs::prelude::Error),
    /// Postcard deserialization error
    ///
    /// Also returns the data which failed to deserialize and its [`Validity`]
    Postcard(postcard::Error, Validity, Vec<u8>),
}

#[cfg(feature = "alloc")]
impl From<a653rs::prelude::Error> for SamplingRecvError {
    fn from(e: a653rs::prelude::Error) -> Self {
        SamplingRecvError::Apex(e)
    }
}

#[derive(Debug, Clone)]
pub enum SamplingRecvBufError<'a> {
    Apex(a653rs::prelude::Error),
    /// Postcard deserialization error
    ///
    /// Also returns the data which failed to deserialize and its [`Validity`]
    Postcard(postcard::Error, Validity, &'a [u8]),
}

impl From<a653rs::prelude::Error> for SamplingRecvBufError<'_> {
    fn from(e: a653rs::prelude::Error) -> Self {
        SamplingRecvBufError::Apex(e)
    }
}

#[derive(Debug)]
pub enum SendError {
    Apex(a653rs::prelude::Error),
    Postcard(postcard::Error),
}

impl From<a653rs::prelude::Error> for SendError {
    fn from(e: a653rs::prelude::Error) -> Self {
        SendError::Apex(e)
    }
}

impl From<postcard::Error> for SendError {
    fn from(e: postcard::Error) -> Self {
        SendError::Postcard(e)
    }
}
