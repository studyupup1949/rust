use crate::{betaincinv, norminv};
use std::f64::consts::PI;

/// Inverse of Student's T cumulative distribution function (CDF).
///
/// Given a probability `p` and degrees of freedom `v`, this function returns
/// the value `x` such that the probability of a Student's T random variable
/// being less than or equal to `x` is `p`.
///
/// # Mathematical Definition
/// - For `v = 1`, it is the standard Cauchy quantile function: `tan(pi * (p - 0.5))`.
/// - For other `v`, it uses the relationship with the Inverse Incomplete Beta function:
///   `x = sign(p - 0.5) * sqrt(v * (1 - z) / z)`, where `z` is the root of the
///   regularized incomplete beta function.
/// - For large `v`, it uses the Cornish-Fisher expansion (Abramowitz & Stegun 26.7.5).
///
/// # Domain
/// - `0 <= p <= 1`
/// - `v > 0`
/// - Returns `NaN` if `p` is out of range, `v` is non-positive, or any input is `NaN`.
///
/// # Examples
/// ```
/// use abax::tinv;
///
/// // Median of any T distribution is 0
/// assert!((tinv(0.5, 5.0) - 0.0).abs() < 1e-15);
///
/// // v=1 is the standard Cauchy distribution: p=0.75 -> x=1
/// assert!((tinv(0.75, 1.0) - 1.0).abs() < 1e-15);
/// ```
pub fn tinv(p: f64, v: f64) -> f64 {
    if p.is_nan() || v.is_nan() || !(0.0..=1.0).contains(&p) || v <= 0.0 {
        return f64::NAN;
    }

    if p == 0.0 {
        return f64::NEG_INFINITY;
    }
    if p == 1.0 {
        return f64::INFINITY;
    }

    if v == 1.0 {
        // Explicit inversion of Cauchy distribution
        return (PI * (p - 0.5)).tan();
    }

    if v < 1000.0 {
        let q = p - 0.5;
        let abs_2q = 2.0 * q.abs();

        // Use betaincinv to invert the T distribution
        // Relationship: P(|T| < x) = 1 - I_{v/(v+x^2)}(v/2, 0.5)
        let (z, oneminusz) = if q.abs() < 0.25 {
            // Central region: compute 1-z directly to avoid roundoff
            let omz = betaincinv(abs_2q, 0.5, v / 2.0, true);
            (1.0 - omz, omz)
        } else {
            // Tail region
            let z_val = betaincinv(abs_2q, v / 2.0, 0.5, false);
            (z_val, 1.0 - z_val)
        };

        return q.signum() * (v * (oneminusz / z)).sqrt();
    }

    // For large degrees of freedom, use Abramowitz & Stegun formula 26.7.5
    let xn = norminv(p, 0.0, 1.0);
    let xn2 = xn * xn;
    let xn3 = xn2 * xn;
    let xn5 = xn3 * xn2;
    let xn7 = xn5 * xn2;
    let xn9 = xn7 * xn2;

    xn + (xn3 + xn) / (4.0 * v)
        + (5.0 * xn5 + 16.0 * xn3 + 3.0 * xn) / (96.0 * v * v)
        + (3.0 * xn7 + 19.0 * xn5 + 17.0 * xn3 - 15.0 * xn) / (384.0 * v * v * v)
        + (79.0 * xn9 + 776.0 * xn7 + 1482.0 * xn5 - 1920.0 * xn3 - 945.0 * xn)
            / (92160.0 * v * v * v * v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tinv_basic() {
        let tol = 1e-14;
        assert!((tinv(0.5, 5.0) - 0.0).abs() < tol);
        // Reference value for v=5, p=0.975 (Standard 95% CI critical value)
        assert!((tinv(0.975, 5.0) - 2.570581835636315).abs() < tol);
    }

    #[test]
    fn test_tinv_cauchy() {
        let tol = 1e-15;
        assert!((tinv(0.75, 1.0) - 1.0).abs() < tol);
        assert!((tinv(0.25, 1.0) + 1.0).abs() < tol);
    }

    #[test]
    fn test_tinv_large_v() {
        // Should approach norminv
        let p = 0.95;
        let t = tinv(p, 1e6);
        let n = 1.6448551507220405;
        assert!((t - n).abs() < 1e-10);
    }

    #[test]
    fn test_tinv_limits() {
        assert_eq!(tinv(0.0, 10.0), f64::NEG_INFINITY);
        assert_eq!(tinv(1.0, 10.0), f64::INFINITY);
        assert!(tinv(0.5, 0.0).is_nan());
        assert!(tinv(-0.1, 5.0).is_nan());
    }
}