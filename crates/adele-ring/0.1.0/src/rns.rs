//! Level 0 — ℤ. RNS integers and the core CRT reconstruction.
//!
//! # Representation
//!
//! A number lives as a tuple of residues, one per prime channel. By the Chinese
//! Remainder Theorem any integer in `[0, M)` (where `M = ∏ moduli`) is uniquely
//! identified by its residue tuple. We use the **symmetric** residue system:
//! residues are stored in `[0, m)`, and on reconstruction a value `U` with
//! `2U > M` is interpreted as the negative number `U - M`. This lets the signed
//! range be `(-M/2, M/2)` while keeping per-channel arithmetic a pure modular op
//! — exactly the property that makes RNS embarrassingly parallel.
//!
//! # CPU vs GPU
//!
//! Single `RnsInt` operations parallelize over channels with rayon only when
//! `k >= RAYON_CHANNEL_THRESHOLD`; below that the task overhead dominates and we
//! stay sequential. For *batches* of `RnsInt`, always go through
//! [`crate::backend::executor`], which auto-selects CPU rayon or the GPU based on
//! the batch size. Never call a backend directly from the math layers.

use std::sync::Arc;

use num_bigint::{BigInt, BigUint, Sign};
use num_traits::Zero;

use crate::primes::{first_n_primes, gcd, mod_inverse};
use crate::RAYON_CHANNEL_THRESHOLD;

/// A shared, immutable set of pairwise-coprime moduli (the RNS "channels").
///
/// Cheap to clone — it is just an `Arc<Vec<u64>>` behind a newtype.
#[derive(Clone, Debug)]
pub struct Channels(pub Arc<Vec<u64>>);

impl Channels {
    /// Build channels from explicit moduli.
    ///
    /// In debug builds this asserts the moduli are pairwise coprime (the CRT
    /// requirement); in release builds the check is skipped for speed.
    pub fn new(moduli: Vec<u64>) -> Self {
        debug_assert!(
            Self::pairwise_coprime(&moduli),
            "RNS channels must be pairwise coprime"
        );
        Channels(Arc::new(moduli))
    }

    /// The first `n` primes as channels — the standard configuration.
    pub fn standard(n: usize) -> Self {
        Channels(Arc::new(first_n_primes(n)))
    }

    fn pairwise_coprime(moduli: &[u64]) -> bool {
        for i in 0..moduli.len() {
            for j in (i + 1)..moduli.len() {
                if gcd(moduli[i], moduli[j]) != 1 {
                    return false;
                }
            }
        }
        true
    }

    /// Number of channels `k`.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether there are no channels.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The modulus of channel `c`.
    #[inline]
    pub fn modulus(&self, c: usize) -> u64 {
        self.0[c]
    }

    /// The moduli as a slice.
    #[inline]
    pub fn moduli(&self) -> &[u64] {
        &self.0
    }

    /// Total dynamic range `M = ∏ moduli`.
    pub fn capacity(&self) -> BigUint {
        self.0.iter().map(|&m| BigUint::from(m)).product()
    }

    /// Signed range bound `⌊M/2⌋`: values in `(-bound, bound]` are representable.
    pub fn signed_capacity(&self) -> BigInt {
        BigInt::from(self.capacity() / BigUint::from(2u8))
    }
}

impl PartialEq for Channels {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.0 == other.0
    }
}
impl Eq for Channels {}

/// An exact integer in RNS form (Level 0 of the tower).
#[derive(Clone, Debug)]
pub struct RnsInt {
    /// `residues[i] = value mod channels[i]`, stored in `[0, m)`.
    pub residues: Vec<u64>,
    pub channels: Channels,
    /// Sign hint: `true` when the represented (symmetric) value is negative.
    pub negative: bool,
}

impl RnsInt {
    /// Construct from an arbitrary `BigInt`.
    pub fn from_bigint(n: &BigInt, channels: Channels) -> Self {
        let negative = n.sign() == Sign::Minus;
        let residues = channels
            .moduli()
            .iter()
            .map(|&m| {
                let mm = BigInt::from(m);
                // Euclidean remainder in [0, m).
                let r = ((n % &mm) + &mm) % &mm;
                r.to_biguint().unwrap().try_into().unwrap()
            })
            .collect();
        RnsInt {
            residues,
            channels,
            negative,
        }
    }

    /// Construct from a machine integer.
    pub fn from_i64(n: i64, channels: Channels) -> Self {
        Self::from_bigint(&BigInt::from(n), channels)
    }

    /// Additive identity.
    pub fn zero(channels: Channels) -> Self {
        RnsInt {
            residues: vec![0; channels.len()],
            channels,
            negative: false,
        }
    }

    /// Build directly from raw channel residues, recomputing the sign hint.
    /// The residues must already be reduced into `[0, m)` for each channel.
    pub fn from_residues(residues: Vec<u64>, channels: Channels) -> Self {
        Self::finish(residues, channels)
    }

    /// Reconstruct the exact signed value via Garner CRT + symmetric folding.
    pub fn to_bigint(&self) -> BigInt {
        let u = garner_crt(&self.residues, self.channels.moduli());
        let m = self.channels.capacity();
        // Symmetric range: if 2u > M, the value is u - M (negative).
        if &u * 2u8 > m {
            BigInt::from_biguint(Sign::Plus, u) - BigInt::from_biguint(Sign::Plus, m)
        } else {
            BigInt::from_biguint(Sign::Plus, u)
        }
    }

    /// `true` iff every residue is zero.
    pub fn is_zero(&self) -> bool {
        self.residues.iter().all(|&r| r == 0)
    }

