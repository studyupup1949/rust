use activitystreams_vocabulary::{impl_default, impl_display};
use base64::Encoding;
use rand::RngExt;

use ed25519::pkcs8::{
    DecodePrivateKey as _, DecodePublicKey as _, EncodePrivateKey as _, EncodePublicKey as _,
};
use rsa::pkcs1::{
    DecodeRsaPrivateKey, DecodeRsaPublicKey, EncodeRsaPrivateKey, EncodeRsaPublicKey, LineEnding,
};
use rsa::pkcs8::{DecodePrivateKey as _, DecodePublicKey as _, EncodePublicKey as _};

use serde::{de, ser};
use zeroize::Zeroizing;

use crate::crypto::KeyType;
use crate::{Error, Result};

/// Convenience alias for a Ed25519 public key.
pub type Ed25519PublicKey = ed25519::VerifyingKey;

/// Convenience alias for a NIST-P256 public key.
pub type Ecdsa256PublicKey = ecdsa::VerifyingKey<p256::NistP256>;

/// Convenience alias for a NIST-P384 public key.
pub type Ecdsa384PublicKey = ecdsa::VerifyingKey<p384::NistP384>;

/// Convenience alias for a Ed25519 private key.
pub type Ed25519PrivateKey = ed25519::SigningKey;

/// Convenience alias for a NIST-P256 private key.
pub type Ecdsa256PrivateKey = ecdsa::SigningKey<p256::NistP256>;

/// Convenience alias for a NIST-P384 private key.
pub type Ecdsa384PrivateKey = ecdsa::SigningKey<p384::NistP384>;

pub use rsa::{RsaPrivateKey, RsaPublicKey};

/// Represents a private signing key.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum PrivateKey {
    Ed25519(Ed25519PrivateKey),
    Ecdsa256(Ecdsa256PrivateKey),
    Ecdsa384(Ecdsa384PrivateKey),
    Rsa(RsaPrivateKey),
}

impl PrivateKey {
    /// Creates a new random [PrivateKey].
    pub fn random(algo: KeyType) -> Result<Self> {
        let mut rng = rand::rng();

        match algo {
            KeyType::Ed25519 => {
                let mut bytes = Zeroizing::new([0u8; 32]);
                rng.fill(bytes.as_mut());
                Ok(Self::Ed25519(Ed25519PrivateKey::from_bytes(&bytes)))
            }
            KeyType::Ecdsa256 => {
                let mut bytes = Zeroizing::new([0u8; 32]);
                rng.fill(bytes.as_mut());
                Ecdsa256PrivateKey::from_slice(bytes.as_ref())
                    .map_err(Error::from)
                    .map(Self::Ecdsa256)
            }
            KeyType::Ecdsa384 => {
                let mut bytes = Zeroizing::new([0u8; 32]);
                rng.fill(bytes.as_mut());
                Ecdsa384PrivateKey::from_slice(bytes.as_ref())
                    .map_err(Error::from)
                    .map(Self::Ecdsa384)
            }
            KeyType::Rsa2048 => RsaPrivateKey::new(&mut rng, 2048)
                .map_err(|err| {
                    Error::crypto(format!(
                        "private_key: error creating RSA-v1.5-SHA256: {err}"
                    ))
                })
                .map(Self::Rsa),
            _ => Err(Error::crypto(format!(
                "private_key: unsupported algorithm: {algo}"
            ))),
        }
    }

    /// Gets the algorithm.
    pub fn algorithm(&self) -> KeyType {
        match self {
            Self::Ed25519(_) => KeyType::Ed25519,
            Self::Ecdsa256(_) => KeyType::Ecdsa256,
            Self::Ecdsa384(_) => KeyType::Ecdsa384,
            Self::Rsa(_) => KeyType::Rsa2048,
        }
    }

