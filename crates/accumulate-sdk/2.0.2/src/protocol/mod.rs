//! Accumulate protocol structures and envelope encoding
//!
//! This module provides transaction envelope building and serialization
//! that matches the TypeScript SDK implementation exactly.

#![allow(missing_docs)]

use crate::codec::{canonical_json, sha256_bytes};
use crate::crypto::ed25519_helper::{Ed25519Helper, Keypair};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

pub mod envelope;
pub mod transaction;

// Re-export envelope and transaction modules (currently empty)
// pub use envelope::*;
// pub use transaction::*;

/// Transaction envelope containing transaction and signatures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionEnvelope {
    pub signatures: Vec<TransactionSignature>,
    pub transaction: Vec<Transaction>,
}

/// Transaction signature for Accumulate protocol
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionSignature {
    #[serde(rename = "type")]
    pub signature_type: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub signature: String,
    pub signer: String,
    #[serde(rename = "signerVersion")]
    pub signer_version: u64,
    pub timestamp: u64,
    #[serde(rename = "transactionHash")]
    pub transaction_hash: String,
}

/// Transaction structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub header: TransactionHeader,
    pub body: Value,
}

/// Transaction header
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionHeader {
    pub principal: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initiator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
}

/// Envelope builder for creating transaction envelopes
#[derive(Debug, Clone, Copy)]
pub struct EnvelopeBuilder;

impl EnvelopeBuilder {
    /// Create a new transaction envelope with signature
    pub fn create_envelope(
        transaction: Transaction,
        keypair: &Keypair,
        signer_url: &str,
        signer_version: u64,
    ) -> Result<TransactionEnvelope, EnvelopeError> {
        // Serialize transaction to canonical JSON
        let tx_value = serde_json::to_value(&transaction)?;
        let canonical = canonical_json(&tx_value);

        // Hash the canonical transaction
        let tx_hash = sha256_bytes(canonical.as_bytes());
        let tx_hash_hex = hex::encode(tx_hash);

        // Sign the transaction hash
        let signature = Ed25519Helper::sign_bytes(keypair, &tx_hash);

        // Get current timestamp in microseconds
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| EnvelopeError::TimestampError(e.to_string()))?
            .as_micros() as u64;

        // Create signature object
        let tx_signature = TransactionSignature {
            signature_type: "ed25519".to_string(),
            public_key: hex::encode(Ed25519Helper::public_key_bytes(keypair)),
            signature: hex::encode(signature.to_bytes()),
            signer: signer_url.to_string(),
            signer_version,
            timestamp,
            transaction_hash: tx_hash_hex,
        };

        Ok(TransactionEnvelope {
            signatures: vec![tx_signature],
            transaction: vec![transaction],
        })
    }

    /// Create envelope from JSON transaction body
    pub fn create_envelope_from_json(
        principal: &str,
        body: Value,
        keypair: &Keypair,
        signer_url: &str,
        signer_version: u64,
    ) -> Result<TransactionEnvelope, EnvelopeError> {
        let header = TransactionHeader {
            principal: principal.to_string(),
            initiator: None,
            timestamp: None,
        };

        let transaction = Transaction { header, body };

        Self::create_envelope(transaction, keypair, signer_url, signer_version)
    }

    /// Create envelope with initiator
    pub fn create_envelope_with_initiator(
        principal: &str,
        initiator: &str,
        body: Value,
        keypair: &Keypair,
        signer_url: &str,
        signer_version: u64,
    ) -> Result<TransactionEnvelope, EnvelopeError> {
        let header = TransactionHeader {
            principal: principal.to_string(),
            initiator: Some(initiator.to_string()),
            timestamp: None,
        };

        let transaction = Transaction { header, body };

        Self::create_envelope(transaction, keypair, signer_url, signer_version)
    }

    /// Serialize envelope to canonical JSON
    pub fn serialize_envelope(envelope: &TransactionEnvelope) -> Result<String, EnvelopeError> {
        let value = serde_json::to_value(envelope)?;
        Ok(canonical_json(&value))
    }

    /// Verify envelope signature
    pub fn verify_envelope(envelope: &TransactionEnvelope) -> Result<(), EnvelopeError> {
        if envelope.signatures.is_empty() || envelope.transaction.is_empty() {
            return Err(EnvelopeError::InvalidEnvelope(
                "Missing signatures or transactions".to_string(),
            ));
        }

        let transaction = &envelope.transaction[0];
        let signature = &envelope.signatures[0];

        // Recreate transaction hash
        let tx_value = serde_json::to_value(transaction)?;
        let canonical = canonical_json(&tx_value);
        let computed_hash = hex::encode(sha256_bytes(canonical.as_bytes()));

        // Verify hash matches
        if computed_hash != signature.transaction_hash {
            return Err(EnvelopeError::HashMismatch {
                expected: signature.transaction_hash.clone(),
                computed: computed_hash,
            });
        }

        // Verify signature
        let public_key_bytes = hex::decode(&signature.public_key)
            .map_err(|e| EnvelopeError::InvalidSignature(e.to_string()))?;
        let signature_bytes = hex::decode(&signature.signature)
            .map_err(|e| EnvelopeError::InvalidSignature(e.to_string()))?;

        if public_key_bytes.len() != 32 || signature_bytes.len() != 64 {
            return Err(EnvelopeError::InvalidSignature(
                "Invalid key or signature length".to_string(),
            ));
        }

        let mut pk_array = [0u8; 32];
        let mut sig_array = [0u8; 64];
        pk_array.copy_from_slice(&public_key_bytes);
        sig_array.copy_from_slice(&signature_bytes);

        let public_key = Ed25519Helper::public_key_from_bytes(&pk_array)
            .map_err(|e| EnvelopeError::InvalidSignature(e.to_string()))?;
        let signature_obj = Ed25519Helper::signature_from_bytes(&sig_array)
            .map_err(|e| EnvelopeError::InvalidSignature(e.to_string()))?;

        let tx_hash_bytes = hex::decode(&signature.transaction_hash)
            .map_err(|e| EnvelopeError::InvalidSignature(e.to_string()))?;

        Ed25519Helper::verify(&public_key, &tx_hash_bytes, &signature_obj)
            .map_err(|e| EnvelopeError::VerificationFailed(e.to_string()))?;

        Ok(())
    }
}

