//! Error module for the [adic](crate) crate

use std::{convert::Infallible, error, fmt, num::TryFromIntError};
use num::bigint::TryFromBigIntError;


/// Result for [adic](crate) operations
pub type AdicResult<A> = Result<A, AdicError>;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Error from [adic](crate) operations
pub enum AdicError {
    /// Error when an adic integer is expected but there are fractional components
    AdicIntegerExpected,
    /// Error that results when an Adic conversion fails
    BadConversion,
    /// A digit does not respect the proper base
    BadDigit,
    /// Dividing by prime in an integer context
    DivideByPrime,
    /// Divide-by-zero error
    DivideByZero,
    /// Taking sum or product of infinite series
    InfiniteOperation,
    /// Error from an ill-defined operation, e.g. 0-th root of a number
    IllDefined(String),
    /// Error that results when the requested precision cannot be acheived
    InappropriatePrecision(String),
    /// Error that involves mixing adic's with different primes
    MixedCharacteristic,
    /// No prime has been set for an adic number when necessary
    NoPrimeSet,
    /// Error propagated from [`TryFromIntError`](std::num::TryFromIntError)
    TryFromIntError,
    /// Generic severe error
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

impl<BI> From<TryFromBigIntError<BI>> for AdicError {
    fn from(_: TryFromBigIntError<BI>) -> Self {
        AdicError::TryFromIntError
    }
}

impl From<Infallible> for AdicError {
    fn from(_: Infallible) -> Self {
        panic!("Infallible error; catastrophic")
    }
}
