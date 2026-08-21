//! Test shims for signature verification (cfg(test) only)
#![cfg(test)]

use crate::generated::signatures::{InternalSignature, PartitionSignature};
use crate::errors::Error;

/// Test-only verification shim for InternalSignature that always returns true
/// This allows us to test signature set threshold logic without requiring real crypto
impl InternalSignature {
    pub fn verify_test(&self, _message: &[u8]) -> Result<bool, Error> {
        Ok(true)
    }
}

/// Test-only verification shim for PartitionSignature that always returns true
impl PartitionSignature {
    pub fn verify_test(&self, _message: &[u8]) -> Result<bool, Error> {
        Ok(true)
    }
}

/// Create a signature that will verify as true during tests
pub fn create_test_true_signature() -> crate::generated::signatures::Signature {
    use crate::generated::signatures::{Signature, InternalSignature};
    use hex;

    Signature::Internal(InternalSignature {
        cause: [0u8; 32],
        transaction_hash: [0u8; 32],
    })
}

/// Create a signature that will verify as false during tests (normal Ed25519 with zeros)
pub fn create_test_false_signature() -> crate::generated::signatures::Signature {
    use crate::generated::signatures::{Signature, ED25519Signature};

    Signature::ED25519(ED25519Signature {
        public_key: vec![0u8; 32],
        signature: vec![0u8; 64],
        signer: "acc://test.acme/signer".to_string(),
        signer_version: 1,
        timestamp: Some(1234567890),
        vote: None,
        transaction_hash: None,
        memo: None,
        data: None,
    })
}