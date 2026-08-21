// Allow unwrap/expect in this module - cryptographic operations have controlled inputs
#![allow(clippy::unwrap_used, clippy::expect_used)]

use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use sha2::{Digest, Sha256};

/// Ed25519 signer that exactly matches TypeScript SDK behavior
/// Updated for ed25519-dalek v2.x API
pub struct Ed25519Signer {
    signing_key: SigningKey,
}

impl std::fmt::Debug for Ed25519Signer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ed25519Signer")
            .field("public_key", &hex::encode(self.signing_key.verifying_key().as_bytes()))
            .finish_non_exhaustive()
    }
}

impl Ed25519Signer {
    pub fn new(signing_key: SigningKey) -> Self {
        Self { signing_key }
    }

    pub fn generate() -> Self {
        // Use a simple deterministic approach for now
        // In production, this should use proper random generation
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut seed = [0u8; 32];
        let bytes = nanos.to_le_bytes();
        seed[0..16].copy_from_slice(&bytes);
        seed[16..32].copy_from_slice(&bytes);
        Self::from_seed(&seed).expect("Failed to create keypair from seed")
    }

    /// Create signer from 32-byte seed (matches TS SDK)
    /// This is the primary method for deterministic keypair generation
    pub fn from_seed(seed: &[u8; 32]) -> Result<Self, ed25519_dalek::SignatureError> {
        // In ed25519-dalek v2.x, SigningKey is created directly from seed bytes
        let signing_key = SigningKey::from_bytes(seed);
        Ok(Self::new(signing_key))
    }

    /// Sign message and return 64-byte signature array (matches TS SDK)
    /// Returns [u8; 64] for exact byte-for-byte compatibility
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        let signature = self.signing_key.sign(message);
        signature.to_bytes()
    }

    /// Sign a pre-hashed message and return 64-byte signature array
    pub fn sign_prehashed(&self, hash: &[u8; 32]) -> [u8; 64] {
        let signature = self.signing_key.sign(hash);
        signature.to_bytes()
    }

    /// Get verifying (public) key as reference
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Get public key as 32-byte array (matches TS SDK)
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }

    /// Get private key (seed) as 32-byte array
    pub fn private_key_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    /// Create signer from 64-byte combined key (private + public)
    pub fn from_keypair_bytes(bytes: &[u8; 64]) -> Result<Self, ed25519_dalek::SignatureError> {
        let signing_key = SigningKey::from_keypair_bytes(bytes)?;
        Ok(Self::new(signing_key))
    }

    /// Get full keypair as 64-byte array (private + public)
    pub fn keypair_bytes(&self) -> [u8; 64] {
        self.signing_key.to_keypair_bytes()
    }
}

/// Verify Ed25519 signature (matches TS SDK)
/// Returns true if signature is valid, false otherwise
pub fn verify(
    public_key: &[u8; 32],
    message: &[u8],
    signature: &[u8; 64],
) -> bool {
    match VerifyingKey::from_bytes(public_key) {
        Ok(verifying_key) => {
            match Signature::from_bytes(signature) {
                sig => verifying_key.verify(message, &sig).is_ok(),
            }
        }
        Err(_) => false,
    }
}

/// Verify Ed25519 signature against pre-hashed message (matches TS SDK)
/// Returns true if signature is valid, false otherwise
pub fn verify_prehashed(
    public_key: &[u8; 32],
    hash: &[u8; 32],
    signature: &[u8; 64],
) -> bool {
    match VerifyingKey::from_bytes(public_key) {
        Ok(verifying_key) => {
            match Signature::from_bytes(signature) {
                sig => verifying_key.verify(hash, &sig).is_ok(),
            }
        }
        Err(_) => false,
    }
}

/// Legacy verify function for backwards compatibility
pub fn verify_signature(
    public_key: &[u8; 32],
    message: &[u8],
    signature: &[u8; 64],
) -> Result<(), ed25519_dalek::SignatureError> {
    let verifying_key = VerifyingKey::from_bytes(public_key)?;
    let sig = Signature::from_bytes(signature);
    verifying_key.verify(message, &sig)
}

/// Legacy verify function for backwards compatibility
pub fn verify_signature_prehashed(
    public_key: &[u8; 32],
    hash: &[u8; 32],
    signature: &[u8; 64],
) -> Result<(), ed25519_dalek::SignatureError> {
    let verifying_key = VerifyingKey::from_bytes(public_key)?;
    let sig = Signature::from_bytes(signature);
    verifying_key.verify(hash, &sig)
}

/// Hash message with SHA-256
pub fn sha256(message: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(message);
    hasher.finalize().into()
}

/// Hash message and return as hex string
pub fn sha256_hex(message: &[u8]) -> String {
    hex::encode(sha256(message))
}

/// Legacy alias for sha256
pub fn hash_message(message: &[u8]) -> [u8; 32] {
    sha256(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_sign() {
        let signer = Ed25519Signer::generate();
        let message = b"Hello, Accumulate!";
        let signature = signer.sign(message);

        let public_key_bytes = signer.public_key_bytes();

        assert!(verify(&public_key_bytes, message, &signature));
        assert!(verify_signature(&public_key_bytes, message, &signature).is_ok());
    }

    #[test]
    fn test_from_seed() {
        let seed = [42u8; 32];
        let signer = Ed25519Signer::from_seed(&seed).unwrap();
        let message = b"Deterministic test";
        let signature = signer.sign(message);

        let signer2 = Ed25519Signer::from_seed(&seed).unwrap();
        let signature2 = signer2.sign(message);

        assert_eq!(signature, signature2);
        assert_eq!(signer.public_key_bytes(), signer2.public_key_bytes());
    }

    #[test]
    fn test_sign_returns_64_bytes() {
        let seed = [1u8; 32];
        let signer = Ed25519Signer::from_seed(&seed).unwrap();
        let message = b"test message";
        let signature = signer.sign(message);

        assert_eq!(signature.len(), 64);
        assert!(verify(&signer.public_key_bytes(), message, &signature));
    }

    #[test]
    fn test_verify_functions() {
        let seed = [2u8; 32];
        let signer = Ed25519Signer::from_seed(&seed).unwrap();
        let message = b"test verification";
        let signature = signer.sign(message);
        let public_key = signer.public_key_bytes();

        // Test new verify function
        assert!(verify(&public_key, message, &signature));

        // Test with wrong message
        assert!(!verify(&public_key, b"wrong message", &signature));

        // Test with wrong signature
        let wrong_signature = [0u8; 64];
        assert!(!verify(&public_key, message, &wrong_signature));
    }
}
