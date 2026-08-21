use activitystreams_vocabulary::impl_default;
use serde::{de, ser};

use crate::{Error, Result};

/// Represents a SHA-1 hash.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Sha1Hash([u8; 20]);
impl Sha1Hash {
    pub const LEN: usize = 20;
    pub const HEX_LEN: usize = Self::LEN * 2;

    /// Creates a new [Sha1Hash].
    pub const fn new() -> Self {
        Self([0u8; Self::LEN])
    }

    /// Gets the [Sha1Hash] length.
    pub const fn len(&self) -> usize {
        Self::LEN
    }

    /// Gets whether the [Sha1Hash] is empty.
    pub const fn is_empty(&self) -> bool {
        false
    }
}

impl_default!(Sha1Hash);

impl TryFrom<String> for Sha1Hash {
    type Error = Error;

    fn try_from(val: String) -> Result<Self> {
        val.as_str().try_into()
    }
}

impl TryFrom<&str> for Sha1Hash {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self> {
        let val_len = val.len();
        if val_len != Self::HEX_LEN {
            Err(Error::hash(format!(
                "invalid SHA1 hash hex length: {val_len}"
            )))
        } else {
            let mut hash = [0u8; Self::LEN];
            for (i, byte) in hash.iter_mut().enumerate() {
                let start = i * 2;
                *byte = u8::from_str_radix(&val[start..start + 2], 16)
                    .map_err(|err| Error::hash(format!("invalid hex byte: {err}")))?;
            }
            Ok(Self(hash))
        }
    }
}

impl TryFrom<&[u8]> for Sha1Hash {
    type Error = Error;

    fn try_from(val: &[u8]) -> Result<Self> {
        <[u8; Self::LEN]>::try_from(val)
            .map(Self)
            .map_err(|_| Error::hash(format!("invalid SHA-1 hash length: {}", val.len())))
    }
}

impl<const N: usize> TryFrom<&[u8; N]> for Sha1Hash {
    type Error = Error;

    fn try_from(val: &[u8; N]) -> Result<Self> {
        val.as_ref().try_into()
    }
}

impl<const N: usize> TryFrom<[u8; N]> for Sha1Hash {
    type Error = Error;

    fn try_from(val: [u8; N]) -> Result<Self> {
        val.as_ref().try_into()
    }
}

impl From<&Sha1Hash> for String {
    fn from(val: &Sha1Hash) -> Self {
        val.0.iter().map(|b| format!("{b:02x}")).collect()
    }
}

impl From<Sha1Hash> for String {
    fn from(val: Sha1Hash) -> Self {
        val.0.into_iter().map(|b| format!("{b:02x}")).collect()
    }
}

impl core::fmt::Display for Sha1Hash {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

impl ser::Serialize for Sha1Hash {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> de::Deserialize<'de> for Sha1Hash {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        <&str>::deserialize(deserializer)
            .and_then(|s| Self::try_from(s).map_err(|err| de::Error::custom(err.to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha1() {
        let hash_str = "109ec9a09c7df7fec775d2ba0b9d466e5643ec8c";
        let hash_json = format!(r#""{hash_str}""#);
        let hash = Sha1Hash::try_from(hash_str).unwrap();

        assert_eq!(hash.to_string(), hash_str);
        assert_eq!(serde_json::to_string(&hash).unwrap(), hash_json);
        assert_eq!(serde_json::from_str::<Sha1Hash>(&hash_json).unwrap(), hash);
    }

    #[test]
    fn test_sha1_invalid() {
        [
            // too short
            "109ec9a09c7df7fec775d2ba0b9d466e5643ec8",
            // too long
            "109ec9a09c7df7fec775d2ba0b9d466e5643ec8cd",
            // invalid hex char
            "109ec9a09c7df7fec775d2ba0b9d466e5643ec8Z",
        ]
        .into_iter()
        .for_each(|invalid_str| {
            let invalid_json = format!(r#""{invalid_str}""#);
            assert!(Sha1Hash::try_from(invalid_str).is_err());
            assert!(Sha1Hash::try_from(invalid_str.as_bytes()).is_err());
            assert!(serde_json::from_str::<Sha1Hash>(&invalid_json).is_err());
        });
    }
}
