pub mod aws;

use chrono::DateTime;
use chrono::Utc;
use hmac_sha256::{Hash, HMAC};

// Base Implementation of the V4 Signing method.  The idea is that we would build one for AWS, Google,
// and any other service that uses the standard V4 signing method.
pub trait SignedV4Base {
    fn generate_auth_header(
        &self,
        payload: &str,
        method: &str,
        path: &str,
        now: DateTime<Utc>,
    ) -> String;
}

// For uploads, we use the unsigned payload option.  To sign the payload
// we would need to read the body and hash it which we dont want to do.  We
// want to keep the body bytes out of the guest to avoid memory utilization.
// See:  https://docs.aws.amazon.com/AmazonS3/latest/API/sig-v4-header-based-auth.html
// The spec says that we put the literal string UNSIGNED-PAYLOAD in the payload
// instead of hex-encoded hash value
pub fn unsigned_payload_hash() -> String {
    "UNSIGNED-PAYLOAD".to_string()
}

// SHA256 HMAC
pub fn sign(key: Vec<u8>, input: String) -> [u8; 32] {
    HMAC::mac(input.as_bytes(), &key)
}

// Create a hex output of the hash
pub fn hash_str(input: String) -> String {
    hex::encode(Hash::hash(input.as_bytes()))
}
