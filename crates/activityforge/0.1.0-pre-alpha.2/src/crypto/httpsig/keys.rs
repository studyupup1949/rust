use activitystreams_vocabulary::{Key as PemPublicKey, Multikey, field_access};
use httpsig::prelude::{HttpSigError, HttpSigResult};

use ecdsa::signature::{DigestSigner, DigestVerifier as _};
use ed25519::{Signer as _, Verifier as _};
use rsa::signature::{RandomizedSigner as _, SignatureEncoding as _, Verifier as _};
use sha2::Digest as _;

use crate::crypto::{AlgorithmName, PrivateKey, PublicKey};
use crate::db::Key;
use crate::{Error, Result};

/// Represents a private key used for signing HTTP Message Signatures (RFC 9421).
#[derive(Clone, Debug)]
pub struct HttpPrivateKey {
    key_id: String,
    key: PrivateKey,
}

impl HttpPrivateKey {
    /// Creates a new [HttpPublicKey].
    pub fn new<S, K>(key_id: S, key: K) -> Self
    where
        S: Into<String>,
        K: Into<PrivateKey>,
    {
        Self {
            key_id: key_id.into(),
            key: key.into(),
        }
    }

    /// Gets the public key derived from this [HttpPrivateKey].
    pub fn public_key(&self) -> HttpPublicKey {
        HttpPublicKey::new(self.key_id(), self.key.public_key())
    }

    /// Attempts to convert a byte slice into a [HttpPrivateKey].
    pub fn from_bytes<S: Into<String>>(
        key_id: S,
        algo: AlgorithmName,
        bytes: &[u8],
    ) -> Result<Self> {
        algo.try_into()
            .and_then(|key_type| PrivateKey::from_bytes(key_type, bytes))
            .map(|key| Self {
                key_id: key_id.into(),
                key,
            })
    }

    /// Attempts to convert a byte slice into a [HttpPrivateKey].
    pub fn from_pem<S: Into<String>>(key_id: S, algo: AlgorithmName, pem: &str) -> Result<Self> {
        algo.try_into()
            .and_then(|key_type| PrivateKey::from_pem(key_type, pem))
            .map(|key| Self {
                key_id: key_id.into(),
                key,
            })
    }

    /// Attempts to create a new random [HttpPrivateKey].
    pub fn random<S: Into<String>>(key_id: S, algo: AlgorithmName) -> Result<Self> {
        algo.try_into()
            .and_then(PrivateKey::random)
            .map(|key| Self {
                key_id: key_id.into(),
                key,
            })
    }
}

field_access! {
    HttpPrivateKey {
        key_id: as_ref { &str, String },
    }
}

field_access! {
    HttpPrivateKey {
        key: as_ref { PrivateKey },
    }
}

impl httpsig::prelude::SigningKey for HttpPrivateKey {
    fn sign(&self, data: &[u8]) -> HttpSigResult<Vec<u8>> {
        match self.key() {
            PrivateKey::Ecdsa256(sk) => {
                let mut digest = sha2::Sha256::default();
                digest.update(data);

                let sig: ecdsa::Signature<p256::NistP256> = sk.sign_digest(digest);

                Ok(sig.to_bytes().to_vec())
            }
            PrivateKey::Ecdsa384(sk) => {
                let mut digest = sha2::Sha384::default();
                digest.update(data);

                let sig: ecdsa::Signature<p384::NistP384> = sk.sign_digest(digest);

                Ok(sig.to_bytes().to_vec())
            }
            PrivateKey::Ed25519(sk) => Ok(sk.sign(data).to_vec()),
            PrivateKey::Rsa(sk) => Ok(rsa::pkcs1v15::SigningKey::<rsa::sha2::Sha256>::new(
                sk.clone(),
            )
            .sign_with_rng(&mut rand::rng(), data)
            .to_vec()),
        }
    }

    fn key_id(&self) -> String {
        self.key_id.clone()
    }

    fn alg(&self) -> AlgorithmName {
        self.key
            .algorithm()
            .try_into()
            .unwrap_or(AlgorithmName::Ed25519)
    }
}

impl TryFrom<HttpPrivateKey> for PemPublicKey {
    type Error = Error;

    fn try_from(val: HttpPrivateKey) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&HttpPrivateKey> for PemPublicKey {
    type Error = Error;

    fn try_from(val: &HttpPrivateKey) -> Result<Self> {
        Key::try_from(val).and_then(Self::try_from)
    }
}

impl TryFrom<HttpPrivateKey> for Multikey {
    type Error = Error;

