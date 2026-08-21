//! Hash utilities that exactly match TypeScript SDK implementation
//!
//! This module provides SHA-256 hashing utilities with byte-for-byte compatibility
//! with the TypeScript SDK for deterministic transaction and data hashing.

use super::{canonical_json, BinaryWriter, EncodingError};
use serde_json::Value;
use sha2::{Digest, Sha256};

/// Hash types used in Accumulate protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashType {
    /// SHA-256 hash of raw bytes
    Sha256,
    /// SHA-256 hash of canonical JSON
    Sha256Json,
    /// SHA-256 hash of binary-encoded data
    Sha256Binary,
}

/// Hash utilities that match TypeScript SDK exactly
#[derive(Debug, Clone, Copy)]
pub struct AccumulateHash;

impl AccumulateHash {
    /// SHA-256 hash of raw bytes
    /// Matches TS: hash(data: Uint8Array): Uint8Array
    pub fn sha256_bytes(data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }

    /// SHA-256 hash of raw bytes, returning hex string
    /// Matches TS: hash(data: Uint8Array): string (when hex output requested)
    pub fn sha256_bytes_hex(data: &[u8]) -> String {
        let hash = Self::sha256_bytes(data);
        hex::encode(hash)
    }

    /// SHA-256 hash of canonical JSON
    /// Matches TS: hashJson(obj: any): Uint8Array
    pub fn sha256_json(value: &Value) -> [u8; 32] {
        let canonical = canonical_json(value);
        Self::sha256_bytes(canonical.as_bytes())
    }

    /// SHA-256 hash of canonical JSON, returning hex string
    /// Matches TS: hashJson(obj: any): string (when hex output requested)
    pub fn sha256_json_hex(value: &Value) -> String {
        let hash = Self::sha256_json(value);
        hex::encode(hash)
    }

    /// Hash a string as UTF-8 bytes
    /// Matches TS: hashString(str: string): Uint8Array
    pub fn sha256_string(text: &str) -> [u8; 32] {
        Self::sha256_bytes(text.as_bytes())
    }

    /// Hash a string as UTF-8 bytes, returning hex string
    /// Matches TS: hashString(str: string): string (when hex output requested)
    pub fn sha256_string_hex(text: &str) -> String {
        let hash = Self::sha256_string(text);
        hex::encode(hash)
    }

    /// Hash multiple byte arrays concatenated
    /// Matches TS: hashConcat(...arrays: Uint8Array[]): Uint8Array
    pub fn sha256_concat(arrays: &[&[u8]]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        for array in arrays {
            hasher.update(array);
        }
        hasher.finalize().into()
    }

    /// Hash multiple byte arrays concatenated, returning hex string
    /// Matches TS: hashConcat(...arrays: Uint8Array[]): string (when hex output requested)
    pub fn sha256_concat_hex(arrays: &[&[u8]]) -> String {
        let hash = Self::sha256_concat(arrays);
        hex::encode(hash)
    }

    /// Double SHA-256 hash (hash of hash)
    /// Matches TS: doubleHash(data: Uint8Array): Uint8Array
    pub fn double_sha256(data: &[u8]) -> [u8; 32] {
        let first_hash = Self::sha256_bytes(data);
        Self::sha256_bytes(&first_hash)
    }

    /// Double SHA-256 hash, returning hex string
    /// Matches TS: doubleHash(data: Uint8Array): string (when hex output requested)
    pub fn double_sha256_hex(data: &[u8]) -> String {
        let hash = Self::double_sha256(data);
        hex::encode(hash)
    }

    /// Hash using binary encoding first
    /// Matches TS: hashBinaryEncoded(value: any, field?: number): Uint8Array
    pub fn sha256_binary_encoded<T>(value: T, field: Option<u32>) -> Result<[u8; 32], EncodingError>
    where
        T: BinaryEncodable,
    {
        let binary_data = value.encode_binary()?;
        let data = if let Some(field_num) = field {
            BinaryWriter::with_field_number(&binary_data, Some(field_num))?
        } else {
            binary_data
        };
        Ok(Self::sha256_bytes(&data))
    }

