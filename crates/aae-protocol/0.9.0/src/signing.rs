//! Audit-event signatures (SPEC §3.5, v0.7).
//!
//! The signer signs the event's **attestation bytes** —
//! `canonicalize(view sans signature sans this_event_hash) || prev_event_hash`
//! — which are exactly the bytes an unsigned event would hash. The signature
//! is then attached and `this_event_hash` is computed over the view
//! *including* it. Two properties follow: the chain commits to the signature
//! (stripping it is hash-detectable without keys), and the signature commits
//! to the event's content *and* its chain position (prev hash is inside the
//! signed bytes).
//!
//! Consequence: signatures are applied at append time or never.
//!
//! v0.x permits exactly one algorithm, `ed25519` — deterministic, so signed
//! fixtures are byte-reproducible and vector-testable.

use crate::hashchain::{canonicalize, event_value_for_hashing, ChainError};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use ed25519_dalek::pkcs8::{DecodePrivateKey, DecodePublicKey};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

/// The only signature algorithm permitted in v0.x.
pub const SIGNATURE_ALG: &str = "ed25519";

/// Errors from signing and signature verification.
#[derive(Debug, thiserror::Error)]
pub enum SignatureError {
    /// The event carries no signature where one was required.
    #[error("event carries no signature (event_id={event_id})")]
    Missing {
        /// Event id of the offending event.
        event_id: String,
    },
    /// The signature declares an algorithm this verifier does not permit.
    #[error("signature algorithm {alg:?} not permitted in v0.x (ed25519 only)")]
    UnpermittedAlgorithm {
        /// The offending algorithm name.
        alg: String,
    },
    /// No trusted public key matches the signature's `key_id`.
    #[error("unknown signing key {key_id:?}")]
    UnknownKey {
        /// The unknown key id.
        key_id: String,
    },
    /// A key or signature value could not be parsed.
    #[error("malformed key or signature: {0}")]
    Malformed(String),
    /// A signature was present but did not verify (content, position, or the
    /// signature itself was altered).
    #[error("signature did not verify (event_id={event_id})")]
    Invalid {
        /// Event id of the offending event.
        event_id: String,
    },
    /// Canonicalization failed.
    #[error(transparent)]
    Chain(#[from] ChainError),
}

/// An Ed25519 event signer.
pub struct Ed25519EventSigner {
    key_id: String,
    key: SigningKey,
}

impl Ed25519EventSigner {
    /// Build from a PKCS#8 PEM private key.
    ///
    /// # Errors
    ///
    /// Returns [`SignatureError::Malformed`] if the PEM is not an Ed25519
    /// PKCS#8 private key.
    pub fn from_pem(key_id: &str, private_key_pem: &str) -> Result<Self, SignatureError> {
        let key = SigningKey::from_pkcs8_pem(private_key_pem)
            .map_err(|e| SignatureError::Malformed(e.to_string()))?;
        Ok(Self {
            key_id: key_id.to_string(),
            key,
        })
    }

    /// The key id this signer stamps into signatures.
    #[must_use]
    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    /// Sign raw attestation bytes.
    #[must_use]
    pub fn sign(&self, data: &[u8]) -> String {
        URL_SAFE_NO_PAD.encode(self.key.sign(data).to_bytes())
    }
}

/// The bytes a signature covers: `canonical(view sans signature sans
/// this_event_hash) || prev_event_hash`.
///
/// # Errors
///
/// Returns [`ChainError`] if the value cannot be canonicalized.
pub fn attestation_bytes(
    event: &serde_json::Value,
    prev_event_hash: Option<&str>,
) -> Result<Vec<u8>, ChainError> {
    let mut view = event_value_for_hashing(event);
    if let Some(obj) = view.as_object_mut() {
        obj.remove("signature");
    }
    let mut bytes = canonicalize(&view)?;
    bytes.extend(prev_event_hash.unwrap_or("").as_bytes());
    Ok(bytes)
}

/// Verify a single signed event against a `key_id -> PEM public key` map.
///
/// Returns `Ok(true)` on a valid signature and `Ok(false)` when a present
/// signature does not verify. Structural problems (missing signature, unknown
/// key, unpermitted algorithm) are *errors*, not `false`: operators must be
/// able to tell "forged" from "misconfigured".
///
/// # Errors
///
/// See [`SignatureError`].
pub fn verify_event_signature<S: std::hash::BuildHasher>(
    event: &serde_json::Value,
    trusted_keys: &std::collections::HashMap<String, String, S>,
) -> Result<bool, SignatureError> {
    let event_id = event
        .get("event_id")
        .and_then(|v| v.as_str())
        .unwrap_or("<unknown>")
        .to_string();
    let sig = event
        .get("signature")
        .filter(|v| !v.is_null())
        .ok_or_else(|| SignatureError::Missing {
            event_id: event_id.clone(),
        })?;

    let alg = sig.get("alg").and_then(|v| v.as_str()).unwrap_or("");
    if alg != SIGNATURE_ALG {
        return Err(SignatureError::UnpermittedAlgorithm {
            alg: alg.to_string(),
        });
    }
    let key_id = sig.get("key_id").and_then(|v| v.as_str()).unwrap_or("");
    let pem = trusted_keys
        .get(key_id)
        .ok_or_else(|| SignatureError::UnknownKey {
            key_id: key_id.to_string(),
        })?;
    let verifying = VerifyingKey::from_public_key_pem(pem)
        .map_err(|e| SignatureError::Malformed(e.to_string()))?;

    let raw = URL_SAFE_NO_PAD
        .decode(sig.get("value").and_then(|v| v.as_str()).unwrap_or(""))
        .map_err(|e| SignatureError::Malformed(e.to_string()))?;
    let signature =
        Signature::from_slice(&raw).map_err(|e| SignatureError::Malformed(e.to_string()))?;

    let prev = event.get("prev_event_hash").and_then(|v| v.as_str());
    let data = attestation_bytes(event, prev)?;
    Ok(verifying.verify(&data, &signature).is_ok())
}

/// Verify signatures across a chain. Returns how many events carried valid
/// signatures. Unsigned events are permitted unless `require_all`; a
/// present-but-invalid signature always errors.
///
/// # Errors
///
/// See [`SignatureError`].
pub fn verify_chain_signatures<S: std::hash::BuildHasher>(
    events: &[serde_json::Value],
    trusted_keys: &std::collections::HashMap<String, String, S>,
    require_all: bool,
) -> Result<usize, SignatureError> {
    let mut verified = 0;
    for ev in events {
        let has_sig = ev.get("signature").is_some_and(|v| !v.is_null());
        if !has_sig {
            if require_all {
                return Err(SignatureError::Missing {
                    event_id: ev
                        .get("event_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("<unknown>")
                        .to_string(),
                });
            }
            continue;
        }
        if !verify_event_signature(ev, trusted_keys)? {
            return Err(SignatureError::Invalid {
                event_id: ev
                    .get("event_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("<unknown>")
                    .to_string(),
            });
        }
        verified += 1;
    }
    Ok(verified)
}
