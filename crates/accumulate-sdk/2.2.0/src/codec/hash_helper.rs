//! High-level hash helper utilities
//!
//! This module provides convenient wrapper functions around the core hashing implementation
//! to match the API expected by test files and provide TypeScript SDK compatibility.

use crate::codec::{canonical_json, sha256_bytes};
use serde_json::Value;

/// High-level hash helper providing convenient hashing operations
#[derive(Debug, Clone, Copy)]
pub struct HashHelper;

impl HashHelper {
    /// Compute SHA-256 hash of raw bytes
    pub fn sha256(data: &[u8]) -> [u8; 32] {
        sha256_bytes(data)
    }

    /// Compute SHA-256 hash of raw bytes and return as hex string
    pub fn sha256_hex(data: &[u8]) -> String {
        hex::encode(sha256_bytes(data))
    }

    /// Compute SHA-256 hash of JSON data
    pub fn sha256_json(json_data: &Value) -> [u8; 32] {
        let canonical = canonical_json(json_data);
        sha256_bytes(canonical.as_bytes())
    }

    /// Compute SHA-256 hash of JSON data and return as hex string
    pub fn sha256_json_hex(json_data: &Value) -> String {
        let canonical = canonical_json(json_data);
        hex::encode(sha256_bytes(canonical.as_bytes()))
    }

    /// Convert bytes to hex string
    pub fn bytes_to_hex(bytes: &[u8]) -> String {
        hex::encode(bytes)
    }

    /// Convert hex string to bytes
    pub fn hex_to_bytes(hex_str: &str) -> Result<Vec<u8>, hex::FromHexError> {
        hex::decode(hex_str)
    }

    /// Compute hash of string data
    pub fn hash_string(data: &str) -> [u8; 32] {
        sha256_bytes(data.as_bytes())
    }

    /// Compute hash of string data and return as hex
    pub fn hash_string_hex(data: &str) -> String {
        hex::encode(sha256_bytes(data.as_bytes()))
    }

    /// Create a double hash (hash of hash)
    pub fn double_hash(data: &[u8]) -> [u8; 32] {
        let first_hash = sha256_bytes(data);
        sha256_bytes(&first_hash)
    }

    /// Create a double hash and return as hex
    pub fn double_hash_hex(data: &[u8]) -> String {
        hex::encode(Self::double_hash(data))
    }

    /// Verify that two byte arrays are equal
    pub fn bytes_equal(a: &[u8], b: &[u8]) -> bool {
        a == b
    }

    /// Verify hash matches expected value
    pub fn verify_hash(data: &[u8], expected_hash: &[u8; 32]) -> bool {
        &sha256_bytes(data) == expected_hash
    }

    /// Verify hash matches expected hex value
    pub fn verify_hash_hex(data: &[u8], expected_hex: &str) -> bool {
        let computed_hash = hex::encode(sha256_bytes(data));
        computed_hash == expected_hex
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_sha256_basic() {
        let data = b"Hello, World!";
        let hash = HashHelper::sha256(data);
        let hash_hex = HashHelper::sha256_hex(data);

        assert_eq!(hash.len(), 32);
        assert_eq!(hash_hex.len(), 64);
        assert_eq!(hash_hex, hex::encode(hash));
    }

    #[test]
    fn test_json_hashing() {
        let json_data = json!({
            "test": "data",
            "number": 42
        });

        let hash = HashHelper::sha256_json(&json_data);
        let hash_hex = HashHelper::sha256_json_hex(&json_data);

        assert_eq!(hash.len(), 32);
        assert_eq!(hash_hex, hex::encode(hash));
    }

    #[test]
    fn test_deterministic_json_hashing() {
        let json1 = json!({"b": 2, "a": 1});
        let json2 = json!({"a": 1, "b": 2});

        let hash1 = HashHelper::sha256_json_hex(&json1);
        let hash2 = HashHelper::sha256_json_hex(&json2);

        // Should be the same due to canonical ordering
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_double_hash() {
        let data = b"test data";
        let single_hash = HashHelper::sha256(data);
        let double_hash = HashHelper::double_hash(data);

        // Double hash should be different from single hash
        assert_ne!(single_hash, double_hash);

        // Verify double hash is hash of hash
        let expected_double = HashHelper::sha256(&single_hash);
        assert_eq!(double_hash, expected_double);
    }

    #[test]
    fn test_hex_conversion() {
        let data = b"test";
        let hex = HashHelper::bytes_to_hex(data);
        let bytes_back = HashHelper::hex_to_bytes(&hex).unwrap();

        assert_eq!(data, &bytes_back[..]);
    }

    #[test]
    fn test_hash_verification() {
        let data = b"verification test";
        let hash = HashHelper::sha256(data);
        let hash_hex = HashHelper::sha256_hex(data);

        assert!(HashHelper::verify_hash(data, &hash));
        assert!(HashHelper::verify_hash_hex(data, &hash_hex));

        // Test with wrong data
        assert!(!HashHelper::verify_hash(b"wrong data", &hash));
        assert!(!HashHelper::verify_hash_hex(b"wrong data", &hash_hex));
    }
}