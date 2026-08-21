//! The adelic carrier — a value living at **both** kinds of place at once.
//!
//! ```text
//! 𝔸_ℚ  =  ℝ  ×  ∏′_p ℚ_p
//!         ▲            ▲
//!    infinite place    finite places
//! ```
//!
//! An [`Adelic`] pairs a **finite** component (pure RNS over a [`Basis`] — the
//! carry-free, embarrassingly parallel arithmetic) with an **infinite**
//! component (a rigorous [`Ball`] — the real interval that answers every
//! Archimedean question: sign, comparison, magnitude, decimal output).
//!
//! - Exactness comes from the finite part (reconstructed at the boundary).
//! - Sign / order / output come from the infinite part — never from the
//!   residues, which are constitutionally blind to size.
//!
//! Overflow is a *detected event*: when the finite reconstruction cannot be
//! validated against the infinite ball, [`RangeError::ReconstructionFailed`] is
//! returned and the caller extends the basis (see the headline test in the
//! refactor plan §11).

use std::cmp::Ordering;

use num_bigint::{BigInt, Sign};
use num_rational::BigRational;
use num_traits::One;

use crate::ball::Ball;
use crate::basis::Basis;
use crate::error::RangeError;
use crate::primes::mod_inverse;
use crate::reconstruct::rational_reconstruct;
use crate::rns::RnsInt;

/// The finite (∏ ℚ_p) component of an adelic value: RNS arithmetic plus a
/// reconstruction routine that may *detect* range overflow.
pub trait Finite: Clone {
    fn basis(&self) -> &Basis;
    fn add(&self, o: &Self) -> Self;
    fn sub(&self, o: &Self) -> Self;
    fn mul(&self, o: &Self) -> Self;
    /// Reconstruct the exact `(p, q)` (with `q == 1` for integers), or `None`
    /// when the value cannot be recovered within the current basis range.
    fn try_reconstruct(&self) -> Option<(BigInt, BigInt)>;
    /// Re-encode a known exact `p/q` over a (possibly larger) basis.
    fn reimage(p: &BigInt, q: &BigInt, basis: &Basis) -> Self;
}

// ── Finite: integers ─────────────────────────────────────────────────────────

impl Finite for RnsInt {
    fn basis(&self) -> &Basis {
        &self.basis
    }
    fn add(&self, o: &Self) -> Self {
        RnsInt::add(self, o)
    }
    fn sub(&self, o: &Self) -> Self {
        RnsInt::sub(self, o)
    }
    fn mul(&self, o: &Self) -> Self {
        RnsInt::mul(self, o)
    }
    fn try_reconstruct(&self) -> Option<(BigInt, BigInt)> {
        // Balanced lift always "succeeds" numerically; the Adelic layer decides
        // whether it is *correct* by validating against the infinite ball.
        Some((self.to_bigint(), BigInt::one()))
    }
    fn reimage(p: &BigInt, q: &BigInt, basis: &Basis) -> Self {
        debug_assert!(q.is_one(), "integer reimage with non-unit denominator");
        RnsInt::from_bigint(p, basis.clone())
    }
}

// ── Finite: modular fractions (Level 1, single residue per channel) ──────────

/// A rational carried as one field element `p·q⁻¹ (mod pᵢ)` per channel.
///
/// A channel whose prime divides the denominator is **invalid** for this value
/// (the fraction is not p-adically integral there); it is masked out of *this
/// value's* reconstruction. This per-value mask is correctness bookkeeping for
/// one number — never the unsound global "skip channels to save power" idea.
#[derive(Clone, Debug)]
pub struct RnsFrac {
    /// `None` marks a channel invalid for this value (prime divides denom).
    pub residues: Vec<Option<u32>>,
    pub basis: Basis,
}

impl RnsFrac {
    pub fn from_fraction(p: &BigInt, q: &BigInt, basis: Basis) -> Self {
        let residues = basis
            .moduli()
            .iter()
            .map(|&m| {
                let mm = BigInt::from(m);
                let pm = (((p % &mm) + &mm) % &mm).to_u32();
                let qm = (((q % &mm) + &mm) % &mm).to_u32();
                match (pm, qm) {
                    (Some(pr), Some(qr)) if qr != 0 => {
                        let inv = mod_inverse(qr as u64, m as u64)? as u32;
                        Some(((pr as u64 * inv as u64) % m as u64) as u32)
                    }
                    _ => None,
                }
            })
            .collect();
        RnsFrac { residues, basis }
    }

