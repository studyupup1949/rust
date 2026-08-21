use activitystreams_vocabulary::{field_access, impl_default, impl_display};
use base64::Encoding;
use serde::{de, ser};

use crate::{Error, Result};

/// Represents the OpenSSH public key types.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OpenSshKeyType {
    Ecdsa256,
    Ecdsa384,
    Ed25519,
    Rsa,
}

impl OpenSshKeyType {
    /// String representation for [Ecdsa256](Self::Ecdsa256) variant.
    pub const ECDSA_256: &str = "ecdsa-sha2-nistp256";
    /// String representation for [Ecdsa384](Self::Ecdsa384) variant.
    pub const ECDSA_384: &str = "ecdsa-sha2-nistp384";
    /// String representation for [Ed25519](Self::Ed25519) variant.
    pub const ED25519: &str = "ssh-ed25519";
    /// String representation for [Rsa](Self::Rsa) variant.
    pub const RSA: &str = "ssh-rsa";

    /// Represents the OpenSSH-encoded ECDSA-NISTP256 key length (in bytes).
    pub const ECDSA_256_LEN: usize = 104;
    /// Represents the OpenSSH-encoded ECDSA-NISTP384 key length (in bytes).
    pub const ECDSA_384_LEN: usize = 136;
    /// Represents the OpenSSH-encoded Ed25519 key length (in bytes).
    pub const ED25519_LEN: usize = 51;
    /// Represents the OpenSSH-encoded RSA-2048 key length (in bytes).
    pub const RSA_2048_LEN: usize = 279;
    /// Represents the OpenSSH-encoded RSA-3072 key length (in bytes).
    pub const RSA_3072_LEN: usize = 407;
    /// Represents the OpenSSH-encoded RSA-4096 key length (in bytes).
    pub const RSA_4096_LEN: usize = 535;

    /// Creates a new [OpenSshKeyType].
    pub const fn new() -> Self {
        Self::Ed25519
    }

    /// Gets the [OpenSshKeyType] string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Ecdsa256 => Self::ECDSA_256,
            Self::Ecdsa384 => Self::ECDSA_384,
            Self::Ed25519 => Self::ED25519,
            Self::Rsa => Self::RSA,
        }
    }

    /// Checks the key length matches the expected length based on key type.
    pub fn check_key_len(&self, key_bytes: &[u8]) -> Result<()> {
        match (self, key_bytes.len()) {
            (Self::Ecdsa256, Self::ECDSA_256_LEN) => Ok(()),
            (Self::Ecdsa384, Self::ECDSA_384_LEN) => Ok(()),
            (Self::Ed25519, Self::ED25519_LEN) => Ok(()),
            (Self::Rsa, Self::RSA_2048_LEN | Self::RSA_3072_LEN | Self::RSA_4096_LEN) => Ok(()),
            (ty, len) => Err(Error::object(format!(
                "ssh_key: invalid key length: {len}, type: {ty}"
            ))),
        }
    }
}

impl_default!(OpenSshKeyType);
impl_display!(OpenSshKeyType, str);

impl TryFrom<&str> for OpenSshKeyType {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self> {
        match val {
            Self::ECDSA_256 => Ok(Self::Ecdsa256),
            Self::ECDSA_384 => Ok(Self::Ecdsa384),
            Self::ED25519 => Ok(Self::Ed25519),
            Self::RSA => Ok(Self::Rsa),
            _ => Err(Error::object(format!(
                "ssh_key: invalid public key type: {val}"
            ))),
        }
    }
}

impl TryFrom<String> for OpenSshKeyType {
    type Error = Error;

    fn try_from(val: String) -> Result<Self> {
        val.as_str().try_into()
    }
}

/// Represents an [OpenSSH public key](https://sshref.dev/#intro-legc-pub).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenSshPublicKey {
    key_type: OpenSshKeyType,
    key_bytes: Vec<u8>,
    comment: Option<String>,
}

impl OpenSshPublicKey {
    /// Creates a new [OpenSshPublicKey].
    pub const fn new() -> Self {
        Self {
            key_type: OpenSshKeyType::new(),
            key_bytes: Vec::new(),
            comment: None,
        }
    }
}

field_access! {
    OpenSshPublicKey {
        /// Represents the OpenSSH key type header.
        key_type: OpenSshKeyType,
    }
}

field_access! {
    OpenSshPublicKey {
        /// Represents the OpenSSH key bytes.
        ///
        /// The key bytes are Base64-decoded, but still in their OpenSSH-encoded format.
        key_bytes: as_ref { &[u8], Vec<u8> },
    }
}

field_access! {
    OpenSshPublicKey {
        /// Represents the OpenSSH key comment.
        comment: option_deref { &str, String },
    }
}

impl_default!(OpenSshPublicKey);

impl From<OpenSshPublicKey> for String {
    fn from(val: OpenSshPublicKey) -> Self {
        val.to_string()
    }
}

impl From<&OpenSshPublicKey> for String {
    fn from(val: &OpenSshPublicKey) -> Self {
        val.to_string()
    }
}

