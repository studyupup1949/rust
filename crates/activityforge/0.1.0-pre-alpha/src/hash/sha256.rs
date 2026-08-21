use activitystreams_vocabulary::impl_default;
use serde::{de, ser};

use crate::{Error, Result};

/// Represents a SHA-256 hash.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Sha256Hash([u8; 32]);
impl Sha256Hash {
    pub const LEN: usize = 32;
    pub const HEX_LEN: usize = Self::LEN * 2;

    /// Creates a new [Sha256Hash].
    pub const fn new() -> Self {
        Self([0u8; Self::LEN])
    }

    /// Gets the [Sha256Hash] length.
    pub const fn len(&self) -> usize {
        Self::LEN
    }

    /// Gets whether the [Sha256Hash] is empty.
    pub const fn is_empty(&self) -> bool {
        false
    }
}

impl_default!(Sha256Hash);

impl TryFrom<String> for Sha256Hash {
    type Error = Error;

    fn try_from(val: String) -> Result<Self> {
        val.as_str().try_into()
    }
}

impl TryFrom<&str> for Sha256Hash {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self> {
        let val_len = val.len();
        if val_len != Self::HEX_LEN {
            Err(Error::hash(format!(
                "invalid SHA-256 hash hex length: {val_len}"
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

impl TryFrom<&[u8]> for Sha256Hash {
    type Error = Error;

    fn try_from(val: &[u8]) -> Result<Self> {
        <[u8; Self::LEN]>::try_from(val)
            .map(Self)
            .map_err(|_| Error::hash(format!("invalid SHA-256 hash length: {}", val.len())))
    }
}

impl<const N: usize> TryFrom<&[u8; N]> for Sha256Hash {
    type Error = Error;

    fn try_from(val: &[u8; N]) -> Result<Self> {
        val.as_ref().try_into()
    }
}

impl<const N: usize> TryFrom<[u8; N]> for Sha256Hash {
    type Error = Error;

    fn try_from(val: [u8; N]) -> Result<Self> {
        val.as_ref().try_into()
    }
}

impl From<&Sha256Hash> for String {
    fn from(val: &Sha256Hash) -> Self {
        val.0.iter().map(|b| format!("{b:x}")).collect()
    }
}

impl From<Sha256Hash> for String {
    fn from(val: Sha256Hash) -> Self {
        val.0.into_iter().map(|b| format!("{b:x}")).collect()
    }
}

impl core::fmt::Display for Sha256Hash {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

impl ser::Serialize for Sha256Hash {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> de::Deserialize<'de> for Sha256Hash {
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
    fn test_sha256() {
        let hash_str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let hash_json = format!(r#""{hash_str}""#);
        let hash = Sha256Hash::try_from(hash_str).unwrap();

        assert_eq!(hash.to_string(), hash_str);
        assert_eq!(serde_json::to_string(&hash).unwrap(), hash_json);
        assert_eq!(
            serde_json::from_str::<Sha256Hash>(&hash_json).unwrap(),
            hash
        );
    }

    #[test]
    fn test_sha256_invalid() {
        [
            // too short
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b85",
            // too long
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b8555",
            // invalid hex char
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b85$",
        ]
        .into_iter()
        .for_each(|invalid_str| {
            let invalid_json = format!(r#""{invalid_str}""#);
            assert!(Sha256Hash::try_from(invalid_str).is_err());
            assert!(Sha256Hash::try_from(invalid_str.as_bytes()).is_err());
            assert!(serde_json::from_str::<Sha256Hash>(&invalid_json).is_err());
        });
    }
}