    /// Hash using binary encoding first, returning hex string
    /// Matches TS: hashBinaryEncoded(value: any, field?: number): string (when hex output requested)
    pub fn sha256_binary_encoded_hex<T>(
        value: T,
        field: Option<u32>,
    ) -> Result<String, EncodingError>
    where
        T: BinaryEncodable,
    {
        let hash = Self::sha256_binary_encoded(value, field)?;
        Ok(hex::encode(hash))
    }

    /// Hash a transaction using canonical JSON (matches TS SDK exactly)
    /// This is the primary method for transaction hashing in Accumulate
    pub fn hash_transaction(transaction: &Value) -> [u8; 32] {
        Self::sha256_json(transaction)
    }

    /// Hash a transaction using canonical JSON, returning hex string
    pub fn hash_transaction_hex(transaction: &Value) -> String {
        let hash = Self::hash_transaction(transaction);
        hex::encode(hash)
    }
}

/// Trait for types that can be binary encoded for hashing
pub trait BinaryEncodable {
    fn encode_binary(&self) -> Result<Vec<u8>, EncodingError>;
}

/// Implementation for basic types
impl BinaryEncodable for u64 {
    fn encode_binary(&self) -> Result<Vec<u8>, EncodingError> {
        Ok(BinaryWriter::encode_uvarint(*self))
    }
}

impl BinaryEncodable for i64 {
    fn encode_binary(&self) -> Result<Vec<u8>, EncodingError> {
        Ok(BinaryWriter::encode_varint(*self))
    }
}

impl BinaryEncodable for String {
    fn encode_binary(&self) -> Result<Vec<u8>, EncodingError> {
        Ok(BinaryWriter::encode_string(self))
    }
}

impl BinaryEncodable for &str {
    fn encode_binary(&self) -> Result<Vec<u8>, EncodingError> {
        Ok(BinaryWriter::encode_string(self))
    }
}

impl BinaryEncodable for Vec<u8> {
    fn encode_binary(&self) -> Result<Vec<u8>, EncodingError> {
        Ok(BinaryWriter::encode_bytes(self))
    }
}

impl BinaryEncodable for &[u8] {
    fn encode_binary(&self) -> Result<Vec<u8>, EncodingError> {
        Ok(BinaryWriter::encode_bytes(self))
    }
}

impl BinaryEncodable for bool {
    fn encode_binary(&self) -> Result<Vec<u8>, EncodingError> {
        Ok(BinaryWriter::encode_bool(*self))
    }
}

impl BinaryEncodable for [u8; 32] {
    fn encode_binary(&self) -> Result<Vec<u8>, EncodingError> {
        Ok(BinaryWriter::encode_hash(self))
    }
}

/// URL hashing utilities that match TypeScript SDK
#[derive(Debug, Clone, Copy)]
pub struct UrlHash;

impl UrlHash {
    /// Hash an Accumulate URL for identity derivation
    /// Matches TS: hashUrl(url: string): Uint8Array
    pub fn hash_url(url: &str) -> [u8; 32] {
        // TypeScript SDK uses specific URL normalization before hashing
        let normalized = Self::normalize_url(url);
        AccumulateHash::sha256_string(&normalized)
    }

    /// Hash an Accumulate URL, returning hex string
    /// Matches TS: hashUrl(url: string): string (when hex output requested)
    pub fn hash_url_hex(url: &str) -> String {
        let hash = Self::hash_url(url);
        hex::encode(hash)
    }

    /// Normalize Accumulate URL for consistent hashing
    /// Matches TS URL normalization rules
    pub fn normalize_url(url: &str) -> String {
        let mut normalized = url.to_lowercase();

        // Remove trailing slashes
        while normalized.ends_with('/') {
            normalized.pop();
        }

        // Ensure acc:// prefix
        if !normalized.starts_with("acc://") {
            if normalized.starts_with("//") {
                normalized = format!("acc:{}", normalized);
            } else if normalized.starts_with("/") {
                normalized = format!("acc:/{}", normalized);
            } else {
                normalized = format!("acc://{}", normalized);
            }
        }

        normalized
    }

