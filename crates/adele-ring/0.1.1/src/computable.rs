//! Level 3 — ℝ_c. Computable reals: numbers stored as *algorithms* that produce
//! a rational approximation to any requested precision, rather than as digits.
//!
//! A [`ComputableReal`] wraps an `Arc<dyn Computable>` oracle plus a precision
//! cache. Asking for `evaluate(p)` returns a rational `r` with `|self - r| < 10⁻ᵖ`.
//! Transcendental constants use fully rational series (Machin's formula for π,
//! the factorial series for e) so no floating point ever contaminates the
//! partial sums.

use std::collections::BTreeMap;
use std::sync::Arc;

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::One;
use parking_lot::Mutex;

use crate::algebraic::AlgebraicNumber;
use crate::ball::Ball;
use crate::basis::Basis;
use crate::rational::RnsRational;

/// An oracle that can approximate a real number to arbitrary precision.
pub trait Computable: Send + Sync {
    /// A rational `r` with `|value - r| < 10^(-precision)`.
    fn evaluate(&self, precision: u64) -> RnsRational;
}

/// A computable real number (Level 3 of the tower).
#[derive(Clone)]
pub struct ComputableReal {
    inner: Arc<dyn Computable>,
    cache: Arc<Mutex<BTreeMap<u64, RnsRational>>>,
    channels: Basis,
}

impl ComputableReal {
    fn wrap(inner: Arc<dyn Computable>, channels: Basis) -> Self {
        ComputableReal {
            inner,
            cache: Arc::new(Mutex::new(BTreeMap::new())),
            channels,
        }
    }

    /// Approximate to `precision` decimal places, memoizing the result.
    ///
    /// The cache is keyed by precision but a cached entry with key `≥ request`
    /// satisfies the ask (a 50-digit enclosure answers a 10-digit query), so we
    /// reuse the tightest already-computed result instead of recomputing.
    pub fn evaluate(&self, precision: u64) -> RnsRational {
        {
            let cache = self.cache.lock();
            if let Some((_, r)) = cache.range(precision..).next() {
                return r.clone();
            }
        }
        let r = self.inner.evaluate(precision);
        self.cache.lock().insert(precision, r.clone());
        r
    }

    /// A rigorous [`Ball`] of width `< 2·10^(-precision)` containing this number.
    ///
    /// This is the Archimedean (Level-3) face of the value: ordering, sign, and
    /// magnitude on computable reals all flow through the returned `Ball`, the
    /// same type the algebraic layer uses for sign-of-a-root. Because the oracle
    /// contract guarantees `|value - evaluate(p)| < 10^(-p)`, padding the rational
    /// midpoint by that radius yields a certified enclosure that composes
    /// rigorously under `Ball` interval arithmetic.
    pub fn enclose(&self, precision: u64) -> Ball {
        let (p, q) = self.evaluate(precision).to_pair();
        let center = BigRational::new(p, q);
        let eps = BigRational::new(BigInt::one(), BigInt::from(10).pow(precision as u32));
        Ball::new(&center - &eps, &center + &eps)
    }

    /// Convenience: evaluate at roughly `f64` precision.
    pub fn evaluate_f64(&self) -> f64 {
        self.evaluate(20).to_f64()
    }

    /// The RNS channels this value computes over.
    pub fn channels(&self) -> Basis {
        self.channels.clone()
    }

    // ── Constructors ────────────────────────────────────────────────────────

    /// A constant rational.
    pub fn from_rational(r: RnsRational) -> Self {
        let channels = r.channels.clone();
        Self::wrap(Arc::new(RationalC { r }), channels)
    }

    /// Drop a Level-2 algebraic number down to Level 3 for digit production.
    pub fn from_algebraic(a: AlgebraicNumber) -> Self {
        let channels = a.channels.clone();
        Self::wrap(Arc::new(AlgebraicC { a }), channels)
    }

    /// π via Machin's formula `π = 16·atan(1/5) - 4·atan(1/239)`.
    pub fn pi(channels: Basis) -> Self {
        Self::wrap(Arc::new(PiC { channels: channels.clone() }), channels)
    }

    /// Euler's number e via the factorial series `Σ 1/k!`.
    pub fn e(channels: Basis) -> Self {
        Self::wrap(Arc::new(EulerC { channels: channels.clone() }), channels)
    }