    /// Attempts to convert the [PublicKey] to PEM encoding.
    pub fn to_pem(&self) -> Result<Zeroizing<String>> {
        match self {
            Self::Ed25519(key) => key.to_pkcs8_pem(LineEnding::LF).map_err(|err| {
                Error::crypto(format!("error encoding ed25519 private key PEM: {err}"))
            }),
            Self::Ecdsa256(key) => key.to_pkcs8_pem(LineEnding::LF).map_err(|err| {
                Error::crypto(format!("error encoding ecdsa-p256 private key PEM: {err}"))
            }),
            Self::Ecdsa384(key) => key.to_pkcs8_pem(LineEnding::LF).map_err(|err| {
                Error::crypto(format!("error encoding ecdsa-p384 private key PEM: {err}"))
            }),
            Self::Rsa(key) => key.to_pkcs1_pem(LineEnding::LF).map_err(|err| {
                Error::crypto(format!(
                    "error encoding rsa-v1_5-sha256 private key PEM: {err}"
                ))
            }),
        }
    }

    /// Attempts to convert a PEM-encoded ASN.1 string into a private key.
    pub fn from_pem(algo: KeyType, pem: &str) -> Result<Self> {
        match algo {
            KeyType::Ed25519 => Ed25519PrivateKey::from_pkcs8_pem(pem)
                .map(Self::Ed25519)
                .map_err(|err| Error::crypto(format!("invalid ed25519 PEM: {err}"))),
            KeyType::Ecdsa256 => Ecdsa256PrivateKey::from_pkcs8_pem(pem)
                .map(Self::Ecdsa256)
                .map_err(|err| Error::crypto(format!("invalid ecdsa-p256 PEM: {err}"))),
            KeyType::Ecdsa384 => Ecdsa384PrivateKey::from_pkcs8_pem(pem)
                .map(Self::Ecdsa384)
                .map_err(|err| Error::crypto(format!("invalid ecdsa-p384 PEM: {err}"))),
            KeyType::Rsa2048 => RsaPrivateKey::from_pkcs8_pem(pem)
                .map(Self::Rsa)
                .map_err(|err| Error::crypto(format!("invalid rsa PEM: {err}"))),
            _ => Err(Error::crypto(format!("unsupported key algorithm: {algo}"))),
        }
    }

    /// Attempts to convert a byte slice into a private key.
    pub fn from_bytes(algo: KeyType, bytes: &[u8]) -> Result<Self> {
        match algo {
            KeyType::Ed25519 => bytes
                .try_into()
                .map_err(|err| Error::crypto(format!("invalid ed25519 key length: {err}")))
                .map(|k| Self::Ed25519(Ed25519PrivateKey::from_bytes(&k))),
            KeyType::Ecdsa256 => Ecdsa256PrivateKey::from_slice(bytes)
                .map(Self::Ecdsa256)
                .map_err(|err| {
                    Error::crypto(format!(
                        "error converting bytes to ecdsa-p256 private key: {err}"
                    ))
                }),
            KeyType::Ecdsa384 => Ecdsa384PrivateKey::from_slice(bytes)
                .map(Self::Ecdsa384)
                .map_err(|err| {
                    Error::crypto(format!(
                        "error converting bytes to ecdsa-p384 private key: {err}"
                    ))
                }),
            KeyType::Rsa2048 => {
                RsaPrivateKey::from_pkcs1_der(bytes)
                    .map(Self::Rsa)
                    .map_err(|err| {
                        Error::crypto(format!("error converting bytes to rsa private key: {err}"))
                    })
            }
            _ => Err(Error::crypto(format!("unsupported algorithm: {algo}"))),
        }
    }

