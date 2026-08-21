use crate::{betainc, normcdf};
use std::f64::consts::PI;

/// Student's T cumulative distribution function (CDF).
///
/// Given a value `x` and degrees of freedom `v`, this function returns
/// the probability that a Student's T random variable is less than or equal to `x`.
///
/// # Mathematical Definition
/// For a Student's T distribution with `v` degrees of freedom:
/// - If `x <= 0`: `F(x; v) = 0.5 * I_{v/(v+x^2)}(v/2, 0.5)`
/// - If `x > 0`: `F(x; v) = 1 - 0.5 * I_{v/(v+x^2)}(v/2, 0.5)`
///
/// where `I_z(a, b)` is the regularized incomplete beta function.
///
/// # Domain
/// - `v > 0`
/// - Returns `NaN` if `v <= 0` or any input is `NaN`.
/// - As `v` approaches infinity, the distribution approaches the standard normal distribution.
///
/// # Examples
/// ```
/// use abax::tcdf;
///
/// // Median of any T distribution is 0.5
/// assert!((tcdf(0.0, 5.0, false) - 0.5).abs() < 1e-15);
///
/// // v=1 is the standard Cauchy distribution
/// assert!((tcdf(1.0, 1.0, false) - 0.75).abs() < 1e-15);
/// ```
pub fn tcdf(x: f64, v: f64, upper: bool) -> f64 {
    if x.is_nan() || v.is_nan() || v <= 0.0 {
        return f64::NAN;
    }

    // Student's T is symmetric about 0.
    // P(T > x) = P(T < -x)
    let effective_x = if upper { -x } else { x };

    if v == 1.0 {
        // Special case: Standard Cauchy Distribution
        return 0.5 + effective_x.atan() / PI;
    }

    const NORM_CUTOFF: f64 = 1e7;
    if v > NORM_CUTOFF || v.is_infinite() {
        // Normal approximation for very large degrees of freedom
        return normcdf(effective_x, 0.0, 1.0, false);
    }

    if effective_x == 0.0 {
        return 0.5;
    }

    let abs_x = effective_x.abs();
    let xsq = abs_x * abs_x;

    // Use the relationship between Student's T and the Incomplete Beta function.
    // To maintain precision:
    // If v < x^2, v/(v+x^2) is small.
    // If v >= x^2, x^2/(v+x^2) is small.
    let p_low = if v < xsq {
        betainc(v / (v + xsq), v / 2.0, 0.5, true) / 2.0
    } else {
        // I_z(a, b) = 1 - I_{1-z}(b, a)
        // Here z = x^2/(v+x^2), 1-z = v/(v+x^2)
        // We calculate 0.5 * (1 - I_{x^2/(v+x^2)}(0.5, v/2))
        betainc(xsq / (v + xsq), 0.5, v / 2.0, false) / 2.0
    };

    if effective_x > 0.0 {
        1.0 - p_low
    } else {
        p_low
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcdf_cauchy() {
        let tol = 1e-15;
        assert!((tcdf(1.0, 1.0, false) - 0.75).abs() < tol);
        assert!((tcdf(-1.0, 1.0, false) - 0.25).abs() < tol);
        assert!((tcdf(0.0, 1.0, false) - 0.5).abs() < tol);
    }


    #[test]
    fn test_tcdf_symmetry_and_upper() {
        let x = 1.5;
        let v = 5.0;
        let p_lower = tcdf(x, v, false);
        let p_upper = tcdf(x, v, true);
        assert!((p_lower + p_upper - 1.0).abs() < 1e-15);
        assert_eq!(p_upper, tcdf(-x, v, false));
    }

    #[test]
    fn test_tcdf_limits() {
        assert_eq!(tcdf(f64::INFINITY, 10.0, false), 1.0);
        assert_eq!(tcdf(f64::NEG_INFINITY, 10.0, false), 0.0);
        assert!(tcdf(1.0, 0.0, false).is_nan());
        assert!(tcdf(1.0, -1.0, false).is_nan());
    }

    #[test]
    fn test_tcdf_large_v() {
        // Should be very close to Normal CDF
        let x = 1.96;
        let t = tcdf(x, 1e8, false);
        let n = normcdf(x, 0.0, 1.0, false);
        assert!((t - n).abs() < 1e-12);
    }
}