    /// √r by Newton's method on rationals (quadratic convergence).
    pub fn sqrt(r: RnsRational) -> Self {
        let channels = r.channels.clone();
        Self::wrap(Arc::new(SqrtC { r }), channels)
    }

    /// exp(r) via the Taylor series `Σ rᵏ/k!`.
    pub fn exp(r: RnsRational) -> Self {
        let channels = r.channels.clone();
        Self::wrap(Arc::new(ExpC { r }), channels)
    }

    /// ln(r) for `r > 0` via `2·atanh((r-1)/(r+1))`.
    pub fn ln(r: RnsRational) -> Self {
        let channels = r.channels.clone();
        Self::wrap(Arc::new(LnC { r }), channels)
    }

    // ── Lazy arithmetic ─────────────────────────────────────────────────────

    /// Sum (lazy).
    pub fn add(&self, other: &Self) -> Self {
        Self::wrap(
            Arc::new(BinOp {
                a: self.clone(),
                b: other.clone(),
                kind: BinKind::Add,
            }),
            self.channels.clone(),
        )
    }

    /// Difference (lazy).
    pub fn sub(&self, other: &Self) -> Self {
        self.add(&other.neg())
    }

    /// Product (lazy).
    pub fn mul(&self, other: &Self) -> Self {
        Self::wrap(
            Arc::new(BinOp {
                a: self.clone(),
                b: other.clone(),
                kind: BinKind::Mul,
            }),
            self.channels.clone(),
        )
    }

    /// Negation (lazy).
    pub fn neg(&self) -> Self {
        Self::wrap(Arc::new(NegC { a: self.clone() }), self.channels.clone())
    }

    /// Reciprocal (lazy). Undefined behaviour if the value is zero.
    pub fn recip(&self) -> Self {
        Self::wrap(Arc::new(RecipC { a: self.clone() }), self.channels.clone())
    }
}

// ── Precision helpers ────────────────────────────────────────────────────────

/// `10^(-prec)` as a rational.
fn eps(prec: u64, channels: &Basis) -> RnsRational {
    RnsRational::new(BigInt::one(), pow10(prec), channels.clone())
}

fn pow10(p: u64) -> BigInt {
    BigInt::from(10u8).pow(p as u32)
}

/// Number of integer digits in `|x|` (at least 1), via a cheap low-precision probe.
fn magnitude_digits(cr: &ComputableReal) -> u64 {
    let v = cr.evaluate(4).to_f64().abs();
    if v < 1.0 {
        1
    } else {
        v.log10().floor() as u64 + 1
    }
}

// ── Oracle implementations ───────────────────────────────────────────────────

struct RationalC {
    r: RnsRational,
}
impl Computable for RationalC {
    fn evaluate(&self, _precision: u64) -> RnsRational {
        self.r.clone()
    }
}

struct AlgebraicC {
    a: AlgebraicNumber,
}
impl Computable for AlgebraicC {
    fn evaluate(&self, precision: u64) -> RnsRational {
        let mut clone = self.a.clone();
        let target = eps(precision + 1, &self.a.channels);
        clone.refine_interval(&target);
        clone.interval.0.midpoint(&clone.interval.1)
    }
}

/// arctan(1/x) as a rational, accurate to better than `eps`.
fn atan_inv(x: i64, target: &RnsRational, channels: &Basis) -> RnsRational {
    let mut acc = RnsRational::zero(channels.clone());
    let mut n: i64 = 0;
    loop {
        let exp = (2 * n + 1) as u32;
        let denom = BigInt::from(2 * n + 1) * BigInt::from(x).pow(exp);
        let sign = if n % 2 == 0 { 1 } else { -1 };
        let term = RnsRational::new(BigInt::from(sign), denom, channels.clone());
        acc = acc.add(&term);
        if term.abs() < *target {
            break;
        }
        n += 1;
    }
    acc
}

struct PiC {
    channels: Basis,
}
impl Computable for PiC {
    fn evaluate(&self, precision: u64) -> RnsRational {
        let target = eps(precision + 5, &self.channels);
        let a = atan_inv(5, &target, &self.channels)
            .mul(&RnsRational::from_int(16, self.channels.clone()));
        let b = atan_inv(239, &target, &self.channels)
            .mul(&RnsRational::from_int(4, self.channels.clone()));
        a.sub(&b)
    }
}