    /// Channel-wise modular addition.
    pub fn add(&self, other: &Self) -> Self {
        let out = channel_map(&self.residues, &other.residues, self.channels.moduli(), |a, b, m| {
            (a + b) % m
        });
        Self::finish(out, self.channels.clone())
    }

    /// Channel-wise modular subtraction.
    pub fn sub(&self, other: &Self) -> Self {
        let out = channel_map(&self.residues, &other.residues, self.channels.moduli(), |a, b, m| {
            (a + m - b) % m
        });
        Self::finish(out, self.channels.clone())
    }

    /// Channel-wise modular multiplication (uses `u128` intermediate).
    pub fn mul(&self, other: &Self) -> Self {
        let out = channel_map(&self.residues, &other.residues, self.channels.moduli(), |a, b, m| {
            gpu_mul_channel(a, b, m)
        });
        Self::finish(out, self.channels.clone())
    }

    /// Additive inverse.
    pub fn neg(&self) -> Self {
        Self::zero(self.channels.clone()).sub(self)
    }

    /// Build from raw residues and recompute the sign hint.
    fn finish(residues: Vec<u64>, channels: Channels) -> Self {
        let mut v = RnsInt {
            residues,
            channels,
            negative: false,
        };
        v.negative = v.to_bigint().sign() == Sign::Minus;
        v
    }
}

/// Apply `f(a, b, m)` channel-wise, parallelizing only above the threshold.
fn channel_map(
    a: &[u64],
    b: &[u64],
    moduli: &[u64],
    f: impl Fn(u64, u64, u64) -> u64 + Sync + Send,
) -> Vec<u64> {
    use rayon::prelude::*;
    if a.len() >= RAYON_CHANNEL_THRESHOLD {
        a.par_iter()
            .zip(b.par_iter())
            .zip(moduli.par_iter())
            .map(|((&av, &bv), &m)| f(av, bv, m))
            .collect()
    } else {
        a.iter()
            .zip(b.iter())
            .zip(moduli.iter())
            .map(|((&av, &bv), &m)| f(av, bv, m))
            .collect()
    }
}

/// Garner's algorithm: reconstruct the unsigned integer in `[0, M)` from its
/// residues. Never materializes the large basis elements `M/m_i`; all
/// intermediates stay small (each fits within its own modulus).
pub fn garner_crt(residues: &[u64], moduli: &[u64]) -> BigUint {
    let k = residues.len();
    assert_eq!(k, moduli.len(), "residue/moduli length mismatch");
    if k == 0 {
        return BigUint::zero();
    }

    // Step 1 — mixed-radix coefficients via forward substitution.
    let mut c: Vec<u64> = residues.to_vec();
    for i in 0..k {
        for j in 0..i {
            let mi = moduli[i];
            // c[i] = (c[i] - c[j]) * inv(m[j], m[i])  (mod m[i])
            let inv = mod_inverse(moduli[j] % mi, mi)
                .expect("channels must be pairwise coprime for CRT");
            let diff = (c[i] + mi - (c[j] % mi)) % mi;
            c[i] = ((diff as u128 * inv as u128) % mi as u128) as u64;
        }
    }

    // Step 2 — Horner reconstruction into a single BigUint.
    let mut result = BigUint::from(c[k - 1]);
    for i in (0..k - 1).rev() {
        result = result * BigUint::from(moduli[i]) + BigUint::from(c[i]);
    }
    result
}

/// Reference implementation of one GPU thread's add: `(a + b) % m`.
#[inline]
pub fn gpu_add_channel(a: u64, b: u64, m: u64) -> u64 {
    (a + b) % m
}

/// Reference implementation of one GPU thread's multiply: `(a * b) % m`.
#[inline]
pub fn gpu_mul_channel(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 * b as u128) % m as u128) as u64
}

/// Largest modulus for which `(a*b)` fits in `u32` on the GPU (`< 2^16`).
pub const MAX_SAFE_MODULUS: u64 = 65535;

#[cfg(test)]
mod tests {
    use super::*;

    fn ch() -> Channels {
        Channels::standard(16)
    }

    #[test]
    fn roundtrip_positive() {
        let a = RnsInt::from_i64(123_456_789, ch());
        assert_eq!(a.to_bigint(), BigInt::from(123_456_789));
    }

    #[test]
    fn roundtrip_negative() {
        let a = RnsInt::from_i64(-42, ch());
        assert!(a.negative);
        assert_eq!(a.to_bigint(), BigInt::from(-42));
    }

    #[test]
    fn add_sub_mul() {
        let a = RnsInt::from_i64(1000, ch());
        let b = RnsInt::from_i64(337, ch());
        assert_eq!(a.add(&b).to_bigint(), BigInt::from(1337));
        assert_eq!(a.sub(&b).to_bigint(), BigInt::from(663));
        assert_eq!(b.sub(&a).to_bigint(), BigInt::from(-663));
        assert_eq!(a.mul(&b).to_bigint(), BigInt::from(337_000));
    }

    #[test]
    fn garner_classic() {
        // x ≡ 2 (3), 3 (5), 2 (7)  =>  x = 23
        assert_eq!(garner_crt(&[2, 3, 2], &[3, 5, 7]), BigUint::from(23u8));
        // x ≡ 0 (2), 1 (3), 0 (5)  =>  x = 10
        assert_eq!(garner_crt(&[0, 1, 0], &[2, 3, 5]), BigUint::from(10u8));
    }

    #[test]
    fn is_zero_works() {
        assert!(RnsInt::zero(ch()).is_zero());
        assert!(!RnsInt::from_i64(1, ch()).is_zero());
    }
}
