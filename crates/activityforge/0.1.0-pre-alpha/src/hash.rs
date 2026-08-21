use activitystreams_vocabulary::impl_default;
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

mod sha1;
mod sha256;

pub use sha1::Sha1Hash;
pub use sha256::Sha256Hash;

/// Represents a commit hash.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Hash {
    Sha1(Sha1Hash),
    Sha256(Sha256Hash),
}

impl Hash {
    /// Creates a new [Hash](Self).
    pub const fn new() -> Self {
        Self::Sha1(Sha1Hash::new())
    }

    /// Creates a new [Hash](Self) [Sha1](Self::Sha1) variant.
    pub fn sha1<I: Into<Sha1Hash>>(val: I) -> Self {
        Self::Sha1(val.into())
    }

    /// Gets whether the [Hash](Self) is a [Sha1](Self::Sha1) variant.
    pub const fn is_sha1(&self) -> bool {
        matches!(self, Self::Sha1(_))
    }

    /// Gets the [Hash](Self) as a [Sha1](Self::Sha1) variant.
    pub fn as_sha1(&self) -> Result<&Sha1Hash> {
        match self {
            Self::Sha1(hash) => Ok(hash),
            _ => Err(Error::hash("invalid SHA-1 hash")),
        }
    }

    /// Converts the [Hash](Self) into a [Sha1](Self::Sha1) variant.
    pub fn to_sha1(self) -> Result<Sha1Hash> {
        match self {
            Self::Sha1(hash) => Ok(hash),
            _ => Err(Error::hash("invalid SHA-1 hash")),
        }
    }

    /// Creates a new [Hash](Self) [Sha256](Self::Sha256) variant.
    pub fn sha256<I: Into<Sha256Hash>>(val: I) -> Self {
        Self::Sha256(val.into())
    }

    /// Gets whether the [Hash](Self) is a [Sha256](Self::Sha256) variant.
    pub const fn is_sha256(&self) -> bool {
        matches!(self, Self::Sha256(_))
    }

    /// Gets the [Hash](Self) as a [Sha256](Self::Sha256) variant.
    pub fn as_sha256(&self) -> Result<&Sha256Hash> {
        match self {
            Self::Sha256(hash) => Ok(hash),
            _ => Err(Error::hash("invalid SHA-256 hash")),
        }
    }

    /// Converts the [Hash](Self) into a [Sha256](Self::Sha256) variant.
    pub fn to_sha256(self) -> Result<Sha256Hash> {
        match self {
            Self::Sha256(hash) => Ok(hash),
            _ => Err(Error::hash("invalid SHA-256 hash")),
        }
    }
}

impl_default!(Hash);

impl core::fmt::Display for Hash {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Sha1(hash) => write!(f, "{hash}"),
            Self::Sha256(hash) => write!(f, "{hash}"),
        }
    }
}

impl From<Sha1Hash> for Hash {
    fn from(val: Sha1Hash) -> Self {
        Self::sha1(val)
    }
}

impl From<Sha256Hash> for Hash {
    fn from(val: Sha256Hash) -> Self {
        Self::sha256(val)
    }
}