    /// Derive key book URL from identity URL
    /// Matches TS: deriveKeyBookUrl(identityUrl: string): string
    pub fn derive_key_book_url(identity_url: &str) -> String {
        let normalized = Self::normalize_url(identity_url);
        format!("{}/book", normalized)
    }

    /// Derive key page URL from key book URL and page index
    /// Matches TS: deriveKeyPageUrl(keyBookUrl: string, pageIndex: number): string
    pub fn derive_key_page_url(key_book_url: &str, page_index: u32) -> String {
        let normalized = Self::normalize_url(key_book_url);
        format!("{}/{}", normalized, page_index)
    }

    /// Extract authority from Accumulate URL
    /// Matches TS: extractAuthority(url: string): string
    pub fn extract_authority(url: &str) -> Option<String> {
        let normalized = Self::normalize_url(url);

        if let Some(authority_start) = normalized.find("://") {
            let after_protocol = &normalized[authority_start + 3..];
            if let Some(path_start) = after_protocol.find('/') {
                Some(after_protocol[..path_start].to_string())
            } else {
                Some(after_protocol.to_string())
            }
        } else {
            None
        }
    }

    /// Extract path from Accumulate URL
    /// Matches TS: extractPath(url: string): string
    pub fn extract_path(url: &str) -> String {
        let normalized = Self::normalize_url(url);

        if let Some(authority_start) = normalized.find("://") {
            let after_protocol = &normalized[authority_start + 3..];
            if let Some(path_start) = after_protocol.find('/') {
                after_protocol[path_start..].to_string()
            } else {
                "/".to_string()
            }
        } else {
            "/".to_string()
        }
    }
}

/// Chain ID hashing utilities
#[derive(Debug, Clone, Copy)]
pub struct ChainHash;

impl ChainHash {
    /// Hash chain ID from URL
    /// Matches TS: hashChainId(url: string): Uint8Array
    pub fn hash_chain_id(url: &str) -> [u8; 32] {
        let normalized = UrlHash::normalize_url(url);
        AccumulateHash::sha256_string(&normalized)
    }

    /// Hash chain ID from URL, returning hex string
    /// Matches TS: hashChainId(url: string): string (when hex output requested)
    pub fn hash_chain_id_hex(url: &str) -> String {
        let hash = Self::hash_chain_id(url);
        hex::encode(hash)
    }

    /// Derive main chain ID from authority
    /// Matches TS: deriveMainChainId(authority: string): Uint8Array
    pub fn derive_main_chain_id(authority: &str) -> [u8; 32] {
        let main_url = format!("acc://{}", authority.to_lowercase());
        Self::hash_chain_id(&main_url)
    }

    /// Derive main chain ID from authority, returning hex string
    /// Matches TS: deriveMainChainId(authority: string): string (when hex output requested)
    pub fn derive_main_chain_id_hex(authority: &str) -> String {
        let hash = Self::derive_main_chain_id(authority);
        hex::encode(hash)
    }
}

/// Merkle tree utilities for transaction hashing
#[derive(Debug, Clone, Copy)]
pub struct MerkleHash;

impl MerkleHash {
    /// Build Merkle root from list of hashes
    /// Matches TS: buildMerkleRoot(hashes: Uint8Array[]): Uint8Array
    pub fn build_merkle_root(hashes: &[[u8; 32]]) -> [u8; 32] {
        if hashes.is_empty() {
            return [0u8; 32];
        }

        if hashes.len() == 1 {
            return hashes[0];
        }

        let mut current_level: Vec<[u8; 32]> = hashes.to_vec();

        while current_level.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in current_level.chunks(2) {
                let combined_hash = if chunk.len() == 2 {
                    // Hash of concatenated pair
                    AccumulateHash::sha256_concat(&[&chunk[0], &chunk[1]])
                } else {
                    // Odd number, hash single element with itself
                    AccumulateHash::sha256_concat(&[&chunk[0], &chunk[0]])
                };
                next_level.push(combined_hash);
            }

            current_level = next_level;
        }

