//! Error module for the [adic](crate) crate

use std::{error, fmt};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Error from [adic](crate) operations
pub enum AdicError {
    /// Error that results when an Adic conversion fails
    BadConversion,
    /// Error from an ill-defined operation, e.g. 0-th root of a number
    IllDefined(String),
    /// Error that results when the requested precision cannot be acheived
    InappropriatePrecision(String),
    /// Error that involves mixing adic's with different primes
    MixedCharacteristic,
    /// Error that results when an operation is requested that is not yet implemented
    NotImplemented(String),
}


impl fmt::Display for AdicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl error::Error for AdicError { }