    /// Attempts to convert a key to encoded bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        match self {
            Self::Ed25519(key) => Ok(key.as_bytes().into()),
            Self::Ecdsa256(key) => key
                .to_pkcs8_der()
                .map(|d| d.as_bytes().into())
                .map_err(|err| Error::crypto(format!("error converting ecdsa-p256 to der: {err}"))),
            Self::Ecdsa384(key) => key
                .to_pkcs8_der()
                .map(|d| d.as_bytes().into())
                .map_err(|err| Error::crypto(format!("error converting ecdsa-p384 to der: {err}"))),
            Self::Rsa(key) => key
                .to_pkcs1_der()
                .map(|d| d.as_bytes().into())
                .map_err(|err| Error::crypto(format!("error converting rsa to der: {err}"))),
        }
    }

    /// Gets the public key.
    pub fn public_key(&self) -> PublicKey {
        match self {
            Self::Ed25519(key) => PublicKey::Ed25519(key.verifying_key()),
            Self::Ecdsa256(key) => PublicKey::Ecdsa256(*key.verifying_key()),
            Self::Ecdsa384(key) => PublicKey::Ecdsa384(*key.verifying_key()),
            Self::Rsa(key) => PublicKey::Rsa(key.to_public_key()),
        }
    }
}

/// Represents a public verifying key.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum PublicKey {
    Ed25519(Ed25519PublicKey),
    Ecdsa256(Ecdsa256PublicKey),
    Ecdsa384(Ecdsa384PublicKey),
    Rsa(RsaPublicKey),
}

impl PublicKey {
    /// Creates a new [PublicKey].
    pub fn new() -> Self {
        Self::Ed25519(Ed25519PrivateKey::from_bytes(&[1u8; 32]).verifying_key())
    }

    /// Gets the algorithm.
    pub fn algorithm(&self) -> KeyType {
        match self {
            Self::Ed25519(_) => KeyType::Ed25519,
            Self::Ecdsa256(_) => KeyType::Ecdsa256,
            Self::Ecdsa384(_) => KeyType::Ecdsa384,
            Self::Rsa(_) => KeyType::Rsa2048,
        }
    }

    /// Attempts to get a reference to the [Ed25519](Self::Ed25519) variant.
    pub fn as_ed25519(&self) -> Result<&ed25519::VerifyingKey> {
        match self {
            Self::Ed25519(key) => Ok(key),
            _ => Err(Error::crypto(format!(
                "invalid key type: {}",
                self.algorithm()
            ))),
        }
    }

    /// Attempts to convert a byte slice into a public key.
    pub fn from_bytes(algo: KeyType, bytes: &[u8]) -> Result<Self> {
        match algo {
            KeyType::Ed25519 => bytes
                .try_into()
                .map_err(|err| Error::crypto(format!("key: invalid ed25519 key length: {err}")))
                .and_then(|k| {
                    Ed25519PublicKey::from_bytes(&k)
                        .map(Self::Ed25519)
                        .map_err(|err| {
                            Error::crypto(format!(
                                "key: error converting bytes to ed25519 key: {err}"
                            ))
                        })
                }),
            KeyType::Ecdsa256 => Ecdsa256PublicKey::from_sec1_bytes(bytes)
                .map(Self::Ecdsa256)
                .map_err(|err| {
                    Error::crypto(format!(
                        "key: error converting bytes to ecdsa-p256 key: {err}"
                    ))
                }),
            KeyType::Ecdsa384 => Ecdsa384PublicKey::from_sec1_bytes(bytes)
                .map(Self::Ecdsa384)
                .map_err(|err| {
                    Error::crypto(format!(
                        "key: error converting bytes to ecdsa-p384 key: {err}"
                    ))
                }),
            KeyType::Rsa2048 => RsaPublicKey::from_pkcs1_der(bytes)
                .map(Self::Rsa)
                .map_err(|err| {
                    Error::crypto(format!(
                        "key: error converting bytes to rsa public key: {err}"
                    ))
                }),
            _ => Err(Error::crypto(format!("key: unsupported algorithm: {algo}"))),
        }
    }

