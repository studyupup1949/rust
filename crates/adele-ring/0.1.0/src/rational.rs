//! Level 1 — ℚ. Exact rational arithmetic over the RNS substrate.
//!
//! An [`RnsRational`] stores a reduced numerator and a positive denominator,
//! each as an [`RnsInt`]. Arithmetic reconstructs to `BigInt`, computes exactly,
//! and re-normalizes — so results never drift the way floating point does.
//!
//! The module also exposes *base awareness*: which primes a fraction's
//! denominator contains, whether it terminates in a given base, and the minimal
//! ("natural") base in which it is exact.

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Signed, ToPrimitive, Zero};

use crate::primes::{distinct_primes, radical, termination_period};
use crate::rns::{Channels, RnsInt};

/// An exact rational number represented over RNS channels.
///
/// The canonical value is held as a reduced `BigInt` pair (`p`, `q`) so that
/// precision is never capped by the channel capacity — essential for the
/// arbitrary-precision rationals produced at Level 3. The `numer`/`denom`
/// [`RnsInt`] fields are the RNS *view* of that value (exact while the value
/// fits inside the channel capacity), kept for the lower-level channel-dispatch
/// machinery.
#[derive(Clone, Debug)]
pub struct RnsRational {
    /// Reduced numerator (carries the sign).
    p: BigInt,
    /// Reduced denominator (always positive).
    q: BigInt,
    /// RNS view of the numerator.
    pub numer: RnsInt,
    /// RNS view of the denominator.
    pub denom: RnsInt,
    pub channels: Channels,
}

impl RnsRational {
    /// Build from a `p/q` pair, normalizing the sign and reducing by the GCD.
    ///
    /// Panics if `q == 0`.
    pub fn new(p: BigInt, q: BigInt, channels: Channels) -> Self {
        assert!(!q.is_zero(), "denominator must be non-zero");
        let mut p = p;
        let mut q = q;
        if q.is_negative() {
            p = -p;
            q = -q;
        }
        let g = p.gcd(&q);
        if !g.is_zero() {
            p /= &g;
            q /= &g;
        }
        let numer = RnsInt::from_bigint(&p, channels.clone());
        let denom = RnsInt::from_bigint(&q, channels.clone());
        RnsRational {
            p,
            q,
            numer,
            denom,
            channels,
        }
    }

    /// Convenience constructor from machine integers.
    pub fn from_fraction(p: i64, q: i64, channels: Channels) -> Self {
        Self::new(BigInt::from(p), BigInt::from(q), channels)
    }

    /// An integer rational `n/1`.
    pub fn from_int(n: i64, channels: Channels) -> Self {
        Self::from_fraction(n, 1, channels)
    }

    /// Zero.
    pub fn zero(channels: Channels) -> Self {
        Self::from_fraction(0, 1, channels)
    }

    /// The reduced `(p, q)` pair as `BigInt`s (exact, never capacity-bounded).
    pub fn to_pair(&self) -> (BigInt, BigInt) {
        (self.p.clone(), self.q.clone())
    }

    /// `true` iff the value is zero.
    pub fn is_zero(&self) -> bool {
        self.p.is_zero()
    }

    /// `true` iff the denominator is 1 (the value is an integer).
    pub fn is_integer(&self) -> bool {
        self.q == BigInt::one()
    }

    fn op(&self, other: &Self, f: impl Fn(&BigInt, &BigInt, &BigInt, &BigInt) -> (BigInt, BigInt)) -> Self {
        let (p1, q1) = self.to_pair();
        let (p2, q2) = other.to_pair();
        let (p, q) = f(&p1, &q1, &p2, &q2);
        Self::new(p, q, self.channels.clone())
    }

    /// `p1/q1 + p2/q2 = (p1·q2 + p2·q1) / (q1·q2)`.
    pub fn add(&self, other: &Self) -> Self {
        self.op(other, |p1, q1, p2, q2| (p1 * q2 + p2 * q1, q1 * q2))
    }

    /// Subtraction.
    pub fn sub(&self, other: &Self) -> Self {
        self.op(other, |p1, q1, p2, q2| (p1 * q2 - p2 * q1, q1 * q2))
    }

