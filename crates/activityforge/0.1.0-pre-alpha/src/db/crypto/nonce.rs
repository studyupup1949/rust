use sha3::TurboShake256;
use sha3::digest::{ExtendableOutput, Update, XofReader};
use zeroize::{Zeroize, ZeroizeOnDrop};

const NONCE_LEN: usize = 12;

/// Represents a nonce used for symmetric encryption.
#[derive(Clone, Debug, Eq, PartialEq, Zeroize, ZeroizeOnDrop)]
pub struct Nonce([u8; NONCE_LEN]);

impl Nonce {
    /// Represents the byte length.
    pub const LEN: usize = NONCE_LEN;

    /// Creates a new [Nonce].
    pub const fn new() -> Self {
        Self([0u8; Self::LEN])
    }

    /// Creates a new random [Nonce].
    pub fn random() -> Self {
        let mut nonce = Self::new();
        rand::fill(nonce.as_mut());
        Self::hash(nonce.as_ref())
    }

    /// Hashes input data to create a new [Nonce].
    pub fn hash(val: &[u8]) -> Self {
        let mut hasher = TurboShake256::from_core(sha3::TurboShake256Core::new(4));
        hasher.update(val);

        let mut nonce = Nonce::new();
        let mut reader = hasher.finalize_xof();
        reader.read(nonce.as_mut());

        nonce
    }

    /// Converts a slice into a [Nonce].
    pub fn from_slice(val: &[u8]) -> Self {
        let mut nonce = Self::new();
        let len = core::cmp::min(Self::LEN, val.len());
        nonce.0[..len].copy_from_slice(&val[..len]);
        nonce
    }

    /// Gets a reference to the inner slice.
    pub const fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Converts the [Nonce] to a ChaCha20Poly1305 nonce.
    pub fn as_nonce(&self) -> &chacha20::Nonce {
        chacha20::Nonce::from_slice(&self.0)
    }

    /// Gets the [Nonce] length.
    pub const fn len(&self) -> usize {
        Self::LEN
    }

    /// Gets whether the [Nonce] is all zeroes.
    pub fn is_empty(&self) -> bool {
        self.0 == [0u8; Self::LEN]
    }
}

impl Default for Nonce {
    fn default() -> Self {
        Self::new()
    }
}

impl AsRef<[u8]> for Nonce {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsMut<[u8]> for Nonce {
    fn as_mut(&mut self) -> &mut [u8] {
        self.0.as_mut()
    }
}

impl<'a> From<&'a Nonce> for &'a chacha20::Nonce {
    fn from(val: &'a Nonce) -> Self {
        val.as_nonce()
    }
}

impl From<&[u8]> for Nonce {
    fn from(val: &[u8]) -> Self {
        Self::from_slice(val)
    }
}

impl<const N: usize> From<&[u8; N]> for Nonce {
    fn from(val: &[u8; N]) -> Self {
        val.as_ref().into()
    }
}

impl<const N: usize> From<[u8; N]> for Nonce {
    fn from(val: [u8; N]) -> Self {
        val.as_ref().into()
    }
}

impl From<Vec<u8>> for Nonce {
    fn from(val: Vec<u8>) -> Self {
        val.as_slice().into()
    }
}