    /// Gets a reference to the key bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        match self {
            Self::Ed25519(key) => Ok(key.as_bytes().into()),
            Self::Ecdsa256(key) => Ok(key.to_encoded_point(true).as_bytes().into()),
            Self::Ecdsa384(key) => Ok(key.to_encoded_point(true).as_bytes().into()),
            Self::Rsa(key) => key
                .to_pkcs1_der()
                .map(|d| d.as_bytes().into())
                .map_err(|err| Error::crypto(format!("key: error converting rsa to der: {err}"))),
        }
    }

    /// Converts the public key to a byte array.
    pub fn to_ecdsa256_bytes(&self) -> Result<[u8; 33]> {
        match self {
            Self::Ecdsa256(key) => key
                .to_encoded_point(true)
                .as_bytes()
                .try_into()
                .map_err(|err| Error::crypto(format!("key: invalid key length: {err}"))),
            _ => Err(Error::crypto("key: invalid public key variant")),
        }
    }

    /// Converts the public key to a byte array.
    pub fn to_ecdsa384_bytes(&self) -> Result<[u8; 49]> {
        match self {
            Self::Ecdsa384(key) => key
                .to_encoded_point(true)
                .as_bytes()
                .try_into()
                .map_err(|err| Error::crypto(format!("key: invalid key length: {err}"))),
            _ => Err(Error::crypto("key: invalid public key variant")),
        }
    }

    /// Converts the public key to a byte array.
    pub fn to_ed25519_bytes(&self) -> Result<[u8; 32]> {
        match self {
            Self::Ed25519(key) => Ok(*key.as_bytes()),
            _ => Err(Error::crypto("key: invalid public key variant")),
        }
    }

    /// Attempts to convert the [PublicKey] to PEM encoding.
    pub fn to_pem(&self) -> Result<String> {
        match self {
            Self::Ed25519(key) => key.to_public_key_pem(LineEnding::LF).map_err(|err| {
                Error::crypto(format!("error encoding ed25519 public key PEM: {err}"))
            }),
            Self::Ecdsa256(key) => key.to_public_key_pem(LineEnding::LF).map_err(|err| {
                Error::crypto(format!("error encoding ecdsa-p256 public key PEM: {err}"))
            }),
            Self::Ecdsa384(key) => key.to_public_key_pem(LineEnding::LF).map_err(|err| {
                Error::crypto(format!("error encoding ecdsa-p384 public key PEM: {err}"))
            }),
            Self::Rsa(key) => key.to_public_key_pem(LineEnding::LF).map_err(|err| {
                Error::crypto(format!(
                    "error encoding rsa-v1_5-sha256 public key PEM: {err}"
                ))
            }),
        }
    }

    /// Attempts to convert a PEM-encoded value into a [PublicKey].
    pub fn from_pem<S: AsRef<str>>(algo: KeyType, pem: S) -> Result<Self> {
        let s = pem.as_ref();

        match algo {
            KeyType::Ed25519 => Ed25519PublicKey::from_public_key_pem(s)
                .map(PublicKey::Ed25519)
                .map_err(|err| Error::crypto(format!("invalid ed25519 PEM encoding: {err}"))),
            KeyType::Ecdsa256 => Ecdsa256PublicKey::from_public_key_pem(s)
                .map(PublicKey::Ecdsa256)
                .map_err(|err| Error::crypto(format!("invalid ecdsa-p256 PEM encoding: {err}"))),
            KeyType::Ecdsa384 => Ecdsa384PublicKey::from_public_key_pem(s)
                .map(PublicKey::Ecdsa384)
                .map_err(|err| Error::crypto(format!("invalid ecdsa-384 PEM encoding: {err}"))),
            KeyType::Rsa2048 => RsaPublicKey::from_public_key_pem(s)
                .map(PublicKey::Rsa)
                .map_err(|err| {
                    Error::crypto(format!("invalid rsa-v1_5-sha256 PEM encoding: {err}"))
                }),
            _ => Err(Error::crypto(format!("unsupported key algorithm: {algo}"))),
        }
    }

    /// Converts the [PublicKey] to ASN.1 DER-encoded bytes.
    pub fn to_der(&self) -> Result<Vec<u8>> {
        match self {
            Self::Ed25519(key) => key
                .to_public_key_der()
                .map(|k| k.as_bytes().to_vec())
                .map_err(|err| {
                    Error::crypto(format!("public_key: error converting to DER: {err}"))
                }),
            Self::Ecdsa256(key) => key
                .to_public_key_der()
                .map(|k| k.as_bytes().to_vec())
                .map_err(|err| {
                    Error::crypto(format!("public_key: error converting to DER: {err}"))
                }),
            Self::Ecdsa384(key) => key
                .to_public_key_der()
                .map(|k| k.as_bytes().to_vec())
                .map_err(|err| {
                    Error::crypto(format!("public_key: error converting to DER: {err}"))
                }),
            Self::Rsa(key) => key
                .to_public_key_der()
                .map(|k| k.as_bytes().to_vec())
                .map_err(|err| {
                    Error::crypto(format!("public_key: error converting to DER: {err}"))
                }),
        }
    }
}

