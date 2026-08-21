//! Level 0 — ℤ. RNS integers over the adaptive [`Basis`], and Garner CRT.
//!
//! # Balanced residues (no sign-magnitude)
//!
//! Residues are stored canonically in `[0, p)` for each prime `p`, but the value
//! they encode is the **balanced** (symmetric) integer in `(-M/2, M/2]`, where
//! `M = ∏ p`. Reconstruction folds `u ∈ [0, M)` to `u - M` when `2u > M`.
//!
//! There is **no `negative` flag**: subtraction is the channel-parallel
//! `(a + p - b) % p`, with no magnitude comparison and no `BigInt` in the loop.
//! The *sign* of a value is an Archimedean question — it is answered by the
//! [`crate::ball::Ball`] in the [`crate::adelic::Adelic`] carrier, never by the
//! residues (which are constitutionally blind to size).
//!
//! All primes are GPU-eligible (`(2^15, 2^16)`), so every channel op — add, sub,
//! and even `(a*b)` — fits in a `u32` with no `u128` intermediate.

use num_bigint::{BigInt, BigUint, Sign};
use num_traits::Zero;

use crate::basis::Basis;
use crate::primes::mod_inverse;
use crate::RAYON_CHANNEL_THRESHOLD;

/// An exact integer in balanced RNS form (Level 0 of the tower).
#[derive(Clone, Debug)]
pub struct RnsInt {
    /// `residues[i] = value mod basis[i]`, stored canonically in `[0, p)`.
    pub residues: Vec<u32>,
    pub basis: Basis,
}

impl RnsInt {
    /// Construct from an arbitrary `BigInt` (reduced into each channel).
    pub fn from_bigint(n: &BigInt, basis: Basis) -> Self {
        let residues = basis
            .moduli()
            .iter()
            .map(|&m| {
                let mm = BigInt::from(m);
                let r = ((n % &mm) + &mm) % &mm;
                r.to_biguint().unwrap().try_into().unwrap()
            })
            .collect();
        RnsInt { residues, basis }
    }

    /// Construct from a machine integer.
    pub fn from_i64(n: i64, basis: Basis) -> Self {
        Self::from_bigint(&BigInt::from(n), basis)
    }

    /// Additive identity.
    pub fn zero(basis: Basis) -> Self {
        RnsInt { residues: vec![0; basis.len()], basis }
    }

    /// Build directly from raw channel residues (already reduced into `[0, p)`).
    pub fn from_residues(residues: Vec<u32>, basis: Basis) -> Self {
        RnsInt { residues, basis }
    }

    /// Reconstruct the exact signed value via Garner CRT + balanced folding.
    pub fn to_bigint(&self) -> BigInt {
        let u = garner_crt(&self.residues, self.basis.moduli());
        let m = self.basis.modulus_product();
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
        let out = channel_map(&self.residues, &other.residues, self.basis.moduli(), add_channel);
        Self::from_residues(out, self.basis.clone())
    }

    /// Channel-wise modular subtraction (`(a + p - b) % p`, no sign-magnitude).
    pub fn sub(&self, other: &Self) -> Self {
        let out = channel_map(&self.residues, &other.residues, self.basis.moduli(), sub_channel);
        Self::from_residues(out, self.basis.clone())
    }

    /// Channel-wise modular multiplication.
    pub fn mul(&self, other: &Self) -> Self {
        let out = channel_map(&self.residues, &other.residues, self.basis.moduli(), mul_channel);
        Self::from_residues(out, self.basis.clone())
    }

    /// Additive inverse (channel-wise `(p - r) % p`).
    pub fn neg(&self) -> Self {
        let out: Vec<u32> = self
            .residues
            .iter()
            .zip(self.basis.moduli())
            .map(|(&r, &m)| (m - r) % m)
            .collect();
        Self::from_residues(out, self.basis.clone())
    }
}

/// Apply `f(a, b, m)` channel-wise, parallelizing only above the threshold.
fn channel_map(
    a: &[u32],
    b: &[u32],
    moduli: &[u32],
    f: impl Fn(u32, u32, u32) -> u32 + Sync + Send,
) -> Vec<u32> {
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

/// One channel's add: `(a + b) % m`. Fits `u32` (`a, b < 2^16`).
#[inline]
pub fn add_channel(a: u32, b: u32, m: u32) -> u32 {
    (a + b) % m
}

/// One channel's subtract: `(a + m - b) % m`. No magnitude comparison.
#[inline]
pub fn sub_channel(a: u32, b: u32, m: u32) -> u32 {
    (a + m - b) % m
}

/// One channel's multiply: `(a * b) % m`. Fits `u32` (`a, b < 2^16`).
#[inline]
pub fn mul_channel(a: u32, b: u32, m: u32) -> u32 {
    (a * b) % m
}

/// Garner's algorithm: reconstruct the unsigned integer in `[0, M)` from its
/// balanced-canonical residues. All intermediates stay within `u64`.
///
/// The mixed-radix step uses the **underflow-safe** form
/// `c[i] = ((c[i] + m_i - (c[j] % m_i)) % m_i) * inv % m_i`.
pub fn garner_crt(residues: &[u32], moduli: &[u32]) -> BigUint {
    let k = residues.len();
    assert_eq!(k, moduli.len(), "residue/moduli length mismatch");
    if k == 0 {
        return BigUint::zero();
    }

    let mut c: Vec<u64> = residues.iter().map(|&r| r as u64).collect();
    for i in 0..k {
        let mi = moduli[i] as u64;
        for j in 0..i {
            let inv = mod_inverse(moduli[j] as u64 % mi, mi)
                .expect("basis primes must be pairwise coprime for CRT");
            let diff = (c[i] + mi - (c[j] % mi)) % mi;
            c[i] = (diff * inv) % mi;
        }
    }

    let mut result = BigUint::from(c[k - 1]);
    for i in (0..k - 1).rev() {
        result = result * BigUint::from(moduli[i]) + BigUint::from(c[i]);
    }
    result
}

/// Garner CRT followed by balanced (symmetric) folding into `(-M/2, M/2]`.
pub fn crt_balanced(residues: &[u32], moduli: &[u32]) -> BigInt {
    let u = garner_crt(residues, moduli);
    let m: BigUint = moduli.iter().map(|&p| BigUint::from(p)).product();
    if &u * 2u8 > m {
        BigInt::from_biguint(Sign::Plus, u) - BigInt::from_biguint(Sign::Plus, m)
    } else {
        BigInt::from_biguint(Sign::Plus, u)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b() -> Basis {
        Basis::standard()
    }

    #[test]
    fn roundtrip_positive() {
        let a = RnsInt::from_i64(123_456_789, b());
        assert_eq!(a.to_bigint(), BigInt::from(123_456_789));
    }

    #[test]
    fn roundtrip_negative() {
        let a = RnsInt::from_i64(-42, b());
        assert_eq!(a.to_bigint(), BigInt::from(-42));
    }

    #[test]
    fn add_sub_mul() {
        let a = RnsInt::from_i64(1000, b());
        let bb = RnsInt::from_i64(337, b());
        assert_eq!(a.add(&bb).to_bigint(), BigInt::from(1337));
        assert_eq!(a.sub(&bb).to_bigint(), BigInt::from(663));
        assert_eq!(bb.sub(&a).to_bigint(), BigInt::from(-663));
        assert_eq!(a.mul(&bb).to_bigint(), BigInt::from(337_000));
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
        assert!(RnsInt::zero(b()).is_zero());
        assert!(!RnsInt::from_i64(1, b()).is_zero());
    }
}
