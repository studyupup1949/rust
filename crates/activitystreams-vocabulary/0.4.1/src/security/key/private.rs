use serde::{de, ser};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{Error, Result, impl_default};

/// Represents a PEM-encoded private key.
#[derive(Clone, Debug, Eq, PartialEq, Zeroize, ZeroizeOnDrop)]
pub struct PrivateKeyPem {
    label: String,
    key: Vec<u8>,
}

impl PrivateKeyPem {
    /// Represents the PEM label.
    pub const LABEL: &str = "PRIVATE KEY";

    /// creates a new [PrivateKeyPem].
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

    /// Sets the PEM-decoded key bytes.
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

impl_default!(PrivateKeyPem);

impl core::fmt::Display for PrivateKeyPem {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        pem::encode_string(self.label.as_str(), pem::LineEnding::LF, self.key.as_ref())
            .map_err(|_| core::fmt::Error)
            .and_then(|s| write!(f, "{s}"))
    }
}

impl TryFrom<&[u8]> for PrivateKeyPem {
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

impl TryFrom<&str> for PrivateKeyPem {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self> {
        val.as_bytes().try_into()
    }
}

impl TryFrom<String> for PrivateKeyPem {
    type Error = Error;

    fn try_from(val: String) -> Result<Self> {
        val.as_str().try_into()
    }
}

impl TryFrom<&String> for PrivateKeyPem {
    type Error = Error;

    fn try_from(val: &String) -> Result<Self> {
        val.as_str().try_into()
    }
}

impl<const N: usize> TryFrom<&[u8; N]> for PrivateKeyPem {
    type Error = Error;

    fn try_from(val: &[u8; N]) -> Result<Self> {
        val.as_ref().try_into()
    }
}

impl<const N: usize> TryFrom<[u8; N]> for PrivateKeyPem {
    type Error = Error;

    fn try_from(val: [u8; N]) -> Result<Self> {
        val.as_ref().try_into()
    }
}

impl TryFrom<Vec<u8>> for PrivateKeyPem {
    type Error = Error;

    fn try_from(val: Vec<u8>) -> Result<Self> {
        val.as_slice().try_into()
    }
}

impl ser::Serialize for PrivateKeyPem {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> de::Deserialize<'de> for PrivateKeyPem {
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
    fn test_private_key() {
        let encoded = "-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEIBftnHPp22SewYmmEoMcX8VwI4IHwaqd+9LFPj/15eqF
-----END PRIVATE KEY-----
";

        let json_str = serde_json::to_string(encoded).unwrap();

        let key_bytes = [
            0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22,
            0x04, 0x20, 0x17, 0xed, 0x9c, 0x73, 0xe9, 0xdb, 0x64, 0x9e, 0xc1, 0x89, 0xa6, 0x12,
            0x83, 0x1c, 0x5f, 0xc5, 0x70, 0x23, 0x82, 0x07, 0xc1, 0xaa, 0x9d, 0xfb, 0xd2, 0xc5,
            0x3e, 0x3f, 0xf5, 0xe5, 0xea, 0x85,
        ];

        let key = PrivateKeyPem::new().with_key(key_bytes);

        assert_eq!(key.label(), PrivateKeyPem::LABEL);
        assert_eq!(key.key(), &key_bytes);
        assert_eq!(key.to_string(), encoded);
        assert_eq!(PrivateKeyPem::try_from(encoded).unwrap(), key);

        assert_eq!(serde_json::to_string(&key).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<PrivateKeyPem>(&json_str).unwrap(),
            key
        );
    }
}