impl ser::Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.to_pem()
            .map_err(|err| ser::Error::custom(err.to_string()))
            .and_then(|pem| pem.serialize(serializer))
    }
}

impl<'de> de::Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        <String>::deserialize(deserializer).and_then(|s| {
            Self::from_pem(KeyType::Ed25519, &s)
                .or_else(|_| Self::from_pem(KeyType::Ecdsa256, &s))
                .or_else(|_| Self::from_pem(KeyType::Ecdsa384, &s))
                .or_else(|_| Self::from_pem(KeyType::Rsa2048, &s))
                .map_err(|err| de::Error::custom(err.to_string()))
        })
    }
}

impl_default!(PublicKey);
impl_display!(PublicKey, json);

impl TryFrom<jwt::jwk::Jwk> for PublicKey {
    type Error = Error;

    fn try_from(val: jwt::jwk::Jwk) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&jwt::jwk::Jwk> for PublicKey {
    type Error = Error;

    fn try_from(val: &jwt::jwk::Jwk) -> Result<Self> {
        match &val.algorithm {
            jwt::jwk::AlgorithmParameters::OctetKeyPair(okp)
                if matches!(okp.curve, jwt::jwk::EllipticCurve::P256) =>
            {
                base64::Base64UrlUnpadded::decode_vec(okp.x.as_str())
                    .map_err(|err| Error::crypto(format!("public key: {err}")))
                    .and_then(|sec1| {
                        Ecdsa256PublicKey::from_sec1_bytes(&sec1)
                            .map_err(|err| Error::crypto(format!("public key: {err}")))
                    })
                    .map(Self::Ecdsa256)
            }
            jwt::jwk::AlgorithmParameters::OctetKeyPair(okp)
                if matches!(okp.curve, jwt::jwk::EllipticCurve::P384) =>
            {
                base64::Base64UrlUnpadded::decode_vec(okp.x.as_str())
                    .map_err(|err| Error::crypto(format!("public key: {err}")))
                    .and_then(|sec1| {
                        Ecdsa384PublicKey::from_sec1_bytes(&sec1)
                            .map_err(|err| Error::crypto(format!("public key: {err}")))
                    })
                    .map(Self::Ecdsa384)
            }
            jwt::jwk::AlgorithmParameters::OctetKeyPair(okp)
                if matches!(okp.curve, jwt::jwk::EllipticCurve::Ed25519) =>
            {
                base64::Base64UrlUnpadded::decode_vec(okp.x.as_str())
                    .map_err(|err| Error::crypto(format!("public key: {err}")))
                    .and_then(|key| {
                        <[u8; 32]>::try_from(key.as_slice()).map_err(|err| {
                            Error::crypto(format!("public key: invalid Ed25519: {err}"))
                        })
                    })
                    .and_then(|key| {
                        Ed25519PublicKey::from_bytes(&key)
                            .map_err(|err| Error::crypto(format!("public key: {err}")))
                    })
                    .map(Self::Ed25519)
            }
            algo => Err(Error::crypto(format!(
                "public key: unsupported algorithm: {algo:?}"
            ))),
        }
    }
}

impl TryFrom<PublicKey> for jwt::jwk::Jwk {
    type Error = Error;

