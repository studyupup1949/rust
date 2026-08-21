//! Bounds-checked little-endian integer reads over a byte slice.
//!
//! AD1 is attacker-controllable input (Paranoid Gatekeeper): every integer read
//! goes through these helpers, which return `0` when the requested range falls
//! outside the buffer instead of panicking. Centralizing the bounds check here
//! means no other module needs raw slice indexing.

/// Read a little-endian `u32` at `off`, or `0` if out of range.
#[must_use]
pub fn u32_le(buf: &[u8], off: usize) -> u32 {
    match buf.get(off..off + 4) {
        Some(b) => u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
        None => 0,
    }
}

/// Read a little-endian `u64` at `off`, or `0` if out of range.
#[must_use]
pub fn u64_le(buf: &[u8], off: usize) -> u64 {
    match buf.get(off..off + 8) {
        Some(b) => u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]),
        None => 0,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn u32_reads_little_endian_at_offset() {
        let buf = [0xff, 0x78, 0x56, 0x34, 0x12];
        assert_eq!(u32_le(&buf, 1), 0x1234_5678);
    }

    #[test]
    fn u64_reads_little_endian() {
        let buf = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        assert_eq!(u64_le(&buf, 0), 0x0807_0605_0403_0201);
    }

    #[test]
    fn out_of_range_reads_return_zero_never_panic() {
        let buf = [0xaa, 0xbb, 0xcc];
        assert_eq!(u32_le(&buf, 0), 0); // need 4, have 3
        assert_eq!(u64_le(&buf, 0), 0);
        assert_eq!(u32_le(&buf, 99), 0); // far past the end
        assert_eq!(u64_le(&[], 0), 0); // empty
    }
}