    /// Multiplication.
    pub fn mul(&self, other: &Self) -> Self {
        self.op(other, |p1, q1, p2, q2| (p1 * p2, q1 * q2))
    }

    /// Division. Panics if `other` is zero.
    pub fn div(&self, other: &Self) -> Self {
        assert!(!other.is_zero(), "division by zero");
        self.op(other, |p1, q1, p2, q2| (p1 * q2, q1 * p2))
    }

    /// Additive inverse.
    pub fn neg(&self) -> Self {
        let (p, q) = self.to_pair();
        Self::new(-p, q, self.channels.clone())
    }

    /// Multiplicative inverse. Panics if zero.
    pub fn recip(&self) -> Self {
        assert!(!self.is_zero(), "reciprocal of zero");
        let (p, q) = self.to_pair();
        Self::new(q, p, self.channels.clone())
    }

    // ── Base awareness ──────────────────────────────────────────────────────

    /// The denominator value as a `u64` (denominators are small after reduction
    /// for practical inputs). Returns `None` if it exceeds `u64`.
    fn denom_u64(&self) -> Option<u64> {
        self.denom.to_bigint().to_u64()
    }

    /// The distinct primes appearing in the reduced denominator.
    pub fn denom_prime_signature(&self) -> Vec<u64> {
        match self.denom_u64() {
            Some(d) => distinct_primes(d),
            None => Vec::new(),
        }
    }

    /// `true` iff this fraction has a terminating expansion in `base`, i.e. every
    /// prime in the denominator divides `base`.
    pub fn exact_in_base(&self, base: u64) -> bool {
        self.denom_prime_signature()
            .into_iter()
            .all(|p| base % p == 0)
    }

    /// Minimal base for an exact (terminating) representation = `radical(denom)`.
    pub fn natural_base(&self) -> u64 {
        self.denom_u64().map(radical).unwrap_or(1).max(1)
    }

    /// Repeating period in `base`: `0` if it terminates, else the period length.
    pub fn termination_period_in_base(&self, base: u64) -> u64 {
        match self.denom_u64() {
            Some(d) => termination_period(d, base).unwrap_or(0),
            None => 0,
        }
    }

    // ── Floating-point bridge ───────────────────────────────────────────────

    /// Lossy `f64` approximation.
    ///
    /// Computed via bit-shift scaling so it stays finite even when the numerator
    /// and denominator are far too large to fit in an `f64` individually.
    pub fn to_f64(&self) -> f64 {
        let (p, q) = self.to_pair();
        ratio_to_f64(&p, &q)
    }

    /// Exact dyadic rational equal to a finite `f64`.
    pub fn from_f64(x: f64, channels: Channels) -> Self {
        if x == 0.0 || !x.is_finite() {
            return Self::zero(channels);
        }
        let bits = x.to_bits();
        let sign = if bits >> 63 == 1 { -1i64 } else { 1 };
        let exponent = ((bits >> 52) & 0x7ff) as i64;
        let mantissa = bits & 0x000f_ffff_ffff_ffff;
        // Reconstruct mantissa * 2^(e) with the implicit leading bit.
        let (m, e) = if exponent == 0 {
            (mantissa, -1074i64) // subnormal
        } else {
            (mantissa | 0x0010_0000_0000_0000, exponent - 1075)
        };
        let m = BigInt::from(sign) * BigInt::from(m);
        if e >= 0 {
            Self::new(m * (BigInt::one() << (e as usize)), BigInt::one(), channels)
        } else {
            Self::new(m, BigInt::one() << ((-e) as usize), channels)
        }
    }

    /// `f64` approximation together with the exact representation error
    /// `self - f64(self)` as an `RnsRational`.
    pub fn to_f64_with_error(&self) -> (f64, RnsRational) {
        let approx = self.to_f64();
        let approx_exact = Self::from_f64(approx, self.channels.clone());
        let error = self.sub(&approx_exact);
        (approx, error)
    }

    /// Sign of the value: `-1`, `0`, or `+1`.
    pub fn signum(&self) -> i32 {
        match self.p.sign() {
            num_bigint::Sign::Minus => -1,
            num_bigint::Sign::NoSign => 0,
            num_bigint::Sign::Plus => 1,
        }
    }

