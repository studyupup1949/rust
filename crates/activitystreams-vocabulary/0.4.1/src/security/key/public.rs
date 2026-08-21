use serde::{de, ser};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{Error, Result, impl_default};

/// Represents a PEM-encoded public key.
#[derive(Clone, Debug, Eq, PartialEq, Zeroize, ZeroizeOnDrop)]
pub struct PublicKeyPem {
    label: String,
    key: Vec<u8>,
}

impl PublicKeyPem {
    /// Represents the PEM label.
    pub const LABEL: &str = "PUBLIC KEY";

    /// creates a new [PublicKeyPem].
    #[inline]
    pub fn new() -> Self {
        Self {
            label: Self::LABEL.to_string(),
            key: Vec::new(),
        }
    }

    /// Gets the PEM label.
    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    /// Sets the PEM label.
    pub fn set_label<I: Into<String>>(&mut self, val: I) {
        self.label = val.into()
    }

    /// Builder function that sets the PEM label.
    pub fn with_label<I: Into<String>>(self, val: I) -> Self {
        Self {
            label: val.into(),
            key: self.key.clone(),
        }
    }

    /// Gets the PEM-decoded key bytes.
    pub fn key(&self) -> &[u8] {
        self.key.as_ref()
    }

    /// Gets the PEM-decoded key bytes.
    pub fn set_key<I: Into<Vec<u8>>>(&mut self, val: I) {
        self.key = val.into();
    }

    /// Builder function that sets the PEM-decoded key bytes.
    pub fn with_key<I: Into<Vec<u8>>>(self, val: I) -> Self {
        Self {
            key: val.into(),
            label: self.label.clone(),
        }
    }
}

impl_default!(PublicKeyPem);

impl core::fmt::Display for PublicKeyPem {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        pem::encode_string(self.label.as_str(), pem::LineEnding::LF, self.key.as_ref())
            .map_err(|_| core::fmt::Error)
            .and_then(|s| write!(f, "{s}"))
    }
}

impl TryFrom<&[u8]> for PublicKeyPem {
    type Error = Error;

    fn try_from(val: &[u8]) -> Result<Self> {
        pem::decode_vec(val)
            .map_err(|err| Error::key(format!("invalid PEM: {err}")))
            .map(|(label, key)| Self {
                label: label.to_string(),
                key,
            })
    }
}

impl TryFrom<&str> for PublicKeyPem {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self> {
        val.as_bytes().try_into()
    }
}

impl TryFrom<String> for PublicKeyPem {
    type Error = Error;

    fn try_from(val: String) -> Result<Self> {
        val.as_str().try_into()
    }
}

impl TryFrom<&String> for PublicKeyPem {
    type Error = Error;

    fn try_from(val: &String) -> Result<Self> {
        val.as_str().try_into()
    }
}

impl<const N: usize> TryFrom<&[u8; N]> for PublicKeyPem {
    type Error = Error;

    fn try_from(val: &[u8; N]) -> Result<Self> {
        val.as_ref().try_into()
    }
}

impl<const N: usize> TryFrom<[u8; N]> for PublicKeyPem {
    type Error = Error;

    fn try_from(val: [u8; N]) -> Result<Self> {
        val.as_ref().try_into()
    }
}

impl TryFrom<Vec<u8>> for PublicKeyPem {
    type Error = Error;

    fn try_from(val: Vec<u8>) -> Result<Self> {
        val.as_slice().try_into()
    }
}

impl ser::Serialize for PublicKeyPem {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> de::Deserialize<'de> for PublicKeyPem {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        <String>::deserialize(deserializer)
            .and_then(|s| Self::try_from(s).map_err(|err| de::Error::custom(err.to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_key() {
        let encoded = "-----BEGIN PUBLIC KEY-----
9IiXDqOOsPkAZIpt7CoJC9pFkd9w8Z7USKjNa7AVmA+rZbZ0C/BSeh0Ywy4ZrncS
-----END PUBLIC KEY-----
";

        let json_str = serde_json::to_string(encoded).unwrap();

        let key_bytes = [
            0xf4, 0x88, 0x97, 0x0e, 0xa3, 0x8e, 0xb0, 0xf9, 0x00, 0x64, 0x8a, 0x6d, 0xec, 0x2a,
            0x09, 0x0b, 0xda, 0x45, 0x91, 0xdf, 0x70, 0xf1, 0x9e, 0xd4, 0x48, 0xa8, 0xcd, 0x6b,
            0xb0, 0x15, 0x98, 0x0f, 0xab, 0x65, 0xb6, 0x74, 0x0b, 0xf0, 0x52, 0x7a, 0x1d, 0x18,
            0xc3, 0x2e, 0x19, 0xae, 0x77, 0x12,
        ];

        let key = PublicKeyPem::new().with_key(key_bytes);

        assert_eq!(key.label(), PublicKeyPem::LABEL);
        assert_eq!(key.key(), &key_bytes);
        assert_eq!(key.to_string(), encoded);
        assert_eq!(PublicKeyPem::try_from(encoded).unwrap(), key);

        assert_eq!(serde_json::to_string(&key).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<PublicKeyPem>(&json_str).unwrap(),
            key
        );
    }
}
