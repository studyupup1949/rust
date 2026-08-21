use activitystreams_vocabulary::{impl_default, impl_display};
use serde::{Deserialize, Serialize};

use crate::crypto::AlgorithmName;
use crate::{Error, Result};

/// Represents the SQL key type variants.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, sqlx::Type,
)]
#[sqlx(type_name = "key_type")]
pub enum KeyType {
    Ecdsa256,
    Ecdsa384,
    Ed25519,
    Bls12,
    Sm2,
    Rsa2048,
    Rsa3072,
    Rsa4096,
}

impl KeyType {
    /// String representation for the [Ecdsa256](Self::Ecdsa256) variant.
    pub const ECDSA256: &str = "ecdsa-p256-sha256";
    /// String representation for the [Ecdsa384](Self::Ecdsa384) variant.
    pub const ECDSA384: &str = "ecdsa-p384-sha384";
    /// String representation for the [Ed25519](Self::Ed25519) variant.
    pub const ED25519: &str = "ed25519";
    /// String representation for the [Bls12](Self::Bls12) variant.
    pub const BLS12: &str = "bls12";
    /// String representation for the [Sm2](Self::Sm2) variant.
    pub const SM2: &str = "sm2";
    /// String representation for the [Rsa2048](Self::Rsa2048) variant.
    pub const RSA2048: &str = "rsa-v1_5-sha256";
    /// String representation for the [Rsa3072](Self::Rsa3072) variant.
    pub const RSA3072: &str = "rsa-pss-sha384";
    /// String representation for the [Rsa4096](Self::Rsa4096) variant.
    pub const RSA4096: &str = "rsa-pss-sha512";

    /// Creates a new [KeyType].
    pub const fn new() -> Self {
        Self::Ed25519
    }

    /// Gets the string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Ecdsa256 => Self::ECDSA256,
            Self::Ecdsa384 => Self::ECDSA384,
            Self::Ed25519 => Self::ED25519,
            Self::Bls12 => Self::BLS12,
            Self::Sm2 => Self::SM2,
            Self::Rsa2048 => Self::RSA2048,
            Self::Rsa3072 => Self::RSA3072,
            Self::Rsa4096 => Self::RSA4096,
        }
    }
}

impl TryFrom<AlgorithmName> for KeyType {
    type Error = Error;

    fn try_from(val: AlgorithmName) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&AlgorithmName> for KeyType {
    type Error = Error;

    fn try_from(val: &AlgorithmName) -> Result<Self> {
        match val {
            AlgorithmName::Ed25519 => Ok(Self::Ed25519),
            AlgorithmName::EcdsaP256Sha256 => Ok(Self::Ecdsa256),
            AlgorithmName::EcdsaP384Sha384 => Ok(Self::Ecdsa384),
            AlgorithmName::RsaV1_5Sha256 => Ok(Self::Rsa2048),
            _ => Err(Error::crypto("invalid key type: {val}")),
        }
    }
}

impl TryFrom<KeyType> for AlgorithmName {
    type Error = Error;

    fn try_from(val: KeyType) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&KeyType> for AlgorithmName {
    type Error = Error;

    fn try_from(val: &KeyType) -> Result<Self> {
        match val {
            KeyType::Ed25519 => Ok(Self::Ed25519),
            KeyType::Ecdsa256 => Ok(Self::EcdsaP256Sha256),
            KeyType::Ecdsa384 => Ok(Self::EcdsaP384Sha384),
            KeyType::Rsa2048 => Ok(Self::RsaV1_5Sha256),
            _ => Err(Error::crypto(format!("invalid key type: {val}"))),
        }
    }
}

impl TryFrom<KeyType> for jwt::Algorithm {
    type Error = Error;

    fn try_from(val: KeyType) -> Result<Self> {
        match val {
            KeyType::Ed25519 => Ok(jwt::Algorithm::EdDSA),
            KeyType::Ecdsa256 => Ok(jwt::Algorithm::ES256),
            KeyType::Ecdsa384 => Ok(jwt::Algorithm::ES384),
            KeyType::Rsa2048 => Ok(jwt::Algorithm::RS256),
            _ => Err(Error::crypto(format!("invalid JWT algorithm: {val}"))),
        }
    }
}

impl_default!(KeyType);
impl_display!(KeyType, str);