struct EulerC {
    channels: Basis,
}
impl Computable for EulerC {
    fn evaluate(&self, precision: u64) -> RnsRational {
        let target = eps(precision + 3, &self.channels);
        let mut acc = RnsRational::zero(self.channels.clone());
        let mut fact = BigInt::one();
        let mut k: u64 = 0;
        loop {
            if k > 0 {
                fact *= BigInt::from(k);
            }
            let term = RnsRational::new(BigInt::one(), fact.clone(), self.channels.clone());
            acc = acc.add(&term);
            if term < target {
                break;
            }
            k += 1;
        }
        acc
    }
}

struct SqrtC {
    r: RnsRational,
}
impl Computable for SqrtC {
    fn evaluate(&self, precision: u64) -> RnsRational {
        let channels = self.r.channels.clone();
        let target = eps(precision + 2, &channels);
        // Initial guess from f64.
        let guess = self.r.to_f64().max(0.0).sqrt();
        let mut x = if guess > 0.0 {
            RnsRational::from_f64(guess, channels.clone())
        } else {
            RnsRational::from_int(1, channels.clone())
        };
        let two = RnsRational::from_int(2, channels.clone());
        // Newton: x_{n+1} = (x + r/x) / 2 until x² is within target of r.
        for _ in 0..200 {
            if x.is_zero() {
                break;
            }
            let next = x.add(&self.r.div(&x)).div(&two);
            let err = next.mul(&next).sub(&self.r).abs();
            x = next;
            if err < target {
                break;
            }
        }
        x
    }
}

struct ExpC {
    r: RnsRational,
}
impl Computable for ExpC {
    fn evaluate(&self, precision: u64) -> RnsRational {
        let channels = self.r.channels.clone();
        let target = eps(precision + 3, &channels);
        let mut acc = RnsRational::zero(channels.clone());
        let mut term = RnsRational::from_int(1, channels.clone()); // r^0 / 0!
        let mut k: u64 = 0;
        loop {
            acc = acc.add(&term);
            if k > 0 && term.abs() < target {
                break;
            }
            k += 1;
            // term *= r / k
            term = term.mul(&self.r).div(&RnsRational::from_int(k as i64, channels.clone()));
            if k > 5000 {
                break;
            }
        }
        acc
    }
}

struct LnC {
    r: RnsRational,
}
impl Computable for LnC {
    fn evaluate(&self, precision: u64) -> RnsRational {
        let channels = self.r.channels.clone();
        let target = eps(precision + 3, &channels);
        // t = (r - 1) / (r + 1); ln(r) = 2 Σ t^(2k+1)/(2k+1).
        let one = RnsRational::from_int(1, channels.clone());
        let t = self.r.sub(&one).div(&self.r.add(&one));
        let t2 = t.mul(&t);
        let mut acc = RnsRational::zero(channels.clone());
        let mut power = t.clone();
        let mut k: u64 = 0;
        loop {
            let term = power.div(&RnsRational::from_int((2 * k + 1) as i64, channels.clone()));
            acc = acc.add(&term);
            if term.abs() < target {
                break;
            }
            power = power.mul(&t2);
            k += 1;
            if k > 100_000 {
                break;
            }
        }
        acc.mul(&RnsRational::from_int(2, channels.clone()))
    }
}

enum BinKind {
    Add,
    Mul,
}

struct BinOp {
    a: ComputableReal,
    b: ComputableReal,
    kind: BinKind,
}
impl Computable for BinOp {
    fn evaluate(&self, precision: u64) -> RnsRational {
        match self.kind {
            BinKind::Add => {
                let pa = self.a.evaluate(precision + 1);
                let pb = self.b.evaluate(precision + 1);
                pa.add(&pb)
            }
            BinKind::Mul => {
                let guard = magnitude_digits(&self.a) + magnitude_digits(&self.b) + 2;
                let pa = self.a.evaluate(precision + guard);
                let pb = self.b.evaluate(precision + guard);
                pa.mul(&pb)
            }
        }
    }
}

