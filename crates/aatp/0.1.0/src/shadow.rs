//! A reversible XOR keystream layer — **obfuscation, NOT encryption.**
//!
//! ⚠️ **This is not secure and must not be used for confidentiality.** It XORs the
//! payload with a keystream derived from a `u64` key. That hides bytes from a casual
//! glance and is exactly reversible, but a static-key XOR stream is *trivially*
//! broken: any known plaintext recovers the keystream, two messages under one key
//! XOR to reveal each other, and there is **no authentication** of the result. For
//! real confidentiality use a vetted authenticated-encryption (AEAD) construction. This layer exists
//! to demonstrate the honest limit of XOR: a reversible, `unsafe`-free transform that
//! noha can hold at 100% mutation coverage — which proves it is *correct and
//! reversible*, NOT that it is *secret*.
//!
//! [`mask`] is its own inverse: masking twice with the same key returns the input.

/// A deterministic keystream from a `u64` key — SplitMix64, a well-mixed counter
/// generator that behaves for every seed (including `0`).
struct KeyStream {
    state: u64,
}

impl KeyStream {
    fn new(key: u64) -> KeyStream {
        KeyStream { state: key }
    }

    /// The next keystream byte. SplitMix64: advance the counter, then avalanche it.
    fn next_byte(&mut self) -> u8 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z = z ^ (z >> 31);
        (z & 0xFF) as u8
    }
}

/// What can go wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObfuscateError {
    /// `out` is smaller than `data`.
    OutputTooSmall,
}

/// XOR `data` with the keystream for `key`, writing to `out`; returns the length
/// (always `data.len()`). Reversible: `mask(mask(x, k), k) == x`. **Obfuscation only —
/// see the module docs; this provides no real confidentiality.**
pub fn mask(data: &[u8], key: u64, out: &mut [u8]) -> Result<usize, ObfuscateError> {
    if out.len() < data.len() {
        return Err(ObfuscateError::OutputTooSmall);
    }
    let mut stream = KeyStream::new(key);
    let mut i = 0;
    while i < data.len() {
        out[i] = data[i] ^ stream.next_byte();
        i += 1;
    }
    Ok(data.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: u64 = 0x0123_4567_89AB_CDEF;

    /// Known-answer keystream: `mask` of all-zero input IS the keystream, so this pins
    /// the SplitMix64 shifts/xors/`& 0xFF` — a mutation of any of them changes these
    /// bytes even though a round trip alone would not (any keystream self-inverts).
    #[test]
    fn keystream_matches_its_known_answer() {
        let mut out = [0u8; 8];
        mask(&[0u8; 8], KEY, &mut out).unwrap();
        assert_eq!(out, [0x9D, 0x93, 0xBE, 0xEC, 0x08, 0x72, 0x05, 0x51]);
    }

    #[test]
    fn masking_twice_returns_the_input() {
        let data = b"tool_call: web_search(\"noha release notes\")";
        let mut once = std::vec![0u8; data.len()];
        mask(data, KEY, &mut once).unwrap();
        assert_ne!(
            &once[..],
            &data[..],
            "masking must actually change the bytes"
        );
        let mut twice = std::vec![0u8; data.len()];
        mask(&once, KEY, &mut twice).unwrap();
        assert_eq!(&twice[..], &data[..], "masking twice is the identity");
    }

    #[test]
    fn a_different_key_yields_a_different_stream() {
        let data = [0u8; 16];
        let mut a = [0u8; 16];
        let mut b = [0u8; 16];
        mask(&data, KEY, &mut a).unwrap();
        mask(&data, KEY ^ 1, &mut b).unwrap();
        assert_ne!(a, b, "the keystream must depend on the key");
    }

    #[test]
    fn empty_input_is_masked_to_empty() {
        let mut out = [0u8; 0];
        assert_eq!(mask(&[], KEY, &mut out), Ok(0));
    }

    #[test]
    fn mask_rejects_a_too_small_output_at_the_boundary() {
        let data = [1u8, 2, 3];
        let mut short = [0u8; 2];
        assert_eq!(
            mask(&data, KEY, &mut short),
            Err(ObfuscateError::OutputTooSmall)
        );
        let mut exact = [0u8; 3];
        assert_eq!(mask(&data, KEY, &mut exact), Ok(3));
    }
}
