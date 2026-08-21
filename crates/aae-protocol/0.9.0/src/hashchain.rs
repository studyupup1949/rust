//! Hash chain helpers: canonicalization, chain construction, chain verification.

use crate::AuditEvent;
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Hash prefix for SHA-256 digests in this protocol.
pub const HASH_PREFIX: &str = "sha256:";

/// Errors raised by chain verification.
#[derive(Debug, Error)]
pub enum ChainError {
    /// Empty chain provided to verifier.
    #[error("empty chain")]
    EmptyChain,
    /// Genesis event has non-null `prev_event_hash`.
    #[error("genesis event must have prev_event_hash=None (event_id={event_id})")]
    InvalidGenesis {
        /// Event id of the offending event.
        event_id: String,
    },
    /// `prev_event_hash` on event N does not match `this_event_hash` on event N-1.
    #[error("prev_event_hash mismatch at event_id={event_id}: expected {expected:?}, got {got:?}")]
    PrevHashMismatch {
        /// Event id of the offending event.
        event_id: String,
        /// Expected `prev_event_hash` (i.e., previous event's `this_event_hash`).
        expected: Option<String>,
        /// Actual `prev_event_hash` on this event.
        got: Option<String>,
    },
    /// `this_event_hash` does not match recomputed value.
    #[error("this_event_hash does not match recomputed (event_id={event_id})")]
    HashMismatch {
        /// Event id of the offending event.
        event_id: String,
    },
    /// Serialization failure during canonicalization.
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Canonicalize a JSON value to RFC 8785 (JCS) bytes for hashing.
///
/// Uses `serde_jcs`, a full RFC 8785 implementation: lexicographic key ordering
/// over UTF-16 code units, ECMAScript number serialization, specified string
/// escaping. Byte-for-byte parity with the Python (`rfc8785`) and TypeScript
/// (`json-canonicalize`) SDKs is enforced by the shared vectors in
/// `reference-tests/test-vectors/canonicalization.json`.
///
/// # Errors
///
/// Returns `serde_json::Error` if the value cannot be serialized (e.g.,
/// non-finite floats, which JSON cannot represent).
pub fn canonicalize(value: &serde_json::Value) -> Result<Vec<u8>, serde_json::Error> {
    let mut out = Vec::new();
    write_canonical(value, &mut out)?;
    Ok(out)
}

/// String escaping and number formatting delegate to `serde_jcs`; object-key
/// ordering is implemented here because RFC 8785 requires lexicographic
/// order over **UTF-16 code units**, and `serde_jcs` (as of 0.1.x) sorts by
/// Unicode code point — the two disagree for astral-plane keys (surrogate
/// units 0xD800..0xDFFF sort before U+E000+ in UTF-16, after them in
/// code-point order). Caught by the `unicode_key_sort_utf16` shared vector.
fn write_canonical(value: &serde_json::Value, out: &mut Vec<u8>) -> Result<(), serde_json::Error> {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
            out.push(b'{');
            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                out.extend(serde_jcs::to_vec(key)?);
                out.push(b':');
                write_canonical(&map[*key], out)?;
            }
            out.push(b'}');
        }
        serde_json::Value::Array(items) => {
            out.push(b'[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                write_canonical(item, out)?;
            }
            out.push(b']');
        }
        primitive => out.extend(serde_jcs::to_vec(primitive)?),
    }
    Ok(())
}

/// Produce the hashing view of an event JSON value (normative; see the Python
/// SDK's `_event_to_canonical_dict`, the single point of truth):
///   - `this_event_hash` is excluded.
///   - `signature` is excluded when null (None-valued optional fields don't
///     pollute the hash).
///   - `prev_event_hash` is INCLUDED even when null (genesis marker is
///     structurally significant).
#[must_use]
pub fn event_value_for_hashing(event: &serde_json::Value) -> serde_json::Value {
    let mut v = event.clone();
    if let serde_json::Value::Object(map) = &mut v {
        map.remove("this_event_hash");
        if matches!(map.get("signature"), Some(serde_json::Value::Null)) {
            map.remove("signature");
        }
    }
    v
}

