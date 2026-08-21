use crate::{gammainc, gammaln};
/// Inverse of the regularized incomplete gamma function.
///
/// Solves for `x >= 0` in either:
/// - `P(a, x) = y` when `lower = true`
/// - `Q(a, x) = y` when `lower = false`
///
/// # Domain
/// - `a > 0`
/// - `0 <= y <= 1`
/// - Invalid inputs return `NaN`.
pub fn gammaincinv(y: f64, a: f64, lower: bool) -> f64 {
    // 1. Validation & Boundaries
    if y.is_nan() || a.is_nan() || a <= 0.0 || !(0.0..=1.0).contains(&y) {
        return f64::NAN;
    }
    match (lower, y) {
        (true, 0.0) | (false, 1.0) => return 0.0,
        (true, 1.0) | (false, 0.0) => return f64::INFINITY,
        _ => {}
    }

    // 2. Monotonicity-Aware Bracketing
    let mut lo = 0.0;
    let mut hi = a.max(1.0);
    let mut val_hi = gammainc(hi, a, lower, false);

    // Goal: Ensure y is between gammainc(lo) and gammainc(hi)
    // For lower=true:  f(0)=0, f(inf)=1  (Increasing)
    // For lower=false: f(0)=1, f(inf)=0  (Decreasing)

    if (lower && val_hi < y) || (!lower && val_hi > y) {
        // We need to move towards Infinity to find the root
        while (lower && val_hi < y) || (!lower && val_hi > y) {
            lo = hi;
            hi *= 2.0;
            val_hi = gammainc(hi, a, lower, false);
            if !hi.is_finite() { return f64::INFINITY; }
        }
    } else {
        // We need to move towards Zero to find the root
        while (lower && val_hi > y) || (!lower && val_hi < y) {
            let next_hi = hi * 0.5;
            let val_next = gammainc(next_hi, a, lower, false);
            if (lower && val_next < y) || (!lower && val_next > y) {
                lo = next_hi;
                break;
            }
            hi = next_hi;
            val_hi = val_next;
            if hi < 1e-300 { break; }
        }
    }

    let gln = gammaln(a);
    let mut x = 0.5 * (lo + hi);

    // 3. Combined Root-Finding Loop (Halley + Newton + Bisection)
    for _ in 0..100 {
        let val = gammainc(x, a, lower, false);
        let f = val - y;

        if f.abs() < f64::EPSILON * y.max(1e-12) { break; }

        // Update Bracket
        // If lower=true and f>0, x is too high. If lower=false and f>0, x is too low.
        if (lower && f > 0.0) || (!lower && f < 0.0) {
            hi = x;
        } else {
            lo = x;
        }

        let log_pdf = (a - 1.0) * x.ln() - x - gln;
        let mut f_prime = log_pdf.exp();
        if !lower { f_prime = -f_prime; } 

        let deriv_ratio = (a - 1.0) / x - 1.0; 
        let mut delta = f / (f_prime - (f * deriv_ratio / 2.0));

        // Fallback to Newton
        if !delta.is_finite() || (x - delta) <= lo || (x - delta) >= hi {
            delta = f / f_prime;
        }

        // Fallback to Bisection
        let mut x_new = x - delta;
        if !x_new.is_finite() || x_new <= lo || x_new >= hi {
            x_new = 0.5 * (lo + hi);
        }

        if (x_new - x).abs() < f64::EPSILON * x {
            x = x_new;
            break;
        }
        x = x_new;
    }

    x.max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gammaincinv_roundtrip_lower() {
        let a = 2.5;
        for &y in &[1e-12, 1e-9, 1e-6, 0.01, 0.2, 0.5, 0.8, 0.99, 1.0 - 1e-10] {
            let x = gammaincinv(y, a, true);
            let y2 = gammainc(x, a, true, false);
            assert!((y2 - y).abs() <= 5e-12_f64.max(5e-12 * y.abs()));
        }
    }

    #[test]
    fn test_gammaincinv_roundtrip_upper() {
        let a = 5.0;
        for &y in &[1e-12, 1e-9, 1e-6, 0.01, 0.2, 0.5, 0.8, 0.99, 1.0 - 1e-10] {
            let x = gammaincinv(y, a, false);
            let y2 = gammainc(x, a, false, false);
            println!("x: {:.e}, y: {:.e}, y2: {:.e}", x, y, y2);
            assert!((y2 - y).abs() <= 5e-12_f64.max(5e-12 * y.abs()));
        }
    }

    #[test]
    fn test_gammaincinv_domain_and_edges() {
        assert!(gammaincinv(0.5, 0.0, true).is_nan());
        assert!(gammaincinv(-0.1, 1.0, true).is_nan());
        assert_eq!(gammaincinv(0.0, 2.0, true), 0.0);
        assert!(gammaincinv(1.0, 2.0, true).is_infinite());
        assert_eq!(gammaincinv(1.0, 2.0, false), 0.0);
        assert!(gammaincinv(0.0, 2.0, false).is_infinite());
    }
}
