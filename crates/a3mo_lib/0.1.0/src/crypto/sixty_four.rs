use crate::crypto::TransmutingByteSlices;
use core::{cmp, hash::Hasher};

#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

const CHUNK_SIZE: usize = 32;

const PRIME_1: u64 = 11_400_714_785_074_694_791;
const PRIME_2: u64 = 14_029_467_366_897_019_727;
const PRIME_3: u64 = 1_609_587_929_392_839_161;
const PRIME_4: u64 = 9_650_029_242_287_828_579;
const PRIME_5: u64 = 2_870_177_450_012_600_261;

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[derive(Copy, Clone, PartialEq)]
struct XxCore {
    v1: u64,
    v2: u64,
    v3: u64,
    v4: u64,
}

/// Calculates the 64-bit hash.
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct XxHash64 {
    total_len: u64,
    seed: u64,
    core: XxCore,
    #[cfg_attr(feature = "serialize", serde(flatten))]
    buffer: Buffer,
}

impl XxCore {
    fn with_seed(seed: u64) -> XxCore {
        XxCore {
            v1: seed.wrapping_add(PRIME_1).wrapping_add(PRIME_2),
            v2: seed.wrapping_add(PRIME_2),
            v3: seed,
            v4: seed.wrapping_sub(PRIME_1),
        }
    }

    #[inline(always)]
    fn ingest_chunks<'a, I>(&mut self, values: I)
    where
        I: IntoIterator<Item = &'a [u64; 4]>,
    {
        #[inline(always)]
        fn ingest_one_number(mut current_value: u64, mut value: u64) -> u64 {
            value = value.wrapping_mul(PRIME_2);
            current_value = current_value.wrapping_add(value);
            current_value = current_value.rotate_left(31);
            current_value.wrapping_mul(PRIME_1)
        };

        // By drawing these out, we can avoid going back and forth to
        // memory. It only really helps for large files, when we need
        // to iterate multiple times here.

        let mut v1 = self.v1;
        let mut v2 = self.v2;
        let mut v3 = self.v3;
        let mut v4 = self.v4;

        for &[n1, n2, n3, n4] in values {
            v1 = ingest_one_number(v1, n1);
            v2 = ingest_one_number(v2, n2);
            v3 = ingest_one_number(v3, n3);
            v4 = ingest_one_number(v4, n4);
        }

        self.v1 = v1;
        self.v2 = v2;
        self.v3 = v3;
        self.v4 = v4;
    }

    #[inline(always)]
    fn finish(&self) -> u64 {
        // The original code pulls out local vars for v[1234]
        // here. Performance tests did not show that to be effective
        // here, presumably because this method is not called in a
        // tight loop.

        let mut hash;

        hash = self.v1.rotate_left(1);
        hash = hash.wrapping_add(self.v2.rotate_left(7));
        hash = hash.wrapping_add(self.v3.rotate_left(12));
        hash = hash.wrapping_add(self.v4.rotate_left(18));

        #[inline(always)]
        fn mix_one(mut hash: u64, mut value: u64) -> u64 {
            value = value.wrapping_mul(PRIME_2);
            value = value.rotate_left(31);
            value = value.wrapping_mul(PRIME_1);
            hash ^= value;
            hash = hash.wrapping_mul(PRIME_1);
            hash.wrapping_add(PRIME_4)
        }

        hash = mix_one(hash, self.v1);
        hash = mix_one(hash, self.v2);
        hash = mix_one(hash, self.v3);
        hash = mix_one(hash, self.v4);

        hash
    }
}

impl core::fmt::Debug for XxCore {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "XxCore {{ {:016x} {:016x} {:016x} {:016x} }}",
            self.v1, self.v2, self.v3, self.v4
        )
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, Default, PartialEq)]
#[repr(align(8))]
#[cfg_attr(feature = "serialize", serde(transparent))]
struct AlignToU64<T>(T);

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, Default, PartialEq)]
struct Buffer {
    #[cfg_attr(feature = "serialize", serde(rename = "buffer"))]
    data: AlignToU64<[u8; CHUNK_SIZE]>,
    #[cfg_attr(feature = "serialize", serde(rename = "buffer_usage"))]
    len: usize,
}

impl Buffer {
    fn data(&self) -> &[u8] {
        &self.data.0[..self.len]
    }

