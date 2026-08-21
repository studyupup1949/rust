//! # Adler 32
//!
//! Adler 32 checksum algorithm
//!
//! [More information about Adler-32](https://en.wikipedia.org/wiki/Adler-32)

use crate::primitives::Result;
use byteorder::{BigEndian, ReadBytesExt};
use rayon::prelude::*;

const ADLER32_MOD: u32 = 0xFFF1;

#[derive(Copy, Clone)]
pub struct Adler32 {
    initial_value: [u8; 8],
}

pub struct Adler32Builder {
    values: Vec<Vec<u8>>,
    checksum: Adler32,
}

pub struct Adler32Result {
    hash: Vec<u8>,
    result: Result<Vec<u8>>,
}

unsafe impl Send for Adler32Result {}

impl Adler32Result {
    pub fn new(hash: Vec<u8>, result: Result<Vec<u8>>) -> Self {
        Adler32Result { hash, result }
    }

    pub fn is_ok(&self) -> bool {
        self.result.is_ok()
    }

    pub fn get_hash(&self) -> &Vec<u8> {
        &self.hash
    }

    pub fn get_result(&self) -> &Result<Vec<u8>> {
        &self.result
    }
}

impl Adler32 {
    pub fn new(initial_value: [u8; 8]) -> Self {
        Adler32 { initial_value }
    }

    /// Checksum hash with [Adler-32](https://en.wikipedia.org/wiki/Adler-32)
    ///
    /// Generate checksum from hash with developer defined left and right initialized values.
    #[inline(always)]
    pub fn adler32_checksum(self, hash: &[u8]) -> Result<Vec<u8>> {
        let mut split = self.initial_value.split_at(4);
        let left_init = split.0.read_u32::<BigEndian>().unwrap();
        let right_init = split.1.read_u32::<BigEndian>().unwrap();

        // https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html#method.fold
        let (lsb, msb) = hash
            .iter()
            .fold((left_init, right_init), |(left, right), &byte| {
                (
                    left.wrapping_add(byte as u32) % ADLER32_MOD,
                    right.wrapping_add(left + (byte) as u32) % ADLER32_MOD,
                )
            });
        Ok(((msb << 16) | lsb).to_be_bytes().to_vec())
    }
}

impl Adler32Builder {
    pub fn new(checksum: Adler32) -> Self {
        Adler32Builder {
            values: Vec::new(),
            checksum,
        }
    }

    pub fn push(&mut self, hash: Vec<u8>) -> &Self {
        self.values.push(hash);
        self
    }

    pub fn push_vec(mut self, hash: Vec<Vec<u8>>) -> Self {
        let _ = hash.iter().map(|h| self.values.push(h.clone()));
        self
    }

    pub fn finalize(self) -> Vec<Adler32Result> {
        self.values
            .par_iter()
            .map(|i| Adler32Result::new(i.clone(), self.checksum.adler32_checksum(i)))
            .filter(|x| x.is_ok())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::adler32::Adler32;

    // Example test from [Adler-32](https://en.wikipedia.org/wiki/Adler-32) Wikipedia page
    #[test]
    fn validate_adler32_checksum() {
        let checksum = Adler32::new([0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00])
            .adler32_checksum(&"Wikipedia".as_bytes());

        assert!(checksum.is_ok());
        assert_eq!(300286872_u32.to_be_bytes().to_vec(), checksum.unwrap())
    }
}
