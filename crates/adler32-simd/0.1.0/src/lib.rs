//! SIMD-accelerated Adler-32 checksum.
//!
//! Vectorized on ARM64 (NEON) and x86/x86_64 (SSSE3), with a scalar
//! fallback for other platforms.
//!
//! # Usage
//!
//! One-shot:
//! ```
//! let checksum = adler32_simd::adler32(b"Hello, world!");
//! ```
//!
//! Streaming:
//! ```
//! let mut hasher = adler32_simd::Adler32::new();
//! hasher.write(b"Hello, ");
//! hasher.write(b"world!");
//! let checksum = hasher.checksum();
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

const MOD: u32 = 65521;

/// Adler-32 hash instance.
#[derive(Clone)]
pub struct Adler32 {
    a: u16,
    b: u16,
}

type UpdateFn = fn(u16, u16, &[u8]) -> (u16, u16);

impl Adler32 {
    pub fn new() -> Self {
        Self { a: 1, b: 0 }
    }

    pub fn from_checksum(checksum: u32) -> Self {
        Self {
            a: checksum as u16,
            b: (checksum >> 16) as u16,
        }
    }

    pub fn checksum(&self) -> u32 {
        (u32::from(self.b) << 16) | u32::from(self.a)
    }

    pub fn write(&mut self, data: &[u8]) {
        let imp = get_imp();
        let (a, b) = imp(self.a, self.b, data);
        self.a = a;
        self.b = b;
    }

    pub fn finish(&self) -> u32 {
        self.checksum()
    }
}

impl Default for Adler32 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "std")]
impl std::hash::Hasher for Adler32 {
    fn write(&mut self, bytes: &[u8]) {
        Adler32::write(self, bytes);
    }

    fn finish(&self) -> u64 {
        self.checksum() as u64
    }
}

/// Compute Adler-32 of a byte slice in one shot.
pub fn adler32(data: &[u8]) -> u32 {
    let mut h = Adler32::new();
    h.write(data);
    h.checksum()
}

