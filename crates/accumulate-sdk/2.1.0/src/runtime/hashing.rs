use sha2::{Digest, Sha256};

/// Compute SHA-256 hash of input data
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let out = hasher.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&out);
    arr
}

/// Compute SHA-256 hash and return as hex string
pub fn sha256_hex(data: &[u8]) -> String {
    hex::encode(sha256(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_basic() {
        let input = b"hello world";
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert_eq!(sha256_hex(input), expected);
    }

    #[test]
    fn test_sha256_empty() {
        let input = b"";
        let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert_eq!(sha256_hex(input), expected);
    }

    #[test]
    fn test_sha256_deterministic() {
        let input = b"test deterministic";
        let hash1 = sha256(input);
        let hash2 = sha256(input);
        assert_eq!(hash1, hash2, "Hash should be deterministic");
    }
}