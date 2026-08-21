use std::ops::BitXor;

/// `u64::MAX / PI`
const C: u64 = 587178100656400245;

/// Heavily inspired by [`FxHasher`].
///
/// [`FxHasher`]: https://github.com/rust-lang/rustc-hash
#[derive(Debug, Default)]
pub struct Hasher {
    state: u64,
}

impl Hasher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&mut self, chunk: u64) {
        self.state = self.state.rotate_left(5).bitxor(chunk).wrapping_mul(C);
    }

    pub fn write_bytes(&mut self, mut bytes: &[u8]) {
        while bytes.len() > 8 {
            self.write(u64::from_ne_bytes(bytes[..8].try_into().unwrap()));
            bytes = &bytes[8..];
        }
        let mut bytes = bytes.to_vec();
        bytes.resize(8, 0);
        self.write(u64::from_ne_bytes(bytes[..8].try_into().unwrap()));
    }

    pub fn finalize(self) -> u64 {
        self.state
    }

    pub fn hash_bytes(bytes: &[u8]) -> u64 {
        let mut hasher = Hasher::new();
        hasher.write_bytes(bytes);
        hasher.finalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_input_produces_same_hash() {
        let a = Hasher::hash_bytes(b"same input");
        let b = Hasher::hash_bytes(b"same input");
        assert_eq!(a, b);
    }

    #[test]
    fn similar_input_produces_different_hash() {
        let a = Hasher::hash_bytes(b"same input");
        let b = Hasher::hash_bytes(b"some input");
        assert_ne!(a, b);

        let mut c = Hasher::new();
        c.write_bytes(b"same input");
        c.write(0);
        let c = c.finalize();
        assert_ne!(a, c);
    }

    #[test]
    fn order_matters() {
        let mut a = Hasher::new();
        a.write_bytes(b"alice");
        a.write_bytes(b"bob");
        let a = a.finalize();

        let mut b = Hasher::new();
        b.write_bytes(b"bob");
        b.write_bytes(b"alice");
        let b = b.finalize();

        assert_ne!(a, b);
    }
}
