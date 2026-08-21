//! Adaptive multimodular basis.
//!
//! Where the original engine used a *fixed* 32-prime channel set (a fixed dynamic
//! range that silently aliased on overflow), `Basis` provisions **as many primes
//! as the computation's height demands** (see [`crate::bounds`]), then CRT +
//! rational-reconstructs once at the boundary.
//!
//! All primes live in the open interval `(2^15, 2^16)`. That makes them
//! GPU-eligible: the product of two residues is `< 2^32`, so the naive WGSL
//! `(a*b) % m` is safe, and each channel contributes ~16 bits of range.

use std::sync::{Arc, OnceLock};

use num_bigint::{BigInt, BigUint};
use num_traits::One;

use crate::primes::primes_up_to;

/// Lower bound (exclusive) for basis primes: `2^15`.
pub const PRIME_MIN: u32 = 1 << 15;
/// Upper bound (exclusive) for basis primes: `2^16`.
pub const PRIME_MAX: u32 = 1 << 16;

/// All GPU-eligible primes in `(2^15, 2^16)`, computed once and shared.
fn gpu_primes() -> &'static Vec<u32> {
    static PRIMES: OnceLock<Vec<u32>> = OnceLock::new();
    PRIMES.get_or_init(|| {
        primes_up_to((PRIME_MAX - 1) as usize)
            .into_iter()
            .map(|p| p as u32)
            .filter(|&p| p > PRIME_MIN)
            .collect()
    })
}

/// An ordered, extensible set of pairwise-coprime moduli.
#[derive(Clone, Debug)]
pub struct Basis {
    primes: Arc<Vec<u32>>,
}

impl Basis {
    /// Build directly from a prime count (mainly for tests / fixed-size needs).
    pub fn from_count(n: usize) -> Self {
        let all = gpu_primes();
        assert!(n <= all.len(), "requested more primes than are GPU-eligible");
        Basis { primes: Arc::new(all[..n].to_vec()) }
    }

    /// A general-purpose default basis (~512 bits ≈ 32 channels). Useful for
    /// open-ended Level 0/1 work and as the seed the multimodular pipeline grows.
    pub fn standard() -> Self {
        Self::with_bits(512)
    }

    /// The smallest basis whose product exceeds `2^bits`.
    pub fn with_bits(bits: u64) -> Self {
        let all = gpu_primes();
        let mut product = BigUint::one();
        let target = BigUint::one() << bits as usize;
        let mut take = 0;
        for &p in all.iter() {
            if product > target {
                break;
            }
            product *= BigUint::from(p);
            take += 1;
        }
        Basis { primes: Arc::new(all[..take].to_vec()) }
    }

    /// Number of channels.
    #[inline]
    pub fn len(&self) -> usize {
        self.primes.len()
    }

    /// Whether the basis is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.primes.is_empty()
    }

    /// The modulus of channel `i`.
    #[inline]
    pub fn modulus(&self, i: usize) -> u32 {
        self.primes[i]
    }

    /// All moduli as a slice.
    #[inline]
    pub fn moduli(&self) -> &[u32] {
        &self.primes
    }

    /// `floor(log2(∏ primes))` — the usable bit capacity.
    pub fn capacity_bits(&self) -> u64 {
        let product = self.modulus_product();
        if product <= BigUint::one() {
            0
        } else {
            product.bits() - 1
        }
    }

    /// The product `M = ∏ primes`.
    pub fn modulus_product(&self) -> BigUint {
        self.primes.iter().map(|&p| BigUint::from(p)).product()
    }

    /// Alias for [`Basis::modulus_product`] — total dynamic range `M`.
    pub fn capacity(&self) -> BigUint {
        self.modulus_product()
    }

    /// Signed range bound `⌊M/2⌋`: balanced values live in `(-bound, bound]`.
    pub fn signed_capacity(&self) -> BigInt {
        BigInt::from(self.modulus_product() / BigUint::from(2u8))
    }

    /// A basis with at least `bits` of capacity (extends with more primes if
    /// needed; the returned basis shares no state but is cheap to build).
    pub fn extend_to_bits(&self, bits: u64) -> Basis {
        if self.capacity_bits() >= bits {
            return self.clone();
        }
        Basis::with_bits(bits)
    }
}

impl PartialEq for Basis {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.primes, &other.primes) || self.primes == other.primes
    }
}
impl Eq for Basis {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primes_are_gpu_eligible() {
        let b = Basis::from_count(10);
        for i in 0..b.len() {
            let p = b.modulus(i);
            assert!(p > PRIME_MIN && p < PRIME_MAX);
        }
    }

    #[test]
    fn with_bits_exceeds_target() {
        let b = Basis::with_bits(166); // ~10^50
        assert!(b.capacity_bits() >= 166);
        // each prime is ~16 bits, so ~11 channels
        assert!(b.len() <= 12 && b.len() >= 10);
    }

    #[test]
    fn extend_grows() {
        let small = Basis::with_bits(50);
        let big = small.extend_to_bits(500);
        assert!(big.capacity_bits() >= 500);
        assert!(big.len() > small.len());
    }
}
