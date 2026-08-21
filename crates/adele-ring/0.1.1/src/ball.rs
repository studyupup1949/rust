//! The Archimedean (infinite) place, made into a type.
//!
//! A [`Ball`] is a rigorous rational enclosure `[lo, hi]` of a real number — never
//! a point estimate. **Every** ordering / sign / magnitude decision in the tower
//! goes through `Ball`, because the p-adic (finite) channels are constitutionally
//! blind to size: they measure divisibility, not magnitude.
//!
//! When [`Ball::sign`] or [`Ball::cmp`] returns `None` the enclosure straddles the
//! boundary and the caller is obligated to *refine* (bisect an algebraic interval,
//! or pull more digits from a computable real) and retry. This is the only honest
//! way to decide the sign of an exact real, and it replaces every place the old
//! code silently reconstructed through `BigInt`.
//!
//! The `lo`/`hi` rationals stay *small*: they are low-precision enclosures whose
//! `BigInt` content is bounded by the requested precision, not by the basis
//! modulus `M`.

use std::cmp::Ordering;

use num_bigint::{BigInt, Sign};
use num_rational::BigRational;
use num_traits::{One, Signed, ToPrimitive, Zero};

/// Exact rational type used for `Ball` endpoints.
pub type Rational = BigRational;

/// A rigorous rational enclosure of a real number.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ball {
    pub lo: Rational,
    pub hi: Rational,
}

impl Ball {
    /// An enclosure `[lo, hi]` (endpoints are swapped if given out of order).
    pub fn new(lo: Rational, hi: Rational) -> Self {
        if lo <= hi {
            Ball { lo, hi }
        } else {
            Ball { lo: hi, hi: lo }
        }
    }

    /// A zero-width ball at the exact rational `r`.
    pub fn point(r: &Rational) -> Self {
        Ball { lo: r.clone(), hi: r.clone() }
    }

    /// A zero-width ball at an integer.
    pub fn from_i64(n: i64) -> Self {
        Self::point(&BigRational::from_integer(BigInt::from(n)))
    }

    /// Sign of the enclosed value, or `None` if the ball straddles zero
    /// (the caller must refine and retry).
    pub fn sign(&self) -> Option<Sign> {
        if self.lo.is_positive() {
            Some(Sign::Plus)
        } else if self.hi.is_negative() {
            Some(Sign::Minus)
        } else if self.lo.is_zero() && self.hi.is_zero() {
            Some(Sign::NoSign)
        } else {
            None
        }
    }

    /// Ordering against another ball, or `None` if the two overlap (refine).
    pub fn cmp(&self, other: &Ball) -> Option<Ordering> {
        if self.hi < other.lo {
            Some(Ordering::Less)
        } else if self.lo > other.hi {
            Some(Ordering::Greater)
        } else if self.lo == self.hi && other.lo == other.hi && self.lo == other.lo {
            Some(Ordering::Equal)
        } else {
            None
        }
    }

    /// Whether the enclosure contains zero.
    pub fn contains_zero(&self) -> bool {
        !self.lo.is_positive() && !self.hi.is_negative()
    }

    /// Whether the enclosure contains the rational `v`.
    pub fn contains(&self, v: &Rational) -> bool {
        self.lo <= *v && *v <= self.hi
    }

    /// Width `hi - lo`.
    pub fn width(&self) -> Rational {
        &self.hi - &self.lo
    }

    /// Midpoint `(lo + hi) / 2`.
    pub fn midpoint(&self) -> Rational {
        (&self.lo + &self.hi) / BigRational::from_integer(BigInt::from(2))
    }

    /// Nearest `f64` to the midpoint (endpoints are small, so this is finite).
    pub fn to_f64(&self) -> f64 {
        rational_to_f64(&self.midpoint())
    }

    /// Interval addition.
    pub fn add(&self, b: &Ball) -> Ball {
        Ball::new(&self.lo + &b.lo, &self.hi + &b.hi)
    }

