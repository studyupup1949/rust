use crate::vector::Vector;
use core::{array::TryFromSliceError, fmt};

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum AcnError {
    InvalidBufferLength { actual: usize, expected: usize },
    InvalidVector(Vector),
    InvalidPreamble,
    InvalidPostamble,
    TryFromSliceError(TryFromSliceError),
}

impl From<TryFromSliceError> for AcnError {
    fn from(err: TryFromSliceError) -> Self {
        AcnError::TryFromSliceError(err)
    }
}

impl fmt::Display for AcnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidBufferLength { actual, expected } => {
                write!(f, "Invalid Buffer Length: {actual}, expected: {expected}")
            }
            Self::InvalidVector(vector) => write!(f, "Invalid Vector: {vector:?}"),
            Self::InvalidPreamble => write!(f, "Invalid Preamble"),
            Self::InvalidPostamble => write!(f, "Invalid Postamble"),
            Self::TryFromSliceError(err) => write!(f, "TryFromSliceError: {err}"),
        }
    }
}

impl core::error::Error for AcnError {}
