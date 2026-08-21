//! AAP cryptographic primitives — Ed25519 + SHA-256.
//! Uses ed25519-dalek (audited) and sha2.

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey, Verifier, Signature};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use serde_json::Value;

use crate::errors::{AAPError, Result};

/// An Ed25519 keypair for signing AAP documents.
pub struct KeyPair {
    signing: SigningKey,
}

impl KeyPair {
    /// Generate a new random Ed25519 keypair.
    pub fn generate() -> Self {
        Self { signing: SigningKey::generate(&mut OsRng) }
    }

    /// Return the public key in AAP wire format: `"ed25519:<base64>"`.
    pub fn public_key_b64(&self) -> String {
        format!("ed25519:{}", B64.encode(self.signing.verifying_key().as_bytes()))
    }

    /// Sign `data`, returning the signature in AAP wire format: `"ed25519:<base64>"`.
    pub fn sign(&self, data: &[u8]) -> String {
        let sig: Signature = self.signing.sign(data);
        format!("ed25519:{}", B64.encode(sig.to_bytes()))
    }
}

/// Verify an Ed25519 AAP signature.
pub fn verify_signature(public_key_b64: &str, data: &[u8], signature_b64: &str) -> Result<()> {
    let pub_bytes = B64
        .decode(public_key_b64.trim_start_matches("ed25519:"))
        .map_err(|e| AAPError::Signature(format!("invalid public key: {e}")))?;

    let sig_bytes = B64
        .decode(signature_b64.trim_start_matches("ed25519:"))
        .map_err(|e| AAPError::Signature(format!("invalid signature: {e}")))?;

    let key = VerifyingKey::from_bytes(
        pub_bytes.as_slice().try_into()
            .map_err(|_| AAPError::Signature("public key must be 32 bytes".into()))?,
    ).map_err(|e| AAPError::Signature(format!("invalid key: {e}")))?;

    let sig = Signature::from_bytes(
        sig_bytes.as_slice().try_into()
            .map_err(|_| AAPError::Signature("signature must be 64 bytes".into()))?,
    );

    key.verify(data, &sig)
        .map_err(|_| AAPError::Signature("signature mismatch".into()))
}

/// Compute `"sha256:<hex>"` of `data`.
pub fn sha256_of(data: &[u8]) -> String {
    let hash = Sha256::digest(data);
    format!("sha256:{}", hex::encode(hash))
}

/// Compute the SHA-256 of the canonical JSON of a serde_json Value.
pub fn hash_entry(v: &Value) -> String {
    let canonical = serde_json::to_string(v).unwrap_or_default();
    sha256_of(canonical.as_bytes())
}

/// Return canonical bytes for signing: sorted-key JSON without the "signature" field.
pub fn signable(v: &Value) -> Result<Vec<u8>> {
    let mut map = match v.as_object() {
        Some(m) => m.clone(),
        None => return Err(AAPError::Serde(serde_json::from_str::<Value>("").unwrap_err())),
    };
    map.remove("signature");
    // Sort keys for canonical representation
    let sorted: serde_json::Map<String, Value> = map.into_iter().collect();
    Ok(serde_json::to_vec(&Value::Object(sorted))?)
}