/// Compute `this_event_hash` for an event represented as a JSON value with
/// `this_event_hash` absent.
///
/// # Errors
///
/// Returns [`ChainError::Serde`] if the event value cannot be canonicalized.
pub fn compute_event_hash(
    event_without_hash: &serde_json::Value,
    prev_event_hash: Option<&str>,
) -> Result<String, ChainError> {
    let canonical = canonicalize(event_without_hash)?;
    let mut hasher = Sha256::new();
    hasher.update(&canonical);
    hasher.update(prev_event_hash.unwrap_or("").as_bytes());
    let digest = hasher.finalize();
    Ok(format!("{HASH_PREFIX}{digest:x}"))
}

/// Compute and assign `this_event_hash` for an event, given the previous chain tip.
///
/// Returns the event with `this_event_hash` and `prev_event_hash` set.
///
/// # Errors
///
/// Returns [`ChainError::Serde`] if the event cannot be serialized for hashing.
pub fn append_event(
    mut event: AuditEvent,
    prev_event_hash: Option<String>,
) -> Result<AuditEvent, ChainError> {
    event.prev_event_hash = prev_event_hash;
    let value = event_value_for_hashing(&serde_json::to_value(&event)?);
    let new_hash = compute_event_hash(&value, event.prev_event_hash.as_deref())?;
    event.this_event_hash = new_hash;
    Ok(event)
}

/// Verify a single event's `this_event_hash` matches its recomputed hash.
///
/// # Errors
///
/// Returns [`ChainError::Serde`] if the event cannot be serialized for hashing.
pub fn verify_event(event: &AuditEvent) -> Result<bool, ChainError> {
    let value = event_value_for_hashing(&serde_json::to_value(event)?);
    let expected = compute_event_hash(&value, event.prev_event_hash.as_deref())?;
    Ok(expected == event.this_event_hash)
}

/// Value-based chain production: seal a raw event JSON value onto the chain.
///
/// Sets `prev_event_hash` and `this_event_hash` on a copy of `event` and
/// returns it. Producers using this path control their serialization
/// byte-for-byte (C-3: no typed round-trip can alter timestamp precision).
///
/// # Errors
///
/// Returns [`ChainError::Serde`] if the value is not an object or cannot be
/// canonicalized.
pub fn append_event_value(
    event: &serde_json::Value,
    prev_event_hash: Option<&str>,
) -> Result<serde_json::Value, ChainError> {
    let sealed = seal_prev(event, prev_event_hash)?;
    finish_append(sealed, prev_event_hash)
}

/// Value-based chain production with an event SIGNATURE (v0.7 §3.5).
///
/// Signs the attestation bytes, attaches the signature, THEN hashes the view
/// including it — so the chain commits to the signature and stripping it is
/// hash-detectable without keys.
///
/// # Errors
///
/// Returns [`ChainError::Serde`] if the value is not an object or cannot be
/// canonicalized.
#[cfg(feature = "signing")]
pub fn append_event_value_signed(
    event: &serde_json::Value,
    prev_event_hash: Option<&str>,
    signer: &crate::signing::Ed25519EventSigner,
) -> Result<serde_json::Value, ChainError> {
    let mut sealed = seal_prev(event, prev_event_hash)?;
    let data = crate::signing::attestation_bytes(&sealed, prev_event_hash)?;
    let signature = serde_json::json!({
        "alg": crate::signing::SIGNATURE_ALG,
        "key_id": signer.key_id(),
        "value": signer.sign(&data),
    });
    if let Some(obj) = sealed.as_object_mut() {
        obj.insert("signature".to_string(), signature);
    }
    finish_append(sealed, prev_event_hash)
}

fn seal_prev(
    event: &serde_json::Value,
    prev_event_hash: Option<&str>,
) -> Result<serde_json::Value, ChainError> {
    let mut sealed = event.clone();
    let obj = sealed
        .as_object_mut()
        .ok_or_else(|| ChainError::Serde(serde::de::Error::custom("event is not a JSON object")))?;
    obj.insert(
        "prev_event_hash".to_string(),
        match prev_event_hash {
            Some(h) => serde_json::Value::String(h.to_string()),
            None => serde_json::Value::Null,
        },
    );
    Ok(sealed)
}

