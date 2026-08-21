//! Error module for the [adic-shape](crate) crate

use std::{error, fmt, num::TryFromIntError};
use adic::AdicError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Error from `adic-shape` operations
pub enum AdicShapeError {
    /// Error from the `adic` crate
    AdicError(AdicError),
    /// Error when dealing with `petgraph`
    PetGraph,
    /// Error for configuration problems
    ImproperConfig(String),
    /// Error from infinite digits where finite are expected
    InfiniteDigits,
    /// Error applying a style to an element
    StyleError(String),
    /// Error propagated from [`TryFromIntError`](std::num::TryFromIntError)
    TryFromIntError,
}


impl fmt::Display for AdicShapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl error::Error for AdicShapeError { }


impl From<AdicError> for AdicShapeError {
    fn from(e: AdicError) -> Self {
        AdicShapeError::AdicError(e)
    }
}

impl From<TryFromIntError> for AdicShapeError {
    fn from(_: TryFromIntError) -> Self {
        AdicShapeError::TryFromIntError
    }
}
