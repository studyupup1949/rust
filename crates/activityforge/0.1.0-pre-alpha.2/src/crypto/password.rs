use argon2::{PasswordHasher, PasswordVerifier};
use sha3::TurboShake256;
use sha3::digest::{ExtendableOutput, Update, XofReader};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::crypto::Salt;
use crate::{Error, Result};

const PASS_LEN: usize = 32;

/// Represents a hashed password used to authenticate clients.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Zeroize, ZeroizeOnDrop, sqlx::Type)]
#[sqlx(transparent)]
pub struct Password(String);

impl Password {
    /// Represents the byte length.
    pub const LEN: usize = PASS_LEN;
    pub const HASH_CONTEXT: u8 = 7;

    /// Creates a new empty [Password].
    pub const fn new() -> Self {
        Self(String::new())
    }

    /// Derives the password using a password-based hashing algorithm (Argon2id).
    pub fn derive(id: &str, val: &[u8]) -> Result<Self> {
        let salt = Salt::random();

        Self::derive_with_salt(id, val, salt)
    }

    /// Derives the password using a password-based hashing algorithm (Argon2id).
    ///
    /// Uses the provided [Salt] for KDF input.
    pub fn derive_with_salt(id: &str, val: &[u8], salt: Salt) -> Result<Self> {
        let pre_hash = Self::pre_hash(id, val);

        let salt_b64 = salt.as_argon2_b64();
        let salt_str = argon2::password_hash::Salt::from_b64(&salt_b64)
            .map_err(|err| Error::crypto(format!("password: error decoding salt: {err}")))?;

        Self::argon2_hasher()
            .hash_password(&pre_hash, salt_str)
            .map(|s| Self(s.to_string()))
            .map_err(|err| Error::crypto(format!("password: error hashing password: {err}")))
    }

    /// Verifies the `password` against the stored password hash.
    pub fn verify(&self, id: &str, password: &[u8]) -> Result<()> {
        let parsed_hash = argon2::PasswordHash::new(&self.0).map_err(|err| {
            Error::crypto(format!("password: error parsing password hash: {err}"))
        })?;

        let pre_hash = Self::pre_hash(id, password);

        Self::argon2_hasher()
            .verify_password(&pre_hash, &parsed_hash)
            .map_err(|err| Error::crypto(format!("password: error verifying hash: {err}")))
    }

    /// Gets the [Argon2](argon2) instance used to calculate the final password hash.
    #[inline]
    pub fn argon2_hasher() -> argon2::Argon2<'static> {
        argon2::Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::default(),
            argon2::Params::default(),
        )
    }

    /// Gets the hash function instance used to derive a [Password].
    #[inline]
    pub fn pre_hasher() -> TurboShake256 {
        TurboShake256::from_core(sha3::TurboShake256Core::new(Self::HASH_CONTEXT))
    }

    /// Calculates the pre-hash used as input to [Argon2](argon2).
    pub(crate) fn pre_hash(id: &str, val: &[u8]) -> [u8; Self::LEN] {
        let mut out = [0u8; Self::LEN];
        let mut hasher = Self::pre_hasher();
        hasher.update(id.as_bytes());
        hasher.update(val);

        let mut reader = hasher.finalize_xof();
        reader.read(out.as_mut());

        out
    }

    /// Converts a slice into a [Password].
    ///
    /// Does not hash the input, converts directly into a [Password].
    pub fn from_slice(val: &[u8]) -> Result<Self> {
        String::from_utf8(val.to_vec())
            .map(Self)
            .map_err(|err| Error::crypto(format!("invalid PHC encoding: {err}")))
    }

    /// Gets a reference to the inner slice.
    pub const fn as_slice(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Converts the [Password] into a byte vector.
    pub fn into_bytes(self) -> Vec<u8> {
        self.0.clone().into_bytes()
    }

    /// Gets the [Password] length.
    #[inline]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Gets whether the [Password] is all zeroes.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Default for Password {
    fn default() -> Self {
        Self::new()
    }
}

impl AsRef<[u8]> for Password {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl TryFrom<&[u8]> for Password {
    type Error = Error;

    fn try_from(val: &[u8]) -> Result<Self> {
        Self::from_slice(val)
    }
}

impl<const N: usize> TryFrom<&[u8; N]> for Password {
    type Error = Error;

    fn try_from(val: &[u8; N]) -> Result<Self> {
        val.as_ref().try_into()
    }
}

impl<const N: usize> TryFrom<[u8; N]> for Password {
    type Error = Error;

    fn try_from(val: [u8; N]) -> Result<Self> {
        val.as_ref().try_into()
    }
}

impl TryFrom<Vec<u8>> for Password {
    type Error = Error;

    fn try_from(val: Vec<u8>) -> Result<Self> {
        val.as_slice().try_into()
    }
}
