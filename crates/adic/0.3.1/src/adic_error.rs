//! Error module for the [adic](crate) crate

use std::{error, fmt, num::TryFromIntError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Error from [adic](crate) operations
pub enum AdicError {
    /// Error that results when an Adic conversion fails
    BadConversion,
    /// Divide-by-zero error
    DivideByZero,
    /// Error from an ill-defined operation, e.g. 0-th root of a number
    IllDefined(String),
    /// Error that results when the requested precision cannot be acheived
    InappropriatePrecision(String),
    /// Error that involves mixing adic's with different primes
    MixedCharacteristic,
    /// Error that results when an operation is requested that is not yet implemented
    NotImplemented(String),
    /// Error propagated from [`TryFromIntError`](std::num::TryFromIntError)
    TryFromIntError,
    /// Severe error
    Severe(String),
}


impl fmt::Display for AdicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl error::Error for AdicError { }


impl From<TryFromIntError> for AdicError {
    fn from(_: TryFromIntError) -> Self {
        AdicError::TryFromIntError
    }
}