    fn combine(&self, o: &Self, f: impl Fn(u32, u32, u32) -> u32) -> Self {
        let residues = self
            .residues
            .iter()
            .zip(&o.residues)
            .zip(self.basis.moduli())
            .map(|((a, b), &m)| match (a, b) {
                (Some(x), Some(y)) => Some(f(*x, *y, m)),
                _ => None,
            })
            .collect();
        RnsFrac { residues, basis: self.basis.clone() }
    }
}

trait ToU32 {
    fn to_u32(&self) -> Option<u32>;
}
impl ToU32 for BigInt {
    fn to_u32(&self) -> Option<u32> {
        use num_traits::ToPrimitive;
        ToPrimitive::to_u32(self)
    }
}

impl Finite for RnsFrac {
    fn basis(&self) -> &Basis {
        &self.basis
    }
    fn add(&self, o: &Self) -> Self {
        self.combine(o, crate::rns::add_channel)
    }
    fn sub(&self, o: &Self) -> Self {
        self.combine(o, crate::rns::sub_channel)
    }
    fn mul(&self, o: &Self) -> Self {
        self.combine(o, crate::rns::mul_channel)
    }
    fn try_reconstruct(&self) -> Option<(BigInt, BigInt)> {
        let mut res = Vec::new();
        let mut mods = Vec::new();
        for (r, &m) in self.residues.iter().zip(self.basis.moduli()) {
            if let Some(v) = r {
                res.push(*v);
                mods.push(m);
            }
        }
        if mods.is_empty() {
            return None;
        }
        let u = crate::rns::garner_crt(&res, &mods);
        let m_prod: num_bigint::BigUint = mods.iter().map(|&p| num_bigint::BigUint::from(p)).product();
        rational_reconstruct(&u, &m_prod)
    }
    fn reimage(p: &BigInt, q: &BigInt, basis: &Basis) -> Self {
        RnsFrac::from_fraction(p, q, basis.clone())
    }
}

// ── The carrier ──────────────────────────────────────────────────────────────

/// A value carried at the finite places (`F`) and the infinite place (`Ball`).
#[derive(Clone)]
pub struct Adelic<F: Finite> {
    finite: F,
    infinite: Ball,
}

impl<F: Finite> Adelic<F> {
    /// Assemble from a finite component and its real enclosure.
    pub fn from_parts(finite: F, infinite: Ball) -> Self {
        Adelic { finite, infinite }
    }

    /// The infinite (real) component.
    pub fn ball(&self) -> &Ball {
        &self.infinite
    }

    /// The finite (RNS) component.
    pub fn finite(&self) -> &F {
        &self.finite
    }

    /// Sign, from the infinite place (the only place that can see it).
    pub fn sign(&self) -> Option<Sign> {
        self.infinite.sign()
    }

    /// Ordering, from the infinite place (`None` ⇒ refine).
    pub fn cmp(&self, other: &Self) -> Option<Ordering> {
        self.infinite.cmp(&other.infinite)
    }

    /// Nearest `f64`.
    pub fn to_f64(&self) -> f64 {
        self.infinite.to_f64()
    }

    /// Addition (updates both places).
    pub fn add(&self, o: &Self) -> Self {
        Adelic { finite: self.finite.add(&o.finite), infinite: self.infinite.add(&o.infinite) }
    }

    /// Subtraction (updates both places).
    pub fn sub(&self, o: &Self) -> Self {
        Adelic { finite: self.finite.sub(&o.finite), infinite: self.infinite.sub(&o.infinite) }
    }

    /// Multiplication (updates both places).
    pub fn mul(&self, o: &Self) -> Self {
        Adelic { finite: self.finite.mul(&o.finite), infinite: self.infinite.mul(&o.infinite) }
    }

    /// The exact value, if the infinite place pins it to a point.
    fn exact_point(&self) -> Option<(BigInt, BigInt)> {
        if self.infinite.lo == self.infinite.hi {
            Some((self.infinite.lo.numer().clone(), self.infinite.lo.denom().clone()))
        } else {
            None
        }
    }

    /// Re-encode the finite part over a (possibly larger) basis. Requires the
    /// infinite place to pin the exact value (true for all integer values and
    /// for any width-0 ball).
    pub fn with_basis(&self, basis: Basis) -> Self {
        let (p, q) = self
            .exact_point()
            .expect("with_basis requires a point-valued infinite place");
        Adelic { finite: F::reimage(&p, &q, &basis), infinite: self.infinite.clone() }
    }

