use base64::Encoding;
use sha3::TurboShake256;
use sha3::digest::{ExtendableOutput, Update, XofReader};
use zeroize::{Zeroize, ZeroizeOnDrop};

const SALT_LEN: usize = 32;

/// Represents a salt used for symmetric encryption.
#[derive(Clone, Debug, Eq, PartialEq, Zeroize, ZeroizeOnDrop)]
pub struct Salt([u8; SALT_LEN]);

impl Salt {
    /// Represents the byte length.
    pub const LEN: usize = SALT_LEN;
    pub const HASH_CONTEXT: u8 = 3;

    /// Creates a new [Salt].
    pub const fn new() -> Self {
        Self([0u8; Self::LEN])
    }

    /// Creates a new random [Salt].
    pub fn random() -> Self {
        let mut salt = Self::new();
        rand::fill(salt.as_mut());
        Self::hash(salt.as_ref())
    }

    /// Hashes input data to create a new [Salt].
    pub fn hash(val: &[u8]) -> Self {
        let mut hasher = TurboShake256::from_core(sha3::TurboShake256Core::new(Self::HASH_CONTEXT));
        hasher.update(val);

        let mut salt = Salt::new();
        let mut reader = hasher.finalize_xof();
        reader.read(salt.as_mut());

        salt
    }

    /// Converts a slice into a [Salt].
    pub fn from_slice(val: &[u8]) -> Self {
        let mut salt = Salt::new();
        let len = core::cmp::min(Self::LEN, val.len());
        salt.0[..len].copy_from_slice(&val[..len]);
        salt
    }

    /// Gets a reference to the inner slice.
    pub const fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Gets the [Salt] length.
    pub const fn len(&self) -> usize {
        Self::LEN
    }

    /// Gets whether the [Salt] is all zeroes.
    pub fn is_empty(&self) -> bool {
        self.0 == [0u8; Self::LEN]
    }

    /// Gets the [Salt] Base64-encoded string.
    pub fn as_b64(&self) -> String {
        base64::Base64UrlUnpadded::encode_string(&self.0)
    }

    /// Gets the [Salt] Argon2-compatible Base64-encoded string.
    pub fn as_argon2_b64(&self) -> String {
        base64::Base64Unpadded::encode_string(&self.0)
    }
}

impl Default for Salt {
    fn default() -> Self {
        Self::new()
    }
}

impl AsRef<[u8]> for Salt {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsMut<[u8]> for Salt {
    fn as_mut(&mut self) -> &mut [u8] {
        self.0.as_mut()
    }
}

impl From<&[u8]> for Salt {
    fn from(val: &[u8]) -> Self {
        Self::from_slice(val)
    }
}

impl<const N: usize> From<&[u8; N]> for Salt {
    fn from(val: &[u8; N]) -> Self {
        val.as_ref().into()
    }
}

impl<const N: usize> From<[u8; N]> for Salt {
    fn from(val: [u8; N]) -> Self {
        val.as_ref().into()
    }
}

impl From<Vec<u8>> for Salt {
    fn from(val: Vec<u8>) -> Self {
        val.as_slice().into()
    }
}