    fn try_from(val: HttpPrivateKey) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&HttpPrivateKey> for Multikey {
    type Error = Error;

    fn try_from(val: &HttpPrivateKey) -> Result<Self> {
        Key::try_from(val).and_then(Self::try_from)
    }
}

impl httpsig::prelude::VerifyingKey for HttpPrivateKey {
    fn verify(&self, data: &[u8], signature: &[u8]) -> HttpSigResult<()> {
        self.public_key().verify(data, signature)
    }

    fn key_id(&self) -> String {
        self.public_key().key_id().to_string()
    }

    fn alg(&self) -> AlgorithmName {
        self.public_key().alg()
    }
}

/// Represents a private key used for signing HTTP Message Signatures (RFC 9421).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpPublicKey {
    key_id: String,
    key: PublicKey,
}

impl HttpPublicKey {
    /// Creates a new [HttpPublicKey].
    pub fn new<S, K>(key_id: S, key: K) -> Self
    where
        S: Into<String>,
        K: Into<PublicKey>,
    {
        Self {
            key_id: key_id.into(),
            key: key.into(),
        }
    }

    /// Attempts to convert a byte slice into a [HttpPublicKey].
    pub fn from_bytes<S: Into<String>>(
        key_id: S,
        algo: AlgorithmName,
        bytes: &[u8],
    ) -> Result<Self> {
        algo.try_into()
            .and_then(|key_type| PublicKey::from_bytes(key_type, bytes))
            .map(|key| Self {
                key_id: key_id.into(),
                key,
            })
    }

    /// Attempts to convert a byte slice into a [HttpPublicKey].
    pub fn from_pem<S: Into<String>>(key_id: S, algo: AlgorithmName, pem: &str) -> Result<Self> {
        algo.try_into()
            .and_then(|key_type| PublicKey::from_pem(key_type, pem))
            .map(|key| Self {
                key_id: key_id.into(),
                key,
            })
    }
}

field_access! {
    HttpPublicKey {
        key_id: as_ref { &str, String },
    }
}

field_access! {
    HttpPublicKey {
        key: as_ref { PublicKey },
    }
}

impl TryFrom<HttpPublicKey> for PemPublicKey {
    type Error = Error;

    fn try_from(val: HttpPublicKey) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&HttpPublicKey> for PemPublicKey {
    type Error = Error;

    fn try_from(val: &HttpPublicKey) -> Result<Self> {
        Key::try_from(val).and_then(Self::try_from)
    }
}

impl TryFrom<HttpPublicKey> for Multikey {
    type Error = Error;

    fn try_from(val: HttpPublicKey) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&HttpPublicKey> for Multikey {
    type Error = Error;

    fn try_from(val: &HttpPublicKey) -> Result<Self> {
        Key::try_from(val).and_then(Self::try_from)
    }
}

impl httpsig::prelude::VerifyingKey for HttpPublicKey {
    fn verify(&self, data: &[u8], signature: &[u8]) -> HttpSigResult<()> {
        match self.key() {
            PublicKey::Ecdsa256(pk) => {
                let signature = ecdsa::Signature::<p256::NistP256>::from_bytes(signature.into())
                    .map_err(|e| HttpSigError::ParseSignatureError(e.to_string()))?;

                let mut digest = <sha2::Sha256 as sha2::Digest>::new();
                digest.update(data);

                pk.verify_digest(digest, &signature)
                    .map_err(|e| HttpSigError::InvalidSignature(e.to_string()))
            }
            PublicKey::Ecdsa384(pk) => {
                let signature = ecdsa::Signature::<p384::NistP384>::from_bytes(signature.into())
                    .map_err(|e| HttpSigError::ParseSignatureError(e.to_string()))?;

                let mut digest = <sha2::Sha384 as sha2::Digest>::new();
                digest.update(data);

                pk.verify_digest(digest, &signature)
                    .map_err(|e| HttpSigError::InvalidSignature(e.to_string()))
            }
            PublicKey::Ed25519(pk) => {
                let sig = ed25519::Signature::from_slice(signature)
                    .map_err(|e| HttpSigError::ParseSignatureError(e.to_string()))?;
                pk.verify(data, &sig)
                    .map_err(|e| HttpSigError::InvalidSignature(e.to_string()))
            }
            PublicKey::Rsa(pk) => {
                let sig = rsa::pkcs1v15::Signature::try_from(signature)
                    .map_err(|e| HttpSigError::ParseSignatureError(e.to_string()))?;

                rsa::pkcs1v15::VerifyingKey::<rsa::sha2::Sha256>::new(pk.clone())
                    .verify(data, &sig)
                    .map_err(|e| HttpSigError::InvalidSignature(e.to_string()))
            }
        }
    }

    fn alg(&self) -> AlgorithmName {
        self.key
            .algorithm()
            .try_into()
            .unwrap_or(AlgorithmName::Ed25519)
    }

    fn key_id(&self) -> String {
        self.key_id.clone()
    }
}
