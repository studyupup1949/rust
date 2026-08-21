//! Error module for the `adic-shape` crate

use std::{error, fmt, num::TryFromIntError};
use adic::error::AdicError;


/// Result for `adic-shape` operations
pub type AdicShapeResult<A> = Result<A, AdicShapeError>;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Error from `adic-shape` operations
pub enum AdicShapeError {
    /// Error from the `adic` crate
    AdicError(AdicError),
    /// Configuration problems
    ImproperConfig(String),
    /// Infinite digits where finite are expected
    InfiniteDigits,
    /// Not enough data to manage shape
    InsufficientData(String),
    /// Error during math
    Math(String),
    /// Error from parsing incorrectly
    Parse(String),
    /// Error when dealing with `petgraph`
    PetGraph,
    /// Error applying a style to an element
    StyleError(String),
    /// Shape is too large to handle
    TooLarge(String),
    /// Error propagated from [`std::num::TryFromIntError`]
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
