use base64::Encoding;
use serde::{de, ser};
use sha3::TurboShake256;
use sha3::digest::{ExtendableOutput, Update, XofReader};
use zeroize::{Zeroize, ZeroizeOnDrop};

const SECRET_LEN: usize = 32;

/// Represents an OAuth-2.0 `client_secret` used to authenticate a client.
#[derive(Clone, Debug, Eq, PartialEq, Zeroize, ZeroizeOnDrop)]
pub struct ClientSecret([u8; SECRET_LEN]);

impl ClientSecret {
    /// Represents the byte length.
    pub const LEN: usize = SECRET_LEN;
    pub const HASH_CONTEXT: u8 = 8;

    /// Creates a new [ClientSecret].
    pub const fn new() -> Self {
        Self([0u8; Self::LEN])
    }

    /// Creates a new random [ClientSecret].
    pub fn random() -> Self {
        let mut secret = Self::new();
        rand::fill(secret.as_mut());
        Self::hash(secret.as_ref())
    }

    /// Hashes input data to create a new [ClientSecret].
    pub fn hash(val: &[u8]) -> Self {
        let mut hasher = Self::hasher();
        hasher.update(val);

        let mut secret = ClientSecret::new();
        let mut reader = hasher.finalize_xof();
        reader.read(secret.as_mut());

        secret
    }

    /// Gets an [ClientSecret] hasher instance.
    #[inline]
    pub fn hasher() -> TurboShake256 {
        TurboShake256::from_core(sha3::TurboShake256Core::new(Self::HASH_CONTEXT))
    }

    /// Converts a slice into a [ClientSecret].
    pub fn from_slice(val: &[u8]) -> Self {
        let mut secret = Self::new();
        let len = core::cmp::min(Self::LEN, val.len());
        secret.0[..len].copy_from_slice(&val[..len]);
        secret
    }

    /// Gets a reference to the inner slice.
    pub const fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Gets the [ClientSecret] length.
    pub const fn len(&self) -> usize {
        Self::LEN
    }

    /// Gets whether the [ClientSecret] is all zeroes.
    pub fn is_empty(&self) -> bool {
        self.0 == [0u8; Self::LEN]
    }
}

impl Default for ClientSecret {
    fn default() -> Self {
        Self::new()
    }
}

impl AsRef<[u8]> for ClientSecret {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsMut<[u8]> for ClientSecret {
    fn as_mut(&mut self) -> &mut [u8] {
        self.0.as_mut()
    }
}

impl From<&[u8]> for ClientSecret {
    fn from(val: &[u8]) -> Self {
        Self::from_slice(val)
    }
}

impl<const N: usize> From<&[u8; N]> for ClientSecret {
    fn from(val: &[u8; N]) -> Self {
        val.as_ref().into()
    }
}

impl<const N: usize> From<[u8; N]> for ClientSecret {
    fn from(val: [u8; N]) -> Self {
        val.as_ref().into()
    }
}

impl From<Vec<u8>> for ClientSecret {
    fn from(val: Vec<u8>) -> Self {
        val.as_slice().into()
    }
}

impl std::fmt::Display for ClientSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            base64::Base64UrlUnpadded::encode_string(self.as_ref())
        )
    }
}

impl ser::Serialize for ClientSecret {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        base64::Base64UrlUnpadded::encode_string(self.as_ref()).serialize(serializer)
    }
}

impl<'de> de::Deserialize<'de> for ClientSecret {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        <&str>::deserialize(deserializer).and_then(|s| {
            base64::Base64UrlUnpadded::decode_vec(s)
                .map(Self::from)
                .map_err(|err| de::Error::custom(format!("oauth: secret: invalid encoding: {err}")))
        })
    }
}