impl TryFrom<&str> for OpenSshPublicKey {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self> {
        let mut parts = val.splitn(3, ' ');

        let key_type = parts
            .next()
            .ok_or(Error::object("ssh_key: missing key type"))
            .and_then(OpenSshKeyType::try_from)?;

        let key_bytes = parts
            .next()
            .ok_or(Error::object("ssh_key: missing key bytes"))
            .and_then(|s| {
                base64::Base64::decode_vec(s).map_err(|err| {
                    Error::object(format!("ssh_key: invalid key bytes encoding: {err}"))
                })
            })
            .and_then(|k| key_type.check_key_len(k.as_ref()).map(|_| k))?;

        let comment = parts.next().map(|s| s.to_string());

        Ok(Self {
            key_type,
            key_bytes,
            comment,
        })
    }
}

impl TryFrom<String> for OpenSshPublicKey {
    type Error = Error;

    fn try_from(val: String) -> Result<Self> {
        val.as_str().try_into()
    }
}

impl TryFrom<&String> for OpenSshPublicKey {
    type Error = Error;

    fn try_from(val: &String) -> Result<Self> {
        val.as_str().try_into()
    }
}

impl core::fmt::Display for OpenSshPublicKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let key_type = self.key_type();
        let key_bytes = base64::Base64::encode_string(self.key_bytes());

        if let Some(comment) = self.comment() {
            write!(f, "{key_type} {key_bytes} {comment}")
        } else {
            write!(f, "{key_type} {key_bytes}")
        }
    }
}

impl ser::Serialize for OpenSshPublicKey {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.key_type
            .check_key_len(self.key_bytes.as_ref())
            .map_err(|err| ser::Error::custom(err.to_string()))
            .and_then(|_| self.to_string().serialize(serializer))
    }
}

impl<'de> de::Deserialize<'de> for OpenSshPublicKey {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        <&str>::deserialize(deserializer).and_then(|s| {
            OpenSshPublicKey::try_from(s).map_err(|err| de::Error::custom(err.to_string()))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_ssh_public_key() {
        [
            ("ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJQ2i70G8zuqjrEaIyxWQRvuivLjqELGDp71JBEHHhyd somekey@example.dev", OpenSshKeyType::Ed25519),

            ("ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQCvbhT4acVkIRd/1N7DzSD+dftckMm5bu05d9qRFatv5F4RFuUdEZZUDW3G4HnC4BY0E6yMBDPJnFjhq2wQc0f8hHt9sCor2DdOxTwOFoHL31V3avC29YWWOFfJ6f4QXg8M5oUHqBLWdgoFjc7i+W3uCnbCdvv7shM6kzygJioZzGv9RNS55++H+91EiMHjSmhcOyoSQiQ/aIicMAEVM2Z0gM3t9hD658pT5i0XpaGb8ssW7TAC04ArNr7Ae113d3trGw7/uPLGIf6PrnBzaM6vQ70B2poqiZ+U6fERpgXAZP0/AbGl64X3Pn7EiXbIRi8PFAVz+q6DpUD5SB090uB9Lka/n6QgJRMDFrRZCoUrVGTEoBArhSvZZ+I5QzRfFkDaNwRYzErAKjFt1iE4O/QmIRyGIhfywlC55emdfGr6zGP5Hq6EdzcPoZTTF8obIeBc4HAUJIh4Rhr5yrca22kr9pWv3pvOoSh6TK7miDa7XOiUa5q3MGqw7fWDi/r//jE= somekey@example.dev", OpenSshKeyType::Rsa),

            ("ecdsa-sha2-nistp256 AAAAE2VjZHNhLXNoYTItbmlzdHAyNTYAAAAIbmlzdHAyNTYAAABBBDKinKm1QTz4m+BXMjGXxveT9v6xEGFpngFKKO7pCH7VszZHl3JHuMza7XFRhsdftEA4bT2wLj/e4tnjXeJN9/s= somekey@example.dev", OpenSshKeyType::Ecdsa256),
            ("ecdsa-sha2-nistp384 AAAAE2VjZHNhLXNoYTItbmlzdHAzODQAAAAIbmlzdHAzODQAAABhBLbpT5+Ui9cVD8B7t67U+CCTQU6d6ZfNepXavGrXbtaLtl+568xrF5qCSDkzHSrh2osJGb0Q6mX7UHF76EC1Lbjf4ikOISknlgX7BWJ9NDLIdRc7/RbS5NIP77fIojTN6A== somekey@example.dev", OpenSshKeyType::Ecdsa384),
        ].into_iter().for_each(|(key_str, key_type)| {
            let json_str = format!(r#""{key_str}""#);
            let key = OpenSshPublicKey::try_from(key_str).unwrap();

            assert_eq!(key.key_type(), key_type);
            assert_eq!(serde_json::to_string(&key).unwrap(), json_str);
            assert_eq!(serde_json::from_str::<OpenSshPublicKey>(&json_str).unwrap(), key);
        });
    }
}
