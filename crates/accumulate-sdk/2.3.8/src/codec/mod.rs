//! Codec utilities for Accumulate protocol compatibility
//!
//! This module provides both JSON and binary encoding to match the TypeScript SDK
//! implementation for bit-for-bit and byte-for-byte parity.

#![allow(missing_docs)]

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub mod canonical;
pub mod crypto;
pub mod hash_helper;
pub mod hashes;
pub mod reader;
pub mod signing;
pub mod transaction_codec;
pub mod writer;

pub use canonical::*;
// crypto module is kept for backwards compatibility but has no exports
// pub use crypto::*;
pub use hash_helper::*;
pub use hashes::*;
pub use reader::*;
pub use signing::*;
pub use transaction_codec::*;
pub use writer::*;

/// Convert a JSON value to canonical JSON string with deterministic ordering
/// This matches the TypeScript SDK implementation exactly
pub fn canonical_json(value: &Value) -> String {
    crate::canonjson::canonicalize(value)
}

/// Convert any serializable value to canonical JSON
/// Convenience wrapper around canonjson::dumps_canonical
pub fn to_canonical_string<T: serde::Serialize>(value: &T) -> String {
    crate::canonjson::dumps_canonical(value)
}

/// SHA-256 hash of raw bytes
pub fn sha256_bytes(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// SHA-256 hash of a JSON value via canonical JSON
pub fn sha256_hex(value: &Value) -> String {
    let canonical = canonical_json(value);
    let hash = sha256_bytes(canonical.as_bytes());
    hex::encode(hash)
}

/// Deterministic JSON object conversion ensuring sorted keys
pub fn canonicalize_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut btree: BTreeMap<String, Value> = BTreeMap::new();
            for (k, v) in map {
                btree.insert(k.clone(), canonicalize_value(v));
            }
            Value::Object(Map::from_iter(btree.into_iter()))
        }
        Value::Array(arr) => Value::Array(arr.iter().map(canonicalize_value).collect()),
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_canonical_json_simple() {
        let value = json!({ "z": 3, "a": 1, "m": 2 });
        let canonical = canonical_json(&value);
        assert_eq!(canonical, r#"{"a":1,"m":2,"z":3}"#);
    }

    #[test]
    fn test_canonical_json_nested() {
        let value = json!({
            "z": { "y": 2, "x": 1 },
            "a": 1
        });
        let canonical = canonical_json(&value);
        assert_eq!(canonical, r#"{"a":1,"z":{"x":1,"y":2}}"#);
    }

    #[test]
    fn test_canonical_json_array() {
        let value = json!({
            "arr": [{ "b": 2, "a": 1 }, { "d": 4, "c": 3 }]
        });
        let canonical = canonical_json(&value);
        assert_eq!(canonical, r#"{"arr":[{"a":1,"b":2},{"c":3,"d":4}]}"#);
    }

    #[test]
    fn test_canonical_json_primitives() {
        let value = json!({
            "string": "test",
            "number": 42,
            "boolean": true,
            "null": null
        });
        let canonical = canonical_json(&value);
        assert_eq!(
            canonical,
            r#"{"boolean":true,"null":null,"number":42,"string":"test"}"#
        );
    }

    #[test]
    fn test_sha256_consistency() {
        let value = json!({
            "header": {
                "principal": "acc://alice.acme/tokens",
                "timestamp": 1234567890123u64
            },
            "body": {
                "type": "send-tokens",
                "to": [{
                    "url": "acc://bob.acme/tokens",
                    "amount": "1000"
                }]
            }
        });

        let canonical = canonical_json(&value);
        let hash = sha256_hex(&value);

        // This should match the expected hash from our TS fixture
        let expected_canonical = r#"{"body":{"to":[{"amount":"1000","url":"acc://bob.acme/tokens"}],"type":"send-tokens"},"header":{"principal":"acc://alice.acme/tokens","timestamp":1234567890123}}"#;
        let expected_hash = "4be49c59c717f1984646998cecac0e5225378d9bbe2e18928272a85b7dfcb608";

        assert_eq!(canonical, expected_canonical);
        assert_eq!(hash, expected_hash);
    }
}
