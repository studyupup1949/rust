use crate::{gammainc, gammaincinv};

/// Inverse Gamma cumulative distribution function (CDF).
///
/// Given a probability `p`, a shape parameter `a`, and a scale parameter `b`,
/// this function returns the value `x` such that the probability of a Gamma
/// random variable being less than or equal to `x` is `p`.
///
/// # Mathematical Definition
/// The inverse Gamma CDF is derived from the inverse regularized incomplete
/// gamma function <math><msup><mi>P</mi><mrow><mo>-</mo><mn>1</mn></mrow></msup><mo>(</mo><mi>a</mi><mo>,</mo><mi>p</mi><mo>)</mo></math>:
/// <math display="block">
///   <mi>x</mi><mo>=</mo><mi>b</mi><mo>·</mo><msup><mi>P</mi><mrow><mo>-</mo><mn>1</mn></mrow></msup><mo>(</mo><mi>a</mi><mo>,</mo><mi>p</mi><mo>)</mo>
/// </math>
///
/// # Domain
/// - <math><mn>0</mn><mo>≤</mo><mi>p</mi><mo>≤</mo><mn>1</mn></math>
/// - <math><mi>a</mi><mo>&gt;</mo><mn>0</mn></math>
/// - <math><mi>b</mi><mo>&gt;</mo><mn>0</mn></math>
/// - Returns `NaN` if `p` is out of range, or `a` or `b` are non-positive.
/// - For <math><mi>a</mi><mo>=</mo><mn>0</mn></math>, the distribution is degenerate at 0, so it returns 0 for any valid `p`.
///
/// # Examples
/// ```
/// use abax::gaminv;
///
/// // For a=1, Gamma reduces to Exponential distribution: x = -b * ln(1 - p)
/// let p = 0.5;
/// let a = 1.0;
/// let b = 2.0;
/// let x = gaminv(p, a, b);
/// assert!((x - (-b * (1.0 - p).ln())).abs() < 1e-12);
///
/// // Median of Gamma(a=3, b=1)
/// let x_median = gaminv(0.5, 3.0, 1.0);
/// // Check against CDF: F(x_median; 3, 1) should be 0.5
/// // (CDF is implemented in gamcdf)
/// ```
pub fn gaminv(p: f64, a: f64, b: f64) -> f64 {
    let ok_ab = (0.0 < a && a < f64::INFINITY) && 0.0 < b;
    let ok_p = 0.0 < p && p < 1.0;

    if !ok_ab || !ok_p {
        let mut x = f64::NAN;
        if ok_ab && p == 0.0 {
            x = 0.0;
        }
        if ok_ab && p == 1.0 {
            x = f64::INFINITY;
        }
        if a == 0.0 && ok_p {
            x = 0.0;
        }
        return x;
    }

    let q = gammaincinv(p, a, true);
    if cfg!(debug_assertions) {
        let tolerance = f64::sqrt(f64::EPSILON);
        let badcdf = (f64::abs(gammainc(q, a, true, false) - p) / p) > tolerance;
        assert!(!badcdf, "cdf is too far off");
    }

    q * b
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gamcdf;

    #[test]
    fn test_gaminv_exponential_identity() {
        // a=1 is Exponential(scale=b). x = -b * ln(1-p)
        let a = 1.0;
        let b = 2.5;
        let p = 0.75;
        let x = gaminv(p, a, b);
        let expected = -b * (1.0 - p).ln();
        assert!((x - expected).abs() < 1e-14);
    }

    #[test]
    fn test_gaminv_roundtrip() {
        let a = 3.2;
        let b = 0.5;
        let probabilities = [0.01, 0.1, 0.5, 0.9, 0.99];
        for &p in &probabilities {
            let x = gaminv(p, a, b);
            let p_back = gamcdf(x, a, b, false);
            assert!((p - p_back).abs() < 1e-12);
        }
    }

    #[test]
    fn test_gaminv_scaling() {
        let p = 0.4;
        let a = 2.0;
        let x1 = gaminv(p, a, 1.0);
        let x2 = gaminv(p, a, 5.0);
        assert!((x2 - 5.0 * x1).abs() < 1e-14);
    }

    #[test]
    fn test_gaminv_boundaries() {
        let a = 2.0;
        let b = 1.0;
        assert_eq!(gaminv(0.0, a, b), 0.0);
        assert_eq!(gaminv(1.0, a, b), f64::INFINITY);
        // Degenerate case a=0
        assert_eq!(gaminv(0.5, 0.0, b), 0.0);
    }

    #[test]
    fn test_gaminv_invalid_params() {
        assert!(gaminv(0.5, -1.0, 1.0).is_nan());
        assert!(gaminv(0.5, 1.0, 0.0).is_nan());
        assert!(gaminv(1.1, 1.0, 1.0).is_nan());
        assert!(gaminv(-0.1, 1.0, 1.0).is_nan());
    }
}