/// Envelope-related errors
#[derive(Debug, thiserror::Error)]
pub enum EnvelopeError {
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Timestamp error: {0}")]
    TimestampError(String),

    #[error("Invalid envelope: {0}")]
    InvalidEnvelope(String),

    #[error("Hash mismatch: expected {expected}, computed {computed}")]
    HashMismatch { expected: String, computed: String },

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Signature verification failed: {0}")]
    VerificationFailed(String),
}

/// Helper functions for common transaction types
pub mod helpers {
    use super::*;
    use serde_json::json;

    /// Create send tokens transaction body
    pub fn create_send_tokens_body(to_url: &str, amount: &str, _token_url: Option<&str>) -> Value {
        json!({
            "type": "sendTokens",
            "to": [{
                "url": to_url,
                "amount": amount
            }]
        })
    }

    /// Create create identity transaction body
    pub fn create_identity_body(url: &str, public_key_hash: &str) -> Value {
        json!({
            "type": "createIdentity",
            "url": url,
            "keyBook": {
                "publicKeyHash": public_key_hash
            }
        })
    }

    /// Create add credits transaction body
    pub fn create_add_credits_body(recipient: &str, amount: u64, oracle: Option<&str>) -> Value {
        json!({
            "type": "addCredits",
            "recipient": recipient,
            "amount": amount,
            "oracle": oracle.unwrap_or("")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_envelope_creation() {
        let hex_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let keypair = Ed25519Helper::keypair_from_hex(hex_key).unwrap();

        let body = helpers::create_send_tokens_body("acc://bob.acme/tokens", "1000", None);

        let envelope = EnvelopeBuilder::create_envelope_from_json(
            "acc://alice.acme/tokens",
            body,
            &keypair,
            "acc://alice.acme/book/1",
            1,
        )
        .unwrap();

        assert_eq!(envelope.signatures.len(), 1);
        assert_eq!(envelope.transaction.len(), 1);
        assert_eq!(envelope.signatures[0].signature_type, "ed25519");
        assert_eq!(envelope.transaction[0].header.principal, "acc://alice.acme/tokens");
    }

    #[test]
    fn test_envelope_serialization() {
        let hex_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let keypair = Ed25519Helper::keypair_from_hex(hex_key).unwrap();

        let body = helpers::create_send_tokens_body("acc://bob.acme/tokens", "1000", None);

        let envelope = EnvelopeBuilder::create_envelope_from_json(
            "acc://alice.acme/tokens",
            body,
            &keypair,
            "acc://alice.acme/book/1",
            1,
        )
        .unwrap();

        let serialized = EnvelopeBuilder::serialize_envelope(&envelope).unwrap();
        assert!(serialized.contains("signatures"));
        assert!(serialized.contains("transaction"));
        assert!(serialized.contains("ed25519"));
    }

    #[test]
    fn test_envelope_verification() {
        let hex_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let keypair = Ed25519Helper::keypair_from_hex(hex_key).unwrap();

        let body = helpers::create_send_tokens_body("acc://bob.acme/tokens", "1000", None);

        let envelope = EnvelopeBuilder::create_envelope_from_json(
            "acc://alice.acme/tokens",
            body,
            &keypair,
            "acc://alice.acme/book/1",
            1,
        )
        .unwrap();

        let result = EnvelopeBuilder::verify_envelope(&envelope);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transaction_helpers() {
        let send_body = helpers::create_send_tokens_body("acc://recipient", "500", None);
        assert_eq!(send_body["type"], "sendTokens");

        let identity_body = helpers::create_identity_body("acc://new-identity", "pubkey123");
        assert_eq!(identity_body["type"], "createIdentity");

        let credits_body = helpers::create_add_credits_body("acc://recipient", 1000, None);
        assert_eq!(credits_body["type"], "addCredits");
    }
}