    fn as_u64_arrays(&self) -> &[[u64; 4]] {
        let (head, u64_arrays, tail) = self.data().as_u64_arrays();

        debug_assert!(head.is_empty(), "buffer was not aligned for 64-bit numbers");
        debug_assert_eq!(
            u64_arrays.len(),
            1,
            "buffer did not have enough 64-bit numbers"
        );
        debug_assert!(tail.is_empty(), "buffer has trailing data");

        u64_arrays
    }

    fn as_u64s_and_u32s(&self) -> (&[u64], &[u32], &[u8]) {
        let (head, u64s, tail) = self.data().as_u64s();

        debug_assert!(head.is_empty(), "buffer was not aligned for 64-bit numbers");

        let (head, u32s, tail) = tail.as_u32s();

        debug_assert!(head.is_empty(), "buffer was not aligned for 32-bit numbers");

        (u64s, u32s, tail)
    }

    /// Consumes as much of the parameter as it can, returning the unused part.
    fn consume<'a>(&mut self, data: &'a [u8]) -> &'a [u8] {
        let to_use = cmp::min(self.available(), data.len());
        let (data, remaining) = data.split_at(to_use);
        self.data.0[self.len..][..to_use].copy_from_slice(data);
        self.len += to_use;
        remaining
    }

    fn available(&self) -> usize {
        CHUNK_SIZE - self.len
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn is_full(&self) -> bool {
        self.len == CHUNK_SIZE
    }
}

impl XxHash64 {
    /// Constructs the hash with an initial seed
    pub fn with_seed(seed: u64) -> XxHash64 {
        XxHash64 {
            total_len: 0,
            seed,
            core: XxCore::with_seed(seed),
            buffer: Buffer::default(),
        }
    }

    pub(crate) fn write(&mut self, bytes: &[u8]) {
        let (unaligned_head, aligned, unaligned_tail) = bytes.as_u64_arrays();

        if !self.buffer.is_empty() || !unaligned_head.is_empty() {
            self.buffer_bytes(bytes);
        } else {
            self.core.ingest_chunks(aligned);
            self.buffer_bytes(unaligned_tail);
        }

        self.total_len += bytes.len() as u64;
    }

    fn buffer_bytes(&mut self, mut data: &[u8]) {
        while !data.is_empty() {
            data = self.buffer.consume(data);
            if self.buffer.is_full() {
                self.core.ingest_chunks(self.buffer.as_u64_arrays());
                self.buffer.len = 0;
            }
        }
    }

    pub(crate) fn finish(&self) -> u64 {
        let mut hash = if self.total_len >= CHUNK_SIZE as u64 {
            // We have processed at least one full chunk
            self.core.finish()
        } else {
            self.seed.wrapping_add(PRIME_5)
        };

        hash = hash.wrapping_add(self.total_len);

        let (buffered_u64s, buffered_u32s, buffered_u8s) = self.buffer.as_u64s_and_u32s();

        for buffered_u64 in buffered_u64s {
            let mut k1 = buffered_u64.wrapping_mul(PRIME_2);
            k1 = k1.rotate_left(31);
            k1 = k1.wrapping_mul(PRIME_1);
            hash ^= k1;
            hash = hash.rotate_left(27);
            hash = hash.wrapping_mul(PRIME_1);
            hash = hash.wrapping_add(PRIME_4);
        }

        for &buffered_u32 in buffered_u32s {
            let k1 = u64::from(buffered_u32).wrapping_mul(PRIME_1);
            hash ^= k1;
            hash = hash.rotate_left(23);
            hash = hash.wrapping_mul(PRIME_2);
            hash = hash.wrapping_add(PRIME_3);
        }

        for &buffered_u8 in buffered_u8s {
            let k1 = u64::from(buffered_u8).wrapping_mul(PRIME_5);
            hash ^= k1;
            hash = hash.rotate_left(11);
            hash = hash.wrapping_mul(PRIME_1);
        }

        // The final intermixing
        hash ^= hash >> 33;
        hash = hash.wrapping_mul(PRIME_2);
        hash ^= hash >> 29;
        hash = hash.wrapping_mul(PRIME_3);
        hash ^= hash >> 32;

        hash
    }
}

impl Default for XxHash64 {
    fn default() -> XxHash64 {
        XxHash64::with_seed(0)
    }
}

impl Hasher for XxHash64 {
    fn write(&mut self, bytes: &[u8]) {
        XxHash64::write(self, bytes)
    }

    fn finish(&self) -> u64 {
        XxHash64::finish(self)
    }
}

#[cfg(feature = "std")]
pub use crate::std_support::sixty_four::RandomXxHashBuilder64;