    /// Interval negation.
    pub fn neg(&self) -> Ball {
        Ball::new(-&self.hi, -&self.lo)
    }

    /// Interval subtraction.
    pub fn sub(&self, b: &Ball) -> Ball {
        Ball::new(&self.lo - &b.hi, &self.hi - &b.lo)
    }

    /// Interval multiplication (min/max over the four endpoint products).
    pub fn mul(&self, b: &Ball) -> Ball {
        let products = [
            &self.lo * &b.lo,
            &self.lo * &b.hi,
            &self.hi * &b.lo,
            &self.hi * &b.hi,
        ];
        let mut min = products[0].clone();
        let mut max = products[0].clone();
        for p in &products[1..] {
            if *p < min {
                min = p.clone();
            }
            if *p > max {
                max = p.clone();
            }
        }
        Ball::new(min, max)
    }

    /// Interval reciprocal, or `None` when the enclosure contains zero.
    pub fn recip(&self) -> Option<Ball> {
        if self.contains_zero() {
            return None;
        }
        let one = BigRational::one();
        Some(Ball::new(&one / &self.hi, &one / &self.lo))
    }
}

/// Convert a (small) rational to `f64` via bit-shift scaling so neither endpoint
/// overflows during the division.
pub fn rational_to_f64(r: &Rational) -> f64 {
    let p = r.numer();
    let q = r.denom();
    if q.is_zero() {
        return f64::NAN;
    }
    if p.is_zero() {
        return 0.0;
    }
    let negative = (p.sign() == Sign::Minus) ^ (q.sign() == Sign::Minus);
    let mut pm = p.magnitude().clone();
    let mut qm = q.magnitude().clone();
    let pb = pm.bits() as i64;
    let qb = qm.bits() as i64;
    let shift = 60 + qb - pb;
    if shift > 0 {
        pm <<= shift as u64;
    } else {
        qm <<= (-shift) as u64;
    }
    let quo = &pm / &qm;
    let mag = quo.to_f64().unwrap_or(f64::INFINITY);
    let value = mag * 2f64.powi(-(shift as i32));
    if negative {
        -value
    } else {
        value
    }
}

/// Build a rational from an `i64` ratio.
pub fn rat(p: i64, q: i64) -> Rational {
    BigRational::new(BigInt::from(p), BigInt::from(q))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_and_straddle() {
        assert_eq!(Ball::new(rat(1, 2), rat(3, 2)).sign(), Some(Sign::Plus));
        assert_eq!(Ball::new(rat(-3, 2), rat(-1, 2)).sign(), Some(Sign::Minus));
        assert_eq!(Ball::new(rat(-1, 1), rat(1, 1)).sign(), None);
        assert_eq!(Ball::from_i64(0).sign(), Some(Sign::NoSign));
    }

    #[test]
    fn ordering() {
        let a = Ball::from_i64(1);
        let b = Ball::from_i64(2);
        assert_eq!(a.cmp(&b), Some(Ordering::Less));
        assert_eq!(b.cmp(&a), Some(Ordering::Greater));
        // overlapping intervals → None
        assert_eq!(Ball::new(rat(0, 1), rat(2, 1)).cmp(&Ball::new(rat(1, 1), rat(3, 1))), None);
    }

    #[test]
    fn interval_arith() {
        let a = Ball::new(rat(1, 1), rat(2, 1));
        let b = Ball::new(rat(3, 1), rat(4, 1));
        assert_eq!(a.add(&b), Ball::new(rat(4, 1), rat(6, 1)));
        assert_eq!(a.mul(&b), Ball::new(rat(3, 1), rat(8, 1)));
        assert!(a.recip().is_some());
        assert!(Ball::new(rat(-1, 1), rat(1, 1)).recip().is_none());
    }

    #[test]
    fn to_float() {
        assert!((Ball::new(rat(1, 3), rat(1, 3)).to_f64() - (1.0 / 3.0)).abs() < 1e-15);
    }
}