struct NegC {
    a: ComputableReal,
}
impl Computable for NegC {
    fn evaluate(&self, precision: u64) -> RnsRational {
        self.a.evaluate(precision).neg()
    }
}

struct RecipC {
    a: ComputableReal,
}
impl Computable for RecipC {
    fn evaluate(&self, precision: u64) -> RnsRational {
        // 1/v loses precision when |v| < 1; add guard digits accordingly.
        let v = self.a.evaluate(4).to_f64().abs();
        let extra = if v > 0.0 && v < 1.0 {
            (-v.log10()).ceil() as u64 * 2 + 2
        } else {
            2
        };
        self.a.evaluate(precision + extra).recip()
    }
}

/// Bridge from Level 2 to Level 3.
impl AlgebraicNumber {
    /// Produce a [`ComputableReal`] that yields digits on demand.
    pub fn to_computable(&self) -> ComputableReal {
        ComputableReal::from_algebraic(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ch() -> Basis {
        Basis::standard()
    }

    #[test]
    fn pi_to_ten_places() {
        let pi = ComputableReal::pi(ch());
        assert!((pi.evaluate(10).to_f64() - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn e_to_fifteen_places() {
        let e = ComputableReal::e(ch());
        assert!((e.evaluate(15).to_f64() - std::f64::consts::E).abs() < 1e-14);
    }

    #[test]
    fn sqrt_two() {
        let r2 = RnsRational::from_int(2, ch());
        let s = ComputableReal::sqrt(r2);
        assert!((s.evaluate(20).to_f64() - 2f64.sqrt()).abs() < 1e-12);
    }

    #[test]
    fn rational_passes_through() {
        let r = RnsRational::from_fraction(1, 3, ch());
        let cr = ComputableReal::from_rational(r.clone());
        assert_eq!(cr.evaluate(100), r);
    }

    #[test]
    fn precision_contract() {
        let pi = ComputableReal::pi(ch());
        let lo = pi.evaluate(5).to_f64();
        let hi = pi.evaluate(50).to_f64();
        assert!((lo - hi).abs() < 1e-5);
    }

    #[test]
    fn lazy_sum_of_pi_and_one() {
        let pi = ComputableReal::pi(ch());
        let one = ComputableReal::from_rational(RnsRational::from_int(1, ch()));
        let sum = pi.add(&one);
        assert!((sum.evaluate(20).to_f64() - (std::f64::consts::PI + 1.0)).abs() < 1e-12);
    }

    #[test]
    fn exp_and_ln() {
        let e = ComputableReal::exp(RnsRational::from_int(1, ch()));
        assert!((e.evaluate(15).to_f64() - std::f64::consts::E).abs() < 1e-13);
        let l = ComputableReal::ln(RnsRational::from_int(2, ch()));
        assert!((l.evaluate(15).to_f64() - 2f64.ln()).abs() < 1e-13);
    }

    #[test]
    fn algebraic_to_computable() {
        let s2 = AlgebraicNumber::sqrt(2, ch()).to_computable();
        assert!((s2.evaluate(15).to_f64() - 2f64.sqrt()).abs() < 1e-13);
    }

    #[test]
    fn enclose_is_rigorous() {
        use crate::ball::rational_to_f64;
        // 1/π via Ball interval arithmetic — certified, narrow.
        let recip = ComputableReal::pi(ch()).enclose(50).recip().unwrap();
        assert!(rational_to_f64(&recip.width()) < 1e-40);
        assert!((recip.to_f64() - 1.0 / std::f64::consts::PI).abs() < 1e-40);

        // e·π composed rigorously, then enclosed.
        let prod = ComputableReal::e(ch()).mul(&ComputableReal::pi(ch())).enclose(30);
        let epi = std::f64::consts::E * std::f64::consts::PI;
        assert!(rational_to_f64(&prod.width()) < 1e-25);
        assert!((prod.to_f64() - epi).abs() < 1e-9);
    }

    #[test]
    fn cache_reuses_tighter_entry() {
        let pi = ComputableReal::pi(ch());
        let _ = pi.evaluate(40); // populate a high-precision entry
        // A lower-precision ask is served from the cached 40-digit result.
        assert!((pi.evaluate(5).to_f64() - std::f64::consts::PI).abs() < 1e-30);
    }
}
