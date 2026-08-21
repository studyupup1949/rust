//! Runtime helpers for signatures (validation beyond serde shape).
use crate::generated::signatures::Signature;
use crate::generated::signatures::{DelegatedSignature, SignatureSet};

#[derive(thiserror::Error, Debug)]
pub enum SigRuntimeError {
    #[error("delegation depth exceeded (max 5)")]
    DelegationDepthExceeded,
    #[error("invalid signature set threshold")]
    InvalidSignatureSetThreshold,
    #[error("verification failed")]
    VerificationFailed,
}

/// Calculate delegated depth:
/// - `Signature::Delegated(inner)` increases depth by 1 and recurses.
/// - Other signatures are leaves (depth 0).
pub fn delegated_depth(sig: &Signature) -> usize {
    match sig {
        Signature::Delegated(inner) => 1 + delegated_depth(&inner.signature),
        _ => 0,
    }
}

/// Enforce max depth (≤ 5). Returns `Ok(())` if within limit.
pub fn enforce_delegated_depth(sig: &Signature) -> Result<(), SigRuntimeError> {
    if delegated_depth(sig) > 5 {
        Err(SigRuntimeError::DelegationDepthExceeded)
    } else {
        Ok(())
    }
}

impl DelegatedSignature {
    /// Smart constructor that enforces depth limit when wrapped in Signature::Delegated.
    pub fn new_enforced(signature: Box<Signature>, delegator: String) -> Result<Self, SigRuntimeError> {
        let wrapper = Signature::Delegated(DelegatedSignature {
            signature: signature.clone(),
            delegator: delegator.clone()
        });
        enforce_delegated_depth(&wrapper)?;
        Ok(DelegatedSignature { signature, delegator })
    }
}

/// Extended SignatureSet with threshold for runtime validation
/// Since the generated SignatureSet doesn't have a threshold field in the YAML,
/// we'll simulate threshold semantics by treating all signatures as requiring consensus
#[derive(Debug, Clone)]
pub struct SignatureSetWithThreshold {
    pub inner: SignatureSet,
    pub threshold: u32,
}

impl SignatureSetWithThreshold {
    pub fn new(inner: SignatureSet, threshold: u32) -> Result<Self, SigRuntimeError> {
        if threshold == 0 || (threshold as usize) > inner.signatures.len() {
            return Err(SigRuntimeError::InvalidSignatureSetThreshold);
        }
        Ok(Self { inner, threshold })
    }
}

/// Count signatures that verify `true`. If a signature cannot be verified due to an error,
/// treat as `false` (verification errors are counted as invalid).
pub fn count_valid_sigs(sigs: &[Box<Signature>], message: &[u8]) -> usize {
    sigs.iter()
        .filter(|s| {
            // Use the AccSignature trait for verification
            use crate::generated::signatures::AccSignature;
            match s.as_ref() {
                Signature::ED25519(sig) => sig.verify(message).unwrap_or(false),
                Signature::LegacyED25519(sig) => sig.verify(message).unwrap_or(false),
                Signature::RCD1(sig) => sig.verify(message).unwrap_or(false),
                Signature::BTC(sig) => sig.verify(message).unwrap_or(false),
                Signature::BTCLegacy(sig) => sig.verify(message).unwrap_or(false),
                Signature::ETH(sig) => sig.verify(message).unwrap_or(false),
                Signature::RsaSha256(sig) => sig.verify(message).unwrap_or(false),
                Signature::EcdsaSha256(sig) => sig.verify(message).unwrap_or(false),
                Signature::TypedData(sig) => sig.verify(message).unwrap_or(false),
                Signature::Receipt(sig) => sig.verify(message).unwrap_or(false),
                Signature::Partition(sig) => sig.verify(message).unwrap_or(false),
                Signature::Set(sig) => sig.verify(message).unwrap_or(false),
                Signature::Remote(sig) => sig.verify(message).unwrap_or(false),
                Signature::Delegated(sig) => sig.verify(message).unwrap_or(false),
                Signature::Internal(sig) => sig.verify(message).unwrap_or(false),
                Signature::Authority(sig) => sig.verify(message).unwrap_or(false),
            }
        })
        .count()
}

/// Validate threshold invariants and evaluate.
/// - threshold must be >=1 and <= signatures.len()
/// - returns Ok(true) iff valid_count >= threshold
pub fn evaluate_signature_set(set: &SignatureSetWithThreshold, message: &[u8]) -> Result<bool, SigRuntimeError> {
    let n = set.inner.signatures.len();
    if set.threshold == 0 || (set.threshold as usize) > n {
        return Err(SigRuntimeError::InvalidSignatureSetThreshold);
    }
    let valid = count_valid_sigs(&set.inner.signatures, message) as u32;
    Ok(valid >= set.threshold)
}