fn finish_append(
    mut sealed: serde_json::Value,
    prev_event_hash: Option<&str>,
) -> Result<serde_json::Value, ChainError> {
    let view = event_value_for_hashing(&sealed);
    let hash = compute_event_hash(&view, prev_event_hash)?;
    if let Some(obj) = sealed.as_object_mut() {
        obj.insert(
            "this_event_hash".to_string(),
            serde_json::Value::String(hash),
        );
    }
    Ok(sealed)
}

/// Verify a single event supplied as a raw JSON value.
///
/// Prefer this over [`verify_event`] when verifying chains produced by other
/// implementations: C-3 requires fractional-second precision in timestamps to
/// be preserved, and a typed round-trip (serde -> chrono -> serde) may
/// re-serialize `ts` with different precision than the producer used. Hashing
/// the received bytes' JSON value avoids that entirely.
///
/// # Errors
///
/// Returns [`ChainError::Serde`] if the value is not an object, lacks
/// `this_event_hash`, or cannot be canonicalized.
pub fn verify_event_value(event: &serde_json::Value) -> Result<bool, ChainError> {
    let obj = event
        .as_object()
        .ok_or_else(|| ChainError::Serde(serde::de::Error::custom("event is not a JSON object")))?;
    let this_hash = obj
        .get("this_event_hash")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ChainError::Serde(serde::de::Error::custom("missing this_event_hash")))?;
    let prev_hash = obj.get("prev_event_hash").and_then(|v| v.as_str());
    let value = event_value_for_hashing(event);
    let expected = compute_event_hash(&value, prev_hash)?;
    Ok(expected == this_hash)
}

/// Verify a chain of events supplied as raw JSON values. Returns the tip hash.
///
/// Same semantics as [`verify_chain`]; see [`verify_event_value`] for why the
/// value-based form is the right tool for cross-implementation verification.
///
/// # Errors
///
/// Returns the corresponding [`ChainError`] variant on an empty chain, a
/// non-null genesis `prev_event_hash`, a broken link, or a hash mismatch.
pub fn verify_chain_values(events: &[serde_json::Value]) -> Result<String, ChainError> {
    let get_id = |e: &serde_json::Value| -> String {
        e.get("event_id")
            .and_then(|v| v.as_str())
            .unwrap_or("<missing event_id>")
            .to_string()
    };

    let first = events.first().ok_or(ChainError::EmptyChain)?;
    if first.get("prev_event_hash").is_some_and(|v| !v.is_null()) {
        return Err(ChainError::InvalidGenesis {
            event_id: get_id(first),
        });
    }

    let mut prev: Option<String> = None;
    for ev in events {
        let ev_prev = ev
            .get("prev_event_hash")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        if ev_prev != prev {
            return Err(ChainError::PrevHashMismatch {
                event_id: get_id(ev),
                expected: prev.clone(),
                got: ev_prev,
            });
        }
        if !verify_event_value(ev)? {
            return Err(ChainError::HashMismatch {
                event_id: get_id(ev),
            });
        }
        prev = ev
            .get("this_event_hash")
            .and_then(|v| v.as_str())
            .map(str::to_string);
    }

    prev.ok_or(ChainError::EmptyChain)
}

/// Verify a sequence of events forms a valid hash chain.
///
/// Returns the tip hash on success.
///
/// # Errors
///
/// Returns the corresponding [`ChainError`] variant on an empty chain, a
/// non-null genesis `prev_event_hash`, a broken link, or a hash mismatch.
pub fn verify_chain(events: &[AuditEvent]) -> Result<String, ChainError> {
    let first = events.first().ok_or(ChainError::EmptyChain)?;
    if first.prev_event_hash.is_some() {
        return Err(ChainError::InvalidGenesis {
            event_id: first.event_id.clone(),
        });
    }

    let mut prev: Option<String> = None;
    for ev in events {
        if ev.prev_event_hash != prev {
            return Err(ChainError::PrevHashMismatch {
                event_id: ev.event_id.clone(),
                expected: prev.clone(),
                got: ev.prev_event_hash.clone(),
            });
        }
        if !verify_event(ev)? {
            return Err(ChainError::HashMismatch {
                event_id: ev.event_id.clone(),
            });
        }
        prev = Some(ev.this_event_hash.clone());
    }

    prev.ok_or(ChainError::EmptyChain)
}