    /// Absolute value.
    pub fn abs(&self) -> Self {
        if self.signum() < 0 {
            self.neg()
        } else {
            self.clone()
        }
    }

    /// Midpoint `(self + other) / 2`.
    pub fn midpoint(&self, other: &Self) -> Self {
        self.add(other).mul(&Self::from_fraction(1, 2, self.channels.clone()))
    }

    /// Human-readable form: `"3/7"`, or just `"5"` when the denominator is 1.
    pub fn display(&self) -> String {
        let (p, q) = self.to_pair();
        if q == BigInt::one() {
            p.to_string()
        } else {
            format!("{p}/{q}")
        }
    }
}

/// Convert `p/q` to `f64`, scaling by powers of two so neither operand
/// overflows the `f64` range during the division.
fn ratio_to_f64(p: &BigInt, q: &BigInt) -> f64 {
    if q.is_zero() {
        return f64::NAN;
    }
    if p.is_zero() {
        return 0.0;
    }
    let negative = (p.sign() == num_bigint::Sign::Minus) ^ (q.sign() == num_bigint::Sign::Minus);
    let mut pm = p.magnitude().clone();
    let mut qm = q.magnitude().clone();
    let pb = pm.bits() as i64;
    let qb = qm.bits() as i64;
    // Aim for the quotient to carry ~60 significant bits.
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

impl PartialEq for RnsRational {
    fn eq(&self, other: &Self) -> bool {
        self.to_pair() == other.to_pair()
    }
}
impl Eq for RnsRational {}

impl PartialOrd for RnsRational {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RnsRational {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // p1/q1 ? p2/q2  <=>  p1*q2 ? p2*q1  (both denominators positive)
        let (p1, q1) = self.to_pair();
        let (p2, q2) = other.to_pair();
        (p1 * q2).cmp(&(p2 * q1))
    }
}

impl std::fmt::Display for RnsRational {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ch() -> Channels {
        Channels::standard(32)
    }

    fn frac(p: i64, q: i64) -> RnsRational {
        RnsRational::from_fraction(p, q, ch())
    }

    #[test]
    fn sixths_tenths_fifteenths() {
        // 1/6 + 1/10 + 1/15 == 1/3
        let r = frac(1, 6).add(&frac(1, 10)).add(&frac(1, 15));
        assert_eq!(r, frac(1, 3));
    }

    #[test]
    fn third_times_three() {
        assert_eq!(frac(1, 3).mul(&frac(3, 1)), frac(1, 1));
        assert_eq!(frac(1, 7).mul(&frac(7, 1)), frac(1, 1));
    }

    #[test]
    fn point_one_plus_point_two() {
        // 0.1 + 0.2 == 3/10  (not 0.30000000000000004)
        assert_eq!(frac(1, 10).add(&frac(1, 5)), frac(3, 10));
    }

    #[test]
    fn eighths() {
        assert_eq!(frac(7, 8).sub(&frac(3, 8)), frac(1, 2));
    }

    #[test]
    fn base_awareness() {
        assert_eq!(frac(1, 6).natural_base(), 6);
        assert_eq!(frac(1, 12).natural_base(), 6); // radical(12) = 6
        assert!(frac(1, 6).exact_in_base(6));
        assert!(!frac(1, 6).exact_in_base(10));
        assert!(frac(1, 6).exact_in_base(30));
        assert_eq!(frac(1, 8).termination_period_in_base(10), 0); // terminates
        assert_eq!(frac(1, 3).termination_period_in_base(10), 1);
    }

    #[test]
    fn f64_error_is_exact() {
        // 1/10 is not exact in binary; the error must be non-zero and exact.
        let (approx, err) = frac(1, 10).to_f64_with_error();
        assert!((approx - 0.1).abs() < 1e-17);
        assert!(!err.is_zero());
        // approx + err reconstructs the original exactly.
        let reconstructed = RnsRational::from_f64(approx, ch()).add(&err);
        assert_eq!(reconstructed, frac(1, 10));
    }

    #[test]
    fn display_form() {
        assert_eq!(frac(3, 7).display(), "3/7");
        assert_eq!(frac(10, 2).display(), "5");
    }
}
