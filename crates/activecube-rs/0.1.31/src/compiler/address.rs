//! Address encoding normalization for `FilterValueTransform::FormatAddress`.
//!
//! Pure, side-effect-free helpers that re-encode user filter values to match
//! the canonical storage form used by the DB. Invalid / unrecognizable inputs
//! return `None` so callers can pass-through unchanged (filters on the same
//! column may legitimately carry non-address substrings — e.g. partial
//! prefix matches — and we don't want to mangle them).
//!
//! # Encodings supported
//!
//! | Enum variant      | Canonical form                                   | Validity check                    |
//! |-------------------|--------------------------------------------------|-----------------------------------|
//! | `EvmEip55`        | `0x` + 40 mixed-case hex (keccak-256 checksum)   | `0x` + exactly 40 hex chars       |
//! | `EvmLower`        | `0x` + 40 lower-case hex                         | `0x` + exactly 40 hex chars       |
//! | `Base58`          | input unchanged                                  | all chars in base58 alphabet      |
//! | `Base58Check`     | re-encoded with sha256d checksum                 | `bs58::decode_check` succeeds     |
//!
//! # EIP-55 algorithm (reference: https://eips.ethereum.org/EIPS/eip-55)
//!
//! 1. Lower-case the 40-char hex body (no `0x`).
//! 2. Compute `keccak256` over the ASCII bytes of that lower-case string.
//! 3. For each hex nibble at position `i`, if byte `hash[i/2]` has the
//!    corresponding half-byte ≥ 8, upper-case the nibble; else lower-case.
//! 4. Prepend `0x`.

use tiny_keccak::{Hasher, Keccak};

/// Re-encode an EVM address to EIP-55 checksum form.
/// Returns `None` if the input isn't `0x` + 40 hex chars.
pub fn to_eip55(addr: &str) -> Option<String> {
    let body = strip_0x_and_validate_hex(addr, 40)?;
    let lower = body.to_ascii_lowercase();

    let mut hasher = Keccak::v256();
    let mut hash = [0u8; 32];
    hasher.update(lower.as_bytes());
    hasher.finalize(&mut hash);

    let mut out = String::with_capacity(42);
    out.push_str("0x");
    for (i, ch) in lower.chars().enumerate() {
        let hash_byte = hash[i / 2];
        let nibble = if i % 2 == 0 { hash_byte >> 4 } else { hash_byte & 0x0f };
        if ch.is_ascii_digit() {
            out.push(ch);
        } else if nibble >= 8 {
            out.push(ch.to_ascii_uppercase());
        } else {
            out.push(ch);
        }
    }
    Some(out)
}

/// Re-encode an EVM address to lower-case hex.
/// Returns `None` if the input isn't `0x` + 40 hex chars.
pub fn to_evm_lower(addr: &str) -> Option<String> {
    let body = strip_0x_and_validate_hex(addr, 40)?;
    Some(format!("0x{}", body.to_ascii_lowercase()))
}

/// Pass-through validator for base58 values: returns `Some(input)` if
/// the string decodes as valid base58, else `None`. Does NOT alter the
/// string — base58 alphabet is already case-sensitive and unique.
pub fn normalize_base58(s: &str) -> Option<String> {
    if s.is_empty() {
        return None;
    }
    bs58::decode(s).into_vec().ok()?;
    Some(s.to_string())
}

/// Re-encode a base58check value (sha256d checksum). Returns canonical
/// encoding if the input checksum validates, else `None`.
pub fn normalize_base58check(s: &str) -> Option<String> {
    if s.is_empty() {
        return None;
    }
    let bytes = bs58::decode(s).with_check(None).into_vec().ok()?;
    Some(bs58::encode(bytes).with_check().into_string())
}

fn strip_0x_and_validate_hex(addr: &str, expected_hex_len: usize) -> Option<&str> {
    let body = addr.strip_prefix("0x").or_else(|| addr.strip_prefix("0X"))?;
    if body.len() != expected_hex_len {
        return None;
    }
    if body.bytes().all(|b| b.is_ascii_hexdigit()) {
        Some(body)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// EIP-55 reference vectors from the spec.
    #[test]
    fn eip55_reference_vectors() {
        let cases = [
            ("0x5aaeb6053f3e94c9b9a09f33669435e7ef1beaed", "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed"),
            ("0xfb6916095ca1df60bb79ce92ce3ea74c37c5d359", "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359"),
            ("0xdbf03b407c01e7cd3cbea99509d93f8dddc8c6fb", "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB"),
            ("0xd1220a0cf47c7b9be7a2e6ba89f429762e7b9adb", "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb"),
            // Mixed-case input must normalize to same canonical output
            ("0x5D4F3C6fA16908609BAC31Ff148Bd002AA6b8c83", "0x5d4F3C6fA16908609BAC31Ff148Bd002AA6b8c83"),
        ];
        for (input, expected) in cases {
            assert_eq!(to_eip55(input).as_deref(), Some(expected), "input {input}");
        }
    }

    #[test]
    fn eip55_invalid_returns_none() {
        assert!(to_eip55("").is_none());
        assert!(to_eip55("0x").is_none());
        assert!(to_eip55("0x123").is_none());
        assert!(to_eip55("5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed").is_none(), "missing 0x prefix");
        assert!(to_eip55("0xzzAeb6053F3E94C9b9A09f33669435E7Ef1BeAed").is_none(), "non-hex char");
        assert!(to_eip55("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed00").is_none(), "too long");
    }

    #[test]
    fn evm_lower_basic() {
        assert_eq!(
            to_evm_lower("0x5D4F3C6fA16908609BAC31Ff148Bd002AA6b8c83").as_deref(),
            Some("0x5d4f3c6fa16908609bac31ff148bd002aa6b8c83"),
        );
        assert_eq!(
            to_evm_lower("0X5D4F3C6fA16908609BAC31Ff148Bd002AA6b8c83").as_deref(),
            Some("0x5d4f3c6fa16908609bac31ff148bd002aa6b8c83"),
            "0X prefix accepted"
        );
        assert!(to_evm_lower("0x123").is_none());
    }

    #[test]
    fn base58_passthrough() {
        // Typical Solana pubkey (32-byte base58 = 32..44 chars)
        let sol = "27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4";
        assert_eq!(normalize_base58(sol).as_deref(), Some(sol));
        // Native SOL sentinel
        let native = "11111111111111111111111111111111";
        assert_eq!(normalize_base58(native).as_deref(), Some(native));
        // WSOL
        let wsol = "So11111111111111111111111111111111111111112";
        assert_eq!(normalize_base58(wsol).as_deref(), Some(wsol));

        // EVM hex should NOT validate as base58 (contains '0' which is not in alphabet)
        assert!(normalize_base58("0x5d4F3C6fA16908609BAC31Ff148Bd002AA6b8c83").is_none());
        assert!(normalize_base58("").is_none());
    }

    #[test]
    fn base58check_roundtrip() {
        // Bitcoin genesis address has valid base58check; should re-encode identically.
        let btc = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
        assert_eq!(normalize_base58check(btc).as_deref(), Some(btc));
        // Invalid checksum → None
        assert!(normalize_base58check("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNb").is_none());
    }
}
