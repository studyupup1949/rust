//! Rational reconstruction — turns the original engine's worst failure mode
//! (silent aliasing mod `M`) into a clean, checkable event.
//!
//! Given a residue `x = value mod M`, recover the unique reduced `p/q` with
//! `|p|, |q| <= sqrt(M/2)` such that `p ≡ q·x (mod M)`. When no such fraction
//! exists within the bound, the basis was too small: the caller extends it and
//! retries (see [`crate::basis::Basis::extend_to_bits`]).

use num_bigint::{BigInt, BigUint, Sign};
use num_integer::Integer;
use num_traits::{One, Signed, Zero};

/// Balanced (symmetric) lift of `x mod m` into `(-m/2, m/2]`.
pub fn balanced_lift(x: &BigUint, m: &BigUint) -> BigInt {
    let x = BigInt::from(x.clone());
    let m_big = BigInt::from(m.clone());
    if &x * 2 > m_big {
        x - BigInt::from(m.clone())
    } else {
        x
    }
}

/// Recover the unique reduced `p/q` with `|p|, |q| <= sqrt(M/2)` and
/// `p ≡ q·x (mod M)`, or `None` if none exists within the bound.
pub fn rational_reconstruct(x: &BigUint, m: &BigUint) -> Option<(BigInt, BigInt)> {
    if m.is_zero() {
        return None;
    }
    let x = x % m;
    if x.is_zero() {
        return Some((BigInt::zero(), BigInt::one()));
    }

    // Bound N = floor(sqrt(M/2)).
    let bound = BigInt::from((m / 2u32).sqrt());

    let m_big = BigInt::from(m.clone());
    let x_big = BigInt::from(x);

    // Extended Euclid on (m, x), tracking only the `t` coefficients.
    let mut r0 = m_big.clone();
    let mut r1 = x_big.clone();
    let mut t0 = BigInt::zero();
    let mut t1 = BigInt::one();

    while r1 > bound {
        let q = &r0 / &r1;
        let r2 = &r0 - &q * &r1;
        r0 = r1;
        r1 = r2;
        let t2 = &t0 - &q * &t1;
        t0 = t1;
        t1 = t2;
    }

    // Candidate fraction p/q = r1 / t1.
    let mut p = r1;
    let mut q = t1;
    if q.is_zero() {
        return None;
    }
    if q.sign() == Sign::Minus {
        p = -p;
        q = -q;
    }

    // Validate: denominator within bound, reduced, and congruence holds.
    if q.abs() > bound || p.abs() > bound {
        return None;
    }
    if p.gcd(&q) != BigInt::one() {
        return None;
    }
    let check = (&p - &q * &x_big).mod_floor(&m_big);
    if !check.is_zero() {
        return None;
    }
    Some((p, q))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode(p: i64, q: i64, m: &BigUint) -> BigUint {
        // x = p * q^{-1} mod m
        let m_big = BigInt::from(m.clone());
        let qi = BigInt::from(q);
        let inv = mod_inverse_big(&qi, &m_big).unwrap();
        let x = (BigInt::from(p) * inv).mod_floor(&m_big);
        x.to_biguint().unwrap()
    }

    fn mod_inverse_big(a: &BigInt, m: &BigInt) -> Option<BigInt> {
        let g = a.extended_gcd(m);
        if g.gcd != BigInt::one() {
            return None;
        }
        Some(g.x.mod_floor(m))
    }

    #[test]
    fn round_trip() {
        let m = BigUint::from(1u32) << 64;
        let x = encode(3, 7, &m);
        assert_eq!(rational_reconstruct(&x, &m), Some((BigInt::from(3), BigInt::from(7))));
        let x = encode(-5, 11, &m);
        assert_eq!(rational_reconstruct(&x, &m), Some((BigInt::from(-5), BigInt::from(11))));
    }

    #[test]
    fn integer_round_trip() {
        let m = BigUint::from(1u32) << 64;
        let x = BigUint::from(42u32);
        assert_eq!(rational_reconstruct(&x, &m), Some((BigInt::from(42), BigInt::one())));
    }

    #[test]
    fn out_of_range_fails() {
        // m too small for p/q with 2*max(|p|,|q|)^2 < M
        let m = BigUint::from(50u32);
        let x = encode(7, 9, &m); // 2*81 = 162 > 50 → not reconstructible
        assert_eq!(rational_reconstruct(&x, &m), None);
    }
}