    fn try_from(val: PublicKey) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&PublicKey> for jwt::jwk::Jwk {
    type Error = Error;

    fn try_from(val: &PublicKey) -> Result<Self> {
        match val {
            PublicKey::Ecdsa256(k) => Ok(jwt::jwk::Jwk {
                algorithm: jwt::jwk::AlgorithmParameters::OctetKeyPair(
                    jwt::jwk::OctetKeyPairParameters {
                        curve: jwt::jwk::EllipticCurve::P256,
                        x: base64::Base64UrlUnpadded::encode_string(
                            k.to_encoded_point(true).as_bytes(),
                        ),
                        ..Default::default()
                    },
                ),
                common: jwt::jwk::CommonParameters::default(),
            }),
            PublicKey::Ecdsa384(k) => Ok(jwt::jwk::Jwk {
                algorithm: jwt::jwk::AlgorithmParameters::OctetKeyPair(
                    jwt::jwk::OctetKeyPairParameters {
                        curve: jwt::jwk::EllipticCurve::P384,
                        x: base64::Base64UrlUnpadded::encode_string(
                            k.to_encoded_point(true).as_bytes(),
                        ),
                        ..Default::default()
                    },
                ),
                common: jwt::jwk::CommonParameters::default(),
            }),
            PublicKey::Ed25519(k) => Ok(jwt::jwk::Jwk {
                algorithm: jwt::jwk::AlgorithmParameters::OctetKeyPair(
                    jwt::jwk::OctetKeyPairParameters {
                        curve: jwt::jwk::EllipticCurve::Ed25519,
                        x: base64::Base64UrlUnpadded::encode_string(&k.to_bytes()),
                        ..Default::default()
                    },
                ),
                common: jwt::jwk::CommonParameters::default(),
            }),
            _ => Err(Error::crypto(format!(
                "public key: unsupported JWK key: {}",
                val.algorithm()
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecdsa256_key() {
        println!("rand UUID: {}", crate::util::rand_uuid());
        let privkey = Ecdsa256PrivateKey::from_slice([1u8; 32].as_ref()).unwrap();
        let pubkey = PublicKey::Ecdsa256(*privkey.verifying_key());
        let pubkey_bytes = pubkey.to_ecdsa256_bytes().unwrap();

        assert_eq!(pubkey.to_bytes().as_deref(), Ok(pubkey_bytes.as_ref()));
        assert_eq!(
            PublicKey::from_bytes(KeyType::Ecdsa256, pubkey_bytes.as_ref()),
            Ok(pubkey)
        );
    }

    #[test]
    fn test_ecdsa384_key() {
        let privkey = Ecdsa384PrivateKey::from_slice([1u8; 32].as_ref()).unwrap();
        let pubkey = PublicKey::Ecdsa384(*privkey.verifying_key());
        let pubkey_bytes = pubkey.to_ecdsa384_bytes().unwrap();

        assert_eq!(pubkey.to_bytes().as_deref(), Ok(pubkey_bytes.as_ref()));
        assert_eq!(
            PublicKey::from_bytes(KeyType::Ecdsa384, pubkey_bytes.as_ref()),
            Ok(pubkey)
        );
    }

    #[test]
    fn test_ed25519_key() {
        let privkey = Ed25519PrivateKey::from_bytes(&[1u8; 32]);
        let pubkey = PublicKey::Ed25519(privkey.verifying_key());
        let pubkey_bytes = pubkey.to_ed25519_bytes().unwrap();

        assert_eq!(pubkey.to_bytes().as_deref(), Ok(pubkey_bytes.as_ref()));
        assert_eq!(
            PublicKey::from_bytes(KeyType::Ed25519, pubkey_bytes.as_ref()),
            Ok(pubkey)
        );
    }
}
