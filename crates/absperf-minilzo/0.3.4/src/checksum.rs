use absperf_minilzo_sys::{lzo_adler32, lzo_uint};

/// Compute adler32.
/// This is made to be chainable.  On initial call, set the prev_checksum parameter to 1, or use
/// the plain adler32 call
/// given x: [u8; 4], adler32(&x) == adler32_chain(1, &x) == adler32_chain(adler32(&x[..2]), &x[2..])
pub fn adler32_chain(prev_checksum: u32, input: &[u8]) -> u32 {
    unsafe { lzo_adler32(prev_checksum, input.as_ptr(), input.len() as lzo_uint) }
}

/// A convenience call for adler32_chain(1, input)
pub fn adler32(input: &[u8]) -> u32 {
    adler32_chain(1, input)
}

#[cfg(test)]
mod tests {
    use super::*;
    const CHECKSUM: u32 = 0x3C2F9A8C;

    #[test]
    fn adler32() {
        let source = include_bytes!("lorem.txt");

        assert_eq!(super::adler32(source), CHECKSUM);

        let half_len = source.len() / 2;
        assert_eq!(
            adler32_chain(super::adler32(&source[..half_len]), &source[half_len..]),
            CHECKSUM
        );
    }
}