        current_level[0]
    }

    /// Build Merkle root from list of hashes, returning hex string
    /// Matches TS: buildMerkleRoot(hashes: Uint8Array[]): string (when hex output requested)
    pub fn build_merkle_root_hex(hashes: &[[u8; 32]]) -> String {
        let root = Self::build_merkle_root(hashes);
        hex::encode(root)
    }

    /// Create Merkle proof for element at index
    /// Matches TS: createMerkleProof(hashes: Uint8Array[], index: number): Uint8Array[]
    pub fn create_merkle_proof(hashes: &[[u8; 32]], index: usize) -> Vec<[u8; 32]> {
        if hashes.is_empty() || index >= hashes.len() {
            return Vec::new();
        }

        if hashes.len() == 1 {
            return Vec::new();
        }

        let mut proof = Vec::new();
        let mut current_level: Vec<[u8; 32]> = hashes.to_vec();
        let mut current_index = index;

        while current_level.len() > 1 {
            // Find sibling hash
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            if sibling_index < current_level.len() {
                proof.push(current_level[sibling_index]);
            } else {
                // Odd number, sibling is the same element
                proof.push(current_level[current_index]);
            }

            // Build next level
            let mut next_level = Vec::new();
            for chunk in current_level.chunks(2) {
                let combined_hash = if chunk.len() == 2 {
                    AccumulateHash::sha256_concat(&[&chunk[0], &chunk[1]])
                } else {
                    AccumulateHash::sha256_concat(&[&chunk[0], &chunk[0]])
                };
                next_level.push(combined_hash);
            }

            current_level = next_level;
            current_index /= 2;
        }

        proof
    }

    /// Verify Merkle proof
    /// Matches TS: verifyMerkleProof(root: Uint8Array, leaf: Uint8Array, proof: Uint8Array[], index: number): boolean
    pub fn verify_merkle_proof(
        root: &[u8; 32],
        leaf: &[u8; 32],
        proof: &[[u8; 32]],
        index: usize,
    ) -> bool {
        let mut computed_hash = *leaf;
        let mut current_index = index;

        for sibling in proof {
            computed_hash = if current_index % 2 == 0 {
                AccumulateHash::sha256_concat(&[&computed_hash, sibling])
            } else {
                AccumulateHash::sha256_concat(&[sibling, &computed_hash])
            };
            current_index /= 2;
        }

        &computed_hash == root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_sha256_consistency() {
        let data = b"hello world";
        let hash1 = AccumulateHash::sha256_bytes(data);
        let hash2 = AccumulateHash::sha256_bytes(data);
        assert_eq!(hash1, hash2);

        let hex1 = AccumulateHash::sha256_bytes_hex(data);
        let hex2 = AccumulateHash::sha256_bytes_hex(data);
        assert_eq!(hex1, hex2);
        assert_eq!(hex1, hex::encode(hash1));
    }

    #[test]
    fn test_json_hashing() {
        let value = json!({
            "name": "test",
            "value": 42,
            "array": [1, 2, 3]
        });

        let hash1 = AccumulateHash::sha256_json(&value);
        let hash2 = AccumulateHash::sha256_json(&value);
        assert_eq!(hash1, hash2);

        // Test with reordered object - should produce same hash due to canonical JSON
        let value2 = json!({
            "value": 42,
            "array": [1, 2, 3],
            "name": "test"
        });

        let hash3 = AccumulateHash::sha256_json(&value2);
        assert_eq!(hash1, hash3);
    }

    #[test]
    fn test_url_normalization() {
        let test_cases = vec![
            ("acc://alice.acme", "acc://alice.acme"),
            ("ACC://ALICE.ACME", "acc://alice.acme"),
            ("acc://alice.acme/", "acc://alice.acme"),
            ("acc://alice.acme///", "acc://alice.acme"),
            ("//alice.acme", "acc://alice.acme"),
            ("alice.acme", "acc://alice.acme"),
            ("/alice.acme", "acc://alice.acme"),
        ];

        for (input, expected) in test_cases {
            let normalized = UrlHash::normalize_url(input);
            assert_eq!(normalized, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_url_hashing() {
        let url = "acc://alice.acme";
        let hash1 = UrlHash::hash_url(url);
        let hash2 = UrlHash::hash_url("ACC://ALICE.ACME/");
        assert_eq!(
            hash1, hash2,
            "URL hashing should be case and trailing slash insensitive"
        );

        let hex = UrlHash::hash_url_hex(url);
        assert_eq!(hex, hex::encode(hash1));
    }

    #[test]
    fn test_url_derivation() {
        let identity_url = "acc://alice.acme";
        let key_book_url = UrlHash::derive_key_book_url(identity_url);
        assert_eq!(key_book_url, "acc://alice.acme/book");

        let key_page_url = UrlHash::derive_key_page_url(&key_book_url, 0);
        assert_eq!(key_page_url, "acc://alice.acme/book/0");
    }

    #[test]
    fn test_url_parsing() {
        let url = "acc://alice.acme/tokens";
        let authority = UrlHash::extract_authority(url).unwrap();
        assert_eq!(authority, "alice.acme");

        let path = UrlHash::extract_path(url);
        assert_eq!(path, "/tokens");

        let root_url = "acc://alice.acme";
        let root_path = UrlHash::extract_path(root_url);
        assert_eq!(root_path, "/");
    }

    #[test]
    fn test_chain_id_hashing() {
        let url = "acc://alice.acme";
        let chain_id1 = ChainHash::hash_chain_id(url);
        let chain_id2 = ChainHash::hash_chain_id("ACC://ALICE.ACME/");
        assert_eq!(chain_id1, chain_id2);

        let main_chain_id = ChainHash::derive_main_chain_id("alice.acme");
        assert_eq!(chain_id1, main_chain_id);
    }

    #[test]
    fn test_merkle_tree() {
        let hashes = vec![[1u8; 32], [2u8; 32], [3u8; 32], [4u8; 32]];

        let root = MerkleHash::build_merkle_root(&hashes);
        assert_ne!(root, [0u8; 32]);

        // Test single element
        let single_root = MerkleHash::build_merkle_root(&hashes[0..1]);
        assert_eq!(single_root, hashes[0]);

        // Test empty
        let empty_root = MerkleHash::build_merkle_root(&[]);
        assert_eq!(empty_root, [0u8; 32]);
    }

    #[test]
    fn test_merkle_proof() {
        let hashes = vec![[1u8; 32], [2u8; 32], [3u8; 32], [4u8; 32]];

        let root = MerkleHash::build_merkle_root(&hashes);
        let proof = MerkleHash::create_merkle_proof(&hashes, 0);

        let is_valid = MerkleHash::verify_merkle_proof(&root, &hashes[0], &proof, 0);
        assert!(is_valid);

        // Test invalid proof
        let invalid_proof = MerkleHash::create_merkle_proof(&hashes, 1);
        let is_invalid = MerkleHash::verify_merkle_proof(&root, &hashes[0], &invalid_proof, 0);
        assert!(!is_invalid);
    }

    #[test]
    fn test_double_hash() {
        let data = b"test data";
        let single_hash = AccumulateHash::sha256_bytes(data);
        let double_hash = AccumulateHash::double_sha256(data);

        assert_ne!(single_hash, double_hash);
        assert_eq!(double_hash, AccumulateHash::sha256_bytes(&single_hash));
    }

    #[test]
    fn test_concat_hash() {
        let arrays = [b"hello".as_slice(), b" ".as_slice(), b"world".as_slice()];
        let concat_hash = AccumulateHash::sha256_concat(&arrays);
        let direct_hash = AccumulateHash::sha256_bytes(b"hello world");
        assert_eq!(concat_hash, direct_hash);
    }

    #[test]
    fn test_binary_encodable() {
        let test_u64 = 12345u64;
        let encoded = test_u64.encode_binary().unwrap();
        assert_eq!(encoded, BinaryWriter::encode_uvarint(test_u64));

        let test_string = "hello";
        let encoded = test_string.encode_binary().unwrap();
        assert_eq!(encoded, BinaryWriter::encode_string(test_string));

        let test_bytes = vec![1, 2, 3, 4];
        let encoded = test_bytes.encode_binary().unwrap();
        assert_eq!(encoded, BinaryWriter::encode_bytes(&test_bytes));
    }
}
