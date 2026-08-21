use sha3::TurboShake256;
use sha3::digest::{ExtendableOutput, Update, XofReader};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::Result;
use crate::db::crypto::Salt;

const KEY_LEN: usize = 32;

/// Represents a symmetric encryption key.
#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop)]
pub struct SymmetricKey([u8; KEY_LEN]);

impl SymmetricKey {
    /// Represents the byte length.
    pub const LEN: usize = KEY_LEN;
    pub const HASH_CONTEXT: u8 = 2;

    /// Creates a new [SymmetricKey].
    pub const fn new() -> Self {
        Self([0u8; Self::LEN])
    }

    /// Creates a new random [SymmetricKey].
    pub fn create() -> Self {
        let mut key = SymmetricKey::new();
        rand::fill(key.as_mut());
        key
    }

    /// Derives the key using a password-based hashing algorithm (Argon2id).
    pub fn derive(val: &[u8], salt: Salt) -> Result<Self> {
        let mut key = Self::new();

        let mut out = [0u8; 64];
        let argon2 = argon2::Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::default(),
            argon2::Params::default(),
        );
        argon2.hash_password_into(val, salt.as_ref(), &mut out)?;

        let mut hasher = Self::hasher();
        hasher.update(val.as_ref());
        hasher.update(&out);

        let mut reader = hasher.finalize_xof();
        reader.read(key.as_mut());

        Ok(key)
    }

    /// Gets the hash function instance used to derive a [SymmetricKey].
    #[inline]
    pub fn hasher() -> TurboShake256 {
        TurboShake256::from_core(sha3::TurboShake256Core::new(Self::HASH_CONTEXT))
    }

    /// Hashes the key with a default salt.
    pub fn hash(val: &[u8]) -> Result<Self> {
        Self::hash_with_salt(val, Salt::new())
    }

    /// Hashes the key with a salt.
    pub fn hash_with_salt(val: &[u8], salt: Salt) -> Result<Self> {
        let mut key = Self::new();

        let mut hasher = Self::hasher();
        hasher.update(val.as_ref());
        hasher.update(salt.as_ref());

        let mut reader = hasher.finalize_xof();
        reader.read(key.as_mut());

        Ok(key)
    }

    /// Converts a slice into a [SymmetricKey].
    ///
    /// Does not hash the input, converts directly into a [SymmetricKey].
    pub fn from_slice(val: &[u8]) -> Self {
        let mut salt = SymmetricKey::new();
        let len = std::cmp::min(Self::LEN, val.len());
        salt.0[..len].copy_from_slice(&val[..len]);
        salt
    }

    /// Gets a reference to the inner slice.
    pub const fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Gets the [SymmetricKey] length.
    pub const fn len(&self) -> usize {
        Self::LEN
    }

    /// Gets whether the [SymmetricKey] is all zeroes.
    pub fn is_empty(&self) -> bool {
        self.0 == [0u8; Self::LEN]
    }
}

impl Default for SymmetricKey {
    fn default() -> Self {
        Self::new()
    }
}

impl AsRef<[u8]> for SymmetricKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsMut<[u8]> for SymmetricKey {
    fn as_mut(&mut self) -> &mut [u8] {
        self.0.as_mut()
    }
}

impl From<&[u8]> for SymmetricKey {
    fn from(val: &[u8]) -> Self {
        Self::from_slice(val)
    }
}

impl<const N: usize> From<&[u8; N]> for SymmetricKey {
    fn from(val: &[u8; N]) -> Self {
        val.as_ref().into()
    }
}

impl<const N: usize> From<[u8; N]> for SymmetricKey {
    fn from(val: [u8; N]) -> Self {
        val.as_ref().into()
    }
}

impl From<Vec<u8>> for SymmetricKey {
    fn from(val: Vec<u8>) -> Self {
        val.as_slice().into()
    }
}