fn get_imp() -> UpdateFn {
    #[cfg(target_arch = "aarch64")]
    {
        if is_aarch64_neon_available() {
            return neon::update;
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_ssse3_available() {
            return ssse3::update;
        }
    }

    scalar::update
}

#[cfg(target_arch = "aarch64")]
fn is_aarch64_neon_available() -> bool {
    #[cfg(feature = "std")]
    {
        std::arch::is_aarch64_feature_detected!("neon")
    }
    #[cfg(not(feature = "std"))]
    {
        cfg!(target_feature = "neon")
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn is_x86_ssse3_available() -> bool {
    #[cfg(feature = "std")]
    {
        std::is_x86_feature_detected!("ssse3")
    }
    #[cfg(not(feature = "std"))]
    {
        cfg!(target_feature = "ssse3")
    }
}

mod scalar {
    use super::MOD;

    const NMAX: usize = 5552;

    pub fn update(mut a: u16, mut b: u16, data: &[u8]) -> (u16, u16) {
        let mut a32 = a as u32;
        let mut b32 = b as u32;

        for chunk in data.chunks(NMAX) {
            for &byte in chunk {
                a32 += byte as u32;
                b32 += a32;
            }
            a32 %= MOD;
            b32 %= MOD;
        }

        a = a32 as u16;
        b = b32 as u16;
        (a, b)
    }
}

#[cfg(target_arch = "aarch64")]
mod neon {
    use super::MOD;
    use core::arch::aarch64::*;

    const BLOCK_SIZE: usize = 32;
    const NMAX: usize = 5552;
    const CHUNK_SIZE: usize = NMAX / BLOCK_SIZE * BLOCK_SIZE; // 5536

    pub fn update(a: u16, b: u16, data: &[u8]) -> (u16, u16) {
        // For safety NEON availability checked by caller
        unsafe { update_neon(a, b, data) }
    }

    #[target_feature(enable = "neon")]
    unsafe fn update_neon(a: u16, b: u16, data: &[u8]) -> (u16, u16) {
        let mut a = a as u32;
        let mut b = b as u32;

        for chunk in data.chunks(CHUNK_SIZE) {
            update_chunk(&mut a, &mut b, chunk);
            a %= MOD;
            b %= MOD;
        }

        (a as u16, b as u16)
    }

    #[inline]
    #[target_feature(enable = "neon")]
    unsafe fn update_chunk(a: &mut u32, b: &mut u32, data: &[u8]) {
        let blocks = data.chunks_exact(BLOCK_SIZE);
        let remainder = blocks.remainder();
        let num_blocks = data.len() / BLOCK_SIZE;

        let mut a_vec = vdupq_n_u32(0); // byte sums for a
        let mut b_vec = vdupq_n_u32(0); // weighted sums for b
        let mut p = vdupq_n_u32(0); // prefix sum of a_vec across blocks

        for block in blocks {
            p = vaddq_u32(p, a_vec);

            let ptr = block.as_ptr();
            let v0 = vld1q_u8(ptr);
            let v1 = vld1q_u8(ptr.add(16));

            // Pairwise widen u8 -> u16 -> u32, accumulate into a_vec
            let sum16_0 = vpaddlq_u8(v0);
            let sum16_1 = vpaddlq_u8(v1);
            let sum32_0 = vpaddlq_u16(sum16_0);
            let sum32_1 = vpaddlq_u16(sum16_1);
            a_vec = vaddq_u32(a_vec, sum32_0);
            a_vec = vaddq_u32(a_vec, sum32_1);

            // Weighted sums: byte[i] * (32 - i) via multiply-accumulate
            let weights_hi: [u8; 16] = [
                32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17,
            ];
            let weights_lo: [u8; 16] = [16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
            let wh = vld1q_u8(weights_hi.as_ptr());
            let wl = vld1q_u8(weights_lo.as_ptr());

            // vmull: u8 * u8 -> u16 for low/high halves
            let prod0_lo = vmull_u8(vget_low_u8(v0), vget_low_u8(wh));
            let prod0_hi = vmull_u8(vget_high_u8(v0), vget_high_u8(wh));
            let prod1_lo = vmull_u8(vget_low_u8(v1), vget_low_u8(wl));
            let prod1_hi = vmull_u8(vget_high_u8(v1), vget_high_u8(wl));

            // Widen u16 -> u32 and accumulate
            b_vec = vaddq_u32(b_vec, vpaddlq_u16(prod0_lo));
            b_vec = vaddq_u32(b_vec, vpaddlq_u16(prod0_hi));
            b_vec = vaddq_u32(b_vec, vpaddlq_u16(prod1_lo));
            b_vec = vaddq_u32(b_vec, vpaddlq_u16(prod1_hi));
        }

        // Horizontal reduction
        let a_sum = vaddvq_u32(a_vec);
        let p_sum = vaddvq_u32(p);
        let b_sum = vaddvq_u32(b_vec);

        *b += *a * (num_blocks as u32 * BLOCK_SIZE as u32) + p_sum * BLOCK_SIZE as u32 + b_sum;
        *a += a_sum;

        // Scalar remainder
        for &byte in remainder {
            *a += byte as u32;
            *b += *a;
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod ssse3 {
    use super::MOD;

    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;

    const BLOCK_SIZE: usize = 32;
    const NMAX: usize = 5552;
    const CHUNK_SIZE: usize = NMAX / BLOCK_SIZE * BLOCK_SIZE;

    pub fn update(a: u16, b: u16, data: &[u8]) -> (u16, u16) {
        // For safety SSSE3 availability checked by caller
        unsafe { update_ssse3(a, b, data) }
    }

    #[target_feature(enable = "ssse3")]
    unsafe fn update_ssse3(a: u16, b: u16, data: &[u8]) -> (u16, u16) {
        let mut a = a as u32;
        let mut b = b as u32;

        for chunk in data.chunks(CHUNK_SIZE) {
            let blocks = chunk.chunks_exact(BLOCK_SIZE);
            let remainder = blocks.remainder();
            let num_blocks = chunk.len() / BLOCK_SIZE;

            let zero = _mm_setzero_si128();
            let ones_16 = _mm_set1_epi16(1);

            let weights_hi = _mm_set_epi8(
                17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
            );
            let weights_lo = _mm_set_epi8(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);

            let mut a_vec = _mm_setzero_si128();
            let mut b_vec = _mm_setzero_si128();
            let mut p = _mm_setzero_si128();

            for block in blocks {
                let ptr = block.as_ptr() as *const __m128i;

                p = _mm_add_epi32(p, a_vec);

                let v0 = _mm_loadu_si128(ptr);
                let v1 = _mm_loadu_si128(ptr.add(1));

                // Byte sums via SAD against zero
                a_vec = _mm_add_epi32(a_vec, _mm_sad_epu8(v0, zero));
                a_vec = _mm_add_epi32(a_vec, _mm_sad_epu8(v1, zero));

                // Weighted sums via maddubs (u8 * i8 -> i16) then madd (i16 -> i32)
                let mad0 = _mm_maddubs_epi16(v0, weights_hi);
                let mad1 = _mm_maddubs_epi16(v1, weights_lo);
                b_vec = _mm_add_epi32(b_vec, _mm_madd_epi16(mad0, ones_16));
                b_vec = _mm_add_epi32(b_vec, _mm_madd_epi16(mad1, ones_16));
            }

            let a_sum = hsum_i32(a_vec);
            let p_sum = hsum_i32(p);
            let b_sum = hsum_i32(b_vec);

            b += a * (num_blocks as u32 * BLOCK_SIZE as u32)
                + p_sum as u32 * BLOCK_SIZE as u32
                + b_sum as u32;
            a += a_sum as u32;

            for &byte in remainder {
                a += byte as u32;
                b += a;
            }

            a %= MOD;
            b %= MOD;
        }

        (a as u16, b as u16)
    }

    #[inline]
    #[target_feature(enable = "ssse3")]
    unsafe fn hsum_i32(v: __m128i) -> i32 {
        let hi = _mm_unpackhi_epi64(v, v);
        let sum = _mm_add_epi32(v, hi);
        let hi = _mm_shuffle_epi32(sum, 0b_00_00_00_01);
        let sum = _mm_add_epi32(sum, hi);
        _mm_cvtsi128_si32(sum)
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::vec;
    use alloc::vec::Vec;

    use super::*;

    fn reference_adler32(data: &[u8]) -> u32 {
        let mut a: u32 = 1;
        let mut b: u32 = 0;
        for &byte in data {
            a = (a + byte as u32) % MOD;
            b = (b + a) % MOD;
        }
        (b << 16) | a
    }

    #[test]
    fn empty() {
        assert_eq!(adler32(&[]), reference_adler32(&[]));
    }

    #[test]
    fn single_byte() {
        assert_eq!(adler32(&[1]), reference_adler32(&[1]));
        assert_eq!(adler32(&[0xff]), reference_adler32(&[0xff]));
    }

    #[test]
    fn wikipedia_example() {
        let data = b"Wikipedia";
        assert_eq!(adler32(data), reference_adler32(data));
        assert_eq!(adler32(data), 0x11E60398);
    }

    #[test]
    fn small_data() {
        for len in 0..=128 {
            let data: Vec<u8> = (0..len).map(|i| (i & 0xff) as u8).collect();
            assert_eq!(
                adler32(&data),
                reference_adler32(&data),
                "mismatch at len={}",
                len
            );
        }
    }

    #[test]
    fn block_boundaries() {
        for &len in &[31, 32, 33, 63, 64, 65, 5551, 5552, 5553, 5600, 11104] {
            let data: Vec<u8> = (0..len).map(|i| ((i * 7 + 13) & 0xff) as u8).collect();
            assert_eq!(
                adler32(&data),
                reference_adler32(&data),
                "mismatch at len={}",
                len
            );
        }
    }

    #[test]
    fn all_zeros() {
        let data = vec![0u8; 100_000];
        assert_eq!(adler32(&data), reference_adler32(&data));
    }

    #[test]
    fn all_ones() {
        let data = vec![1u8; 100_000];
        assert_eq!(adler32(&data), reference_adler32(&data));
    }

    #[test]
    fn all_0xff() {
        let data = vec![0xffu8; 100_000];
        assert_eq!(adler32(&data), reference_adler32(&data));
    }

    #[test]
    fn incremental_matches_oneshot() {
        let data: Vec<u8> = (0..10_000).map(|i| (i & 0xff) as u8).collect();
        let oneshot = adler32(&data);

        let mut h = Adler32::new();
        let mut offset = 0;
        for chunk_size in [1, 7, 13, 31, 32, 33, 64, 100, 1000] {
            let end = (offset + chunk_size).min(data.len());
            if offset < end {
                h.write(&data[offset..end]);
                offset = end;
            }
        }
        h.write(&data[offset..]);

        assert_eq!(h.checksum(), oneshot);
    }
}
