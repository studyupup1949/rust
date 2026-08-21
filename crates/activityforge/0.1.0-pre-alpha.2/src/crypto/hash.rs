use hmac::Hmac;
use sha3::Sha3_256;

/// Convenience alias for the HMAC-SHA3-256 keyed-hash algorithm.
pub type HmacSha3_256 = Hmac<Sha3_256>;
