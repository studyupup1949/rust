use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{Error, Result, impl_default};

/// Represents [Multikey]() public key prefix variants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum MultikeyPublicKeyPrefix {
    Ecdsa256 = 0x8024,
    Ecdsa384 = 0x8124,
    Ed25519 = 0xed01,
    Bls12 = 0xeb01,
    Sm2 = 0x8624,
}

impl MultikeyPublicKeyPrefix {
    pub const ECDSA_256: u16 = 0x8024;
    pub const ECDSA_384: u16 = 0x8124;
    pub const ED25519: u16 = 0xed01;
    pub const BLS12: u16 = 0xeb01;
    pub const SM2: u16 = 0x8624;

    /// Represents the byte length of the prefix.
    pub const LEN: usize = 2;

    /// Creates a new [MultikeyPublicKeyPrefix].
    pub const fn new() -> Self {
        Self::Ecdsa256
    }

    /// Converts the [MultikeyPublicKeyPrefix] into a [`u16`].
    pub const fn to_u16(self) -> u16 {
        self as u16
    }

    /// Converts the [MultikeyPublicKeyPrefix] into a byte array.
    pub const fn to_bytes(self) -> [u8; Self::LEN] {
        self.to_u16().to_be_bytes()
    }
}

impl_default!(MultikeyPublicKeyPrefix);

impl TryFrom<u16> for MultikeyPublicKeyPrefix {
    type Error = Error;

    fn try_from(val: u16) -> Result<Self> {
        match val {
            Self::ECDSA_256 => Ok(Self::Ecdsa256),
            Self::ECDSA_384 => Ok(Self::Ecdsa384),
            Self::ED25519 => Ok(Self::Ed25519),
            Self::BLS12 => Ok(Self::Bls12),
            Self::SM2 => Ok(Self::Sm2),
            _ => Err(Error::multikey(format!("invalid public key prefix: {val}"))),
        }
    }
}

impl TryFrom<&[u8]> for MultikeyPublicKeyPrefix {
    type Error = Error;

    fn try_from(val: &[u8]) -> Result<Self> {
        val.get(..2)
            .ok_or(Error::multikey(format!(
                "invalid prefix length: {}",
                val.len()
            )))
            .and_then(|b| <[u8; 2]>::try_from(b).map_err(|_| Error::multikey("invalid u16")))
            .map(u16::from_be_bytes)
            .and_then(Self::try_from)
    }
}

/// Represents a [Multikey](https://www.w3.org/TR/cid-1.0/#Multikey) public key.
#[derive(Clone, Debug, Eq, PartialEq, Zeroize, ZeroizeOnDrop)]
pub enum MultikeyPublicKey {
    Ecdsa256([u8; 33]),
    Ecdsa384([u8; 49]),
    Ed25519([u8; 32]),
    Bls12([u8; 96]),
    Sm2([u8; 33]),
}

impl MultikeyPublicKey {
    pub const ECDSA_256_LEN: usize = 33;
    pub const ECDSA_384_LEN: usize = 49;
    pub const ED25519_LEN: usize = 32;
    pub const BLS12_LEN: usize = 96;
    pub const SM2_LEN: usize = 33;

    /// Creates a new [MultikeyPublicKey].
    pub const fn new() -> Self {
        Self::Ecdsa256([0u8; Self::ECDSA_256_LEN])
    }

    /// Gets the prefix for an encoded [MultikeyPublicKey].
    #[inline]
    pub const fn prefix(&self) -> MultikeyPublicKeyPrefix {
        match self {
            Self::Ecdsa256(_) => MultikeyPublicKeyPrefix::Ecdsa256,
            Self::Ecdsa384(_) => MultikeyPublicKeyPrefix::Ecdsa384,
            Self::Ed25519(_) => MultikeyPublicKeyPrefix::Ed25519,
            Self::Bls12(_) => MultikeyPublicKeyPrefix::Bls12,
            Self::Sm2(_) => MultikeyPublicKeyPrefix::Sm2,
        }
    }

