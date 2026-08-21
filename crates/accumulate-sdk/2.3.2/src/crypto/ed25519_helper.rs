//! High-level Ed25519 helper utilities for ease of use
//!
//! This module provides convenient wrapper functions around the core Ed25519 implementation
//! to match the API expected by test files and provide TypeScript SDK compatibility.
//!
//! Updated for ed25519-dalek v2.x API

// Allow expect in this module - cryptographic operations have controlled inputs
#![allow(clippy::expect_used)]

use crate::crypto::ed25519::{Ed25519Signer, verify_signature, sha256};
use crate::codec::{canonical_json, sha256_bytes};
use crate::errors::{Error, SignatureError};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature};
use serde_json::Value;

/// Wrapper around ed25519_dalek::SigningKey to provide a convenient API
/// Updated for ed25519-dalek v2.x
pub struct Keypair {
    pub inner: SigningKey,
    pub public: VerifyingKey,
}

impl std::fmt::Debug for Keypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Keypair")
            .field("public", &hex::encode(self.public.as_bytes()))
            .finish_non_exhaustive()
    }
}

impl Clone for Keypair {
    fn clone(&self) -> Self {
        // In v2, we can clone from the seed bytes
        let seed_bytes = self.inner.to_bytes();
        let signing_key = SigningKey::from_bytes(&seed_bytes);
        Self::new(signing_key)
    }
}

impl Keypair {
    pub fn new(inner: SigningKey) -> Self {
        let public = inner.verifying_key();
        Self { inner, public }
    }
}

impl From<SigningKey> for Keypair {
    fn from(signing_key: SigningKey) -> Self {
        Self::new(signing_key)
    }
}

/// High-level Ed25519 helper providing convenient cryptographic operations
#[derive(Debug, Clone, Copy)]
pub struct Ed25519Helper;

impl Ed25519Helper {
    /// Create a keypair from a hex-encoded private key
    /// This uses standard Ed25519 seed-based key generation to match test vectors
    pub fn keypair_from_hex(hex_key: &str) -> Result<Keypair, Error> {
        let bytes = hex::decode(hex_key)
            .map_err(|_| Error::Signature(SignatureError::InvalidFormat))?;

        if bytes.len() != 32 {
            return Err(Error::Signature(SignatureError::InvalidFormat));
        }

        let mut seed = [0u8; 32];
        seed.copy_from_slice(&bytes);

        // Use standard Ed25519 seed-based key generation
        // This matches Go's crypto/ed25519.NewKeyFromSeed() behavior
        let signing_key = SigningKey::from_bytes(&seed);
        Ok(Keypair::new(signing_key))
    }

    /// Get public key bytes from a keypair
    pub fn public_key_bytes(keypair: &Keypair) -> [u8; 32] {
        keypair.public.to_bytes()
    }

    /// Create a verifying (public) key from bytes
    pub fn public_key_from_bytes(bytes: &[u8; 32]) -> Result<VerifyingKey, Error> {
        VerifyingKey::from_bytes(bytes)
            .map_err(|_| Error::Signature(SignatureError::InvalidPublicKey))
    }

    /// Create a signature from bytes
    pub fn signature_from_bytes(bytes: &[u8; 64]) -> Result<Signature, Error> {
        // In v2, Signature::from_bytes doesn't return Result - it's infallible
        Ok(Signature::from_bytes(bytes))
    }

    /// Sign JSON data with a keypair
    pub fn sign_json(keypair: &Keypair, json_data: &Value) -> Signature {
        let canonical = canonical_json(json_data);
        let message_bytes = canonical.as_bytes();

        // Use the Ed25519Signer to maintain consistency
        let cloned_keypair = keypair.clone();
        let signer = Ed25519Signer::new(cloned_keypair.inner);
        let sig_bytes = signer.sign(message_bytes);

        // Convert [u8; 64] to Signature
        Signature::from_bytes(&sig_bytes)
    }

    /// Verify a signature against JSON data
    pub fn verify_json(public_key: &VerifyingKey, json_data: &Value, signature: &Signature) -> Result<(), Error> {
        let canonical = canonical_json(json_data);
        let message_bytes = canonical.as_bytes();

        verify_signature(&public_key.to_bytes(), message_bytes, &signature.to_bytes())
            .map_err(|_| Error::Signature(SignatureError::VerificationFailed("JSON signature verification failed".to_string())))
    }

    /// Verify a signature against raw data
    pub fn verify(public_key: &VerifyingKey, message: &[u8], signature: &Signature) -> Result<(), Error> {
        verify_signature(&public_key.to_bytes(), message, &signature.to_bytes())
            .map_err(|_| Error::Signature(SignatureError::VerificationFailed("Raw signature verification failed".to_string())))
    }

    /// Sign raw bytes with a keypair
    pub fn sign_bytes(keypair: &Keypair, message: &[u8]) -> Signature {
        let cloned_keypair = keypair.clone();
        let signer = Ed25519Signer::new(cloned_keypair.inner);
        let sig_bytes = signer.sign(message);
        Signature::from_bytes(&sig_bytes)
    }

    /// Get private key bytes from keypair
    pub fn private_key_bytes(keypair: &Keypair) -> [u8; 32] {
        keypair.inner.to_bytes()
    }

    /// Create keypair from seed bytes
    pub fn keypair_from_seed(seed: &[u8; 32]) -> Result<Keypair, Error> {
        let signing_key = SigningKey::from_bytes(seed);
        Ok(Keypair::from(signing_key))
    }

    /// Hash data using SHA-256
    pub fn sha256(data: &[u8]) -> [u8; 32] {
        sha256(data)
    }

    /// Hash JSON data using SHA-256
    pub fn sha256_json(json_data: &Value) -> [u8; 32] {
        let canonical = canonical_json(json_data);
        sha256_bytes(canonical.as_bytes())
    }

    /// Get hex representation of public key
    pub fn public_key_hex(keypair: &Keypair) -> String {
        hex::encode(keypair.public.to_bytes())
    }

    /// Get hex representation of signature
    pub fn signature_hex(signature: &Signature) -> String {
        hex::encode(signature.to_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_keypair_from_hex() {
        let hex_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let keypair = Ed25519Helper::keypair_from_hex(hex_key).unwrap();

        // Should be able to get public key
        let pub_key_bytes = Ed25519Helper::public_key_bytes(&keypair);
        assert_eq!(pub_key_bytes.len(), 32);
    }

    #[test]
    fn test_sign_and_verify_json() {
        let hex_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let keypair = Ed25519Helper::keypair_from_hex(hex_key).unwrap();

        let json_data = json!({
            "test": "message",
            "amount": 1000
        });

        let signature = Ed25519Helper::sign_json(&keypair, &json_data);
        let result = Ed25519Helper::verify_json(&keypair.public, &json_data, &signature);

        assert!(result.is_ok());
    }

    #[test]
    fn test_deterministic_signing() {
        let hex_key = "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";
        let keypair = Ed25519Helper::keypair_from_hex(hex_key).unwrap();

        let json_data = json!({"test": "data"});

        let sig1 = Ed25519Helper::sign_json(&keypair, &json_data);
        let sig2 = Ed25519Helper::sign_json(&keypair, &json_data);

        assert_eq!(sig1.to_bytes(), sig2.to_bytes());
    }
}