    /// Validate a reconstructed `p/q` against the infinite ball.
    fn validate(&self, p: &BigInt, q: &BigInt) -> Result<(BigInt, BigInt), RangeError> {
        let candidate = BigRational::new(p.clone(), q.clone());
        if self.infinite.contains(&candidate) {
            Ok((p.clone(), q.clone()))
        } else {
            let have_bits = self.finite.basis().capacity_bits();
            Err(RangeError::ReconstructionFailed { have_bits, need_more: true })
        }
    }
}

/// Integer instantiation.
pub type AdelicInt = Adelic<RnsInt>;
/// Rational instantiation.
pub type AdelicRat = Adelic<RnsFrac>;

impl AdelicInt {
    /// From an exact `BigInt` over `basis` (the infinite place pins the value).
    pub fn from_bigint(n: &BigInt, basis: Basis) -> Self {
        Adelic {
            finite: RnsInt::from_bigint(n, basis),
            infinite: Ball::point(&BigRational::from_integer(n.clone())),
        }
    }

    /// From a machine integer.
    pub fn from_i64(n: i64, basis: Basis) -> Self {
        Self::from_bigint(&BigInt::from(n), basis)
    }

    /// The exact integer, or [`RangeError::ReconstructionFailed`] if the finite
    /// reconstruction disagrees with the infinite place (basis too small).
    pub fn try_exact(&self) -> Result<BigInt, RangeError> {
        let (p, q) = self
            .finite
            .try_reconstruct()
            .ok_or(RangeError::ReconstructionFailed {
                have_bits: self.finite.basis().capacity_bits(),
                need_more: true,
            })?;
        self.validate(&p, &q).map(|(p, _)| p)
    }
}

impl AdelicRat {
    /// From an exact reduced `p/q` over `basis`.
    pub fn from_fraction(p: &BigInt, q: &BigInt, basis: Basis) -> Self {
        Adelic {
            finite: RnsFrac::from_fraction(p, q, basis),
            infinite: Ball::point(&BigRational::new(p.clone(), q.clone())),
        }
    }

    /// The exact `(p, q)`, or a detected reconstruction failure.
    pub fn try_exact(&self) -> Result<(BigInt, BigInt), RangeError> {
        let (p, q) = self
            .finite
            .try_reconstruct()
            .ok_or(RangeError::ReconstructionFailed {
                have_bits: self.finite.basis().capacity_bits(),
                need_more: true,
            })?;
        self.validate(&p, &q)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Headline regression test (refactor plan §11): overflow is DETECTED.
    #[test]
    fn overflow_is_detected_then_fixed() {
        let small = Basis::with_bits(40);
        let big = AdelicInt::from_bigint(&(BigInt::one() << 60), small.clone());
        assert!(matches!(big.try_exact(), Err(RangeError::ReconstructionFailed { .. })));

        let grown = big.with_basis(small.extend_to_bits(80));
        assert_eq!(grown.try_exact().unwrap(), BigInt::one() << 60);
    }

    #[test]
    fn sign_from_infinite_place_with_mixed_signs() {
        let b = Basis::standard();
        let a = AdelicInt::from_i64(-7, b.clone());
        let c = AdelicInt::from_i64(5, b);
        assert_eq!(a.add(&c).sign(), Some(Sign::Minus)); // -2
        assert_eq!(a.sub(&c).sign(), Some(Sign::Minus)); // -12 (batch_sub path)
    }

    #[test]
    fn integer_arithmetic_round_trips() {
        let b = Basis::standard();
        let a = AdelicInt::from_i64(1000, b.clone());
        let c = AdelicInt::from_i64(337, b);
        assert_eq!(a.add(&c).try_exact().unwrap(), BigInt::from(1337));
        assert_eq!(a.sub(&c).try_exact().unwrap(), BigInt::from(663));
        assert_eq!(a.mul(&c).try_exact().unwrap(), BigInt::from(337_000));
    }

    #[test]
    fn rational_reconstructs() {
        let b = Basis::standard();
        let x = AdelicRat::from_fraction(&BigInt::from(1), &BigInt::from(6), b.clone());
        let y = AdelicRat::from_fraction(&BigInt::from(1), &BigInt::from(4), b);
        let sum = x.add(&y); // 1/6 + 1/4 = 5/12
        assert_eq!(sum.try_exact().unwrap(), (BigInt::from(5), BigInt::from(12)));
    }
}