    /// Gets the [MultikeyPublicKey] key bytes.
    pub const fn key_bytes(&self) -> &[u8] {
        match self {
            Self::Ecdsa256(key) => key,
            Self::Ecdsa384(key) => key,
            Self::Ed25519(key) => key,
            Self::Bls12(key) => key,
            Self::Sm2(key) => key,
        }
    }

    /// Gets an iterator over the [MultikeyPublicKey] bytes.
    pub fn key_iter(&self) -> impl Iterator<Item = u8> {
        self.key_bytes().iter().copied()
    }
}

impl_default!(MultikeyPublicKey);

impl From<&MultikeyPublicKey> for Vec<u8> {
    fn from(val: &MultikeyPublicKey) -> Self {
        val.prefix()
            .to_bytes()
            .into_iter()
            .chain(val.key_iter())
            .collect()
    }
}

impl TryFrom<&[u8]> for MultikeyPublicKey {
    type Error = Error;

    fn try_from(val: &[u8]) -> Result<Self> {
        let prefix = MultikeyPublicKeyPrefix::try_from(val)?;

        match prefix {
            MultikeyPublicKeyPrefix::Ecdsa256 => val
                .get(2..2 + Self::ECDSA_256_LEN)
                .ok_or(Error::multikey(format!(
                    "invalid ecdsa-256 public key length: {}",
                    val.len()
                )))
                .and_then(|b| {
                    <[u8; Self::ECDSA_256_LEN]>::try_from(b)
                        .map_err(|err| Error::multikey(format!("invalid ecdsa-256 length: {err}")))
                })
                .map(Self::Ecdsa256),
            MultikeyPublicKeyPrefix::Ecdsa384 => val
                .get(2..2 + Self::ECDSA_384_LEN)
                .ok_or(Error::multikey(format!(
                    "invalid ecdsa-384 public key length: {}",
                    val.len()
                )))
                .and_then(|b| {
                    <[u8; Self::ECDSA_384_LEN]>::try_from(b)
                        .map_err(|err| Error::multikey(format!("invalid ecdsa-384 length: {err}")))
                })
                .map(Self::Ecdsa384),
            MultikeyPublicKeyPrefix::Ed25519 => val
                .get(2..2 + Self::ED25519_LEN)
                .ok_or(Error::multikey(format!(
                    "invalid ed25519 public key length: {}",
                    val.len()
                )))
                .and_then(|b| {
                    <[u8; Self::ED25519_LEN]>::try_from(b)
                        .map_err(|err| Error::multikey(format!("invalid ed25519 length: {err}")))
                })
                .map(Self::Ed25519),
            MultikeyPublicKeyPrefix::Bls12 => val
                .get(2..2 + Self::BLS12_LEN)
                .ok_or(Error::multikey(format!(
                    "invalid bls12-381 public key length: {}",
                    val.len()
                )))
                .and_then(|b| {
                    <[u8; Self::BLS12_LEN]>::try_from(b)
                        .map_err(|err| Error::multikey(format!("invalid bls12-381 length: {err}")))
                })
                .map(Self::Bls12),
            MultikeyPublicKeyPrefix::Sm2 => val
                .get(2..2 + Self::SM2_LEN)
                .ok_or(Error::multikey(format!(
                    "invalid sm2 public key length: {}",
                    val.len()
                )))
                .and_then(|b| {
                    <[u8; Self::SM2_LEN]>::try_from(b)
                        .map_err(|err| Error::multikey(format!("invalid sm2 length: {err}")))
                })
                .map(Self::Sm2),
        }
    }
}

impl TryFrom<Vec<u8>> for MultikeyPublicKey {
    type Error = Error;

    fn try_from(val: Vec<u8>) -> Result<Self> {
        val.as_slice().try_into()
    }
}
