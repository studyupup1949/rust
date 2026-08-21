use crate::{betainc, gammainc};

/// F cumulative distribution function (CDF).
///
/// Returns the cumulative distribution function at `x` for the F-distribution 
/// with `v1` and `v2` degrees of freedom.
///
/// # Mathematical Definition
/// The CDF for the F-distribution is:
/// <math display="block">
///   <mi>F</mi><mo>(</mo><mi>x</mi><mo>;</mo><msub><mi>ν</mi><mn>1</mn></msub><mo>,</mo><msub><mi>ν</mi><mn>2</mn></msub><mo>)</mo>
///   <mo>=</mo>
///   <msub><mi>I</mi><mfrac><mrow><msub><mi>ν</mi><mn>1</mn></msub><mi>x</mi></mrow><mrow><msub><mi>ν</mi><mn>1</mn></msub><mi>x</mi><mo>+</mo><msub><mi>ν</mi><mn>2</mn></msub></mrow></mfrac></msub>
///   <mo>(</mo><mfrac><msub><mi>ν</mi><mn>1</mn></msub><mn>2</mn></mfrac><mo>,</mo><mfrac><msub><mi>ν</mi><mn>2</mn></msub><mn>2</mn></mfrac><mo>)</mo>
/// </math>
/// for <math><mi>x</mi><mo>></mo><mn>0</mn></math>, where <math><msub><mi>I</mi><mi>z</mi></msub><mo>(</mo><mi>a</mi><mo>,</mo><mi>b</mi><mo>)</mo></math> is the 
/// regularized incomplete beta function.
///
/// The upper tail probability (survival function) is computed using the symmetry property:
/// <math display="block">
///   <mi>Q</mi><mo>(</mo><mi>x</mi><mo>;</mo><msub><mi>ν</mi><mn>1</mn></msub><mo>,</mo><msub><mi>ν</mi><mn>2</mn></msub><mo>)</mo>
///   <mo>=</mo>
///   <mi>F</mi><mo>(</mo><mfrac><mn>1</mn><mi>x</mi></mfrac><mo>;</mo><msub><mi>ν</mi><mn>2</mn></msub><mo>,</mo><msub><mi>ν</mi><mn>1</mn></msub><mo>)</mo>
/// </math>
///
/// # Domain
/// - <math><msub><mi>ν</mi><mn>1</mn></msub><mo>&gt;</mo><mn>0</mn></math>
/// - <math><msub><mi>ν</mi><mn>2</mn></msub><mo>&gt;</mo><mn>0</mn></math>
/// - Returns `NaN` if degrees of freedom are non-positive or if `x` is `NaN`.
/// - Returns `0.0` for <math><mi>x</mi><mo>≤</mo><mn>0</mn></math> (lower tail) or `1.0` (upper tail).
///
/// # Special Cases
/// - If <math><msub><mi>ν</mi><mn>2</mn></msub><mo>=</mo><mi>∞</mi></math> and <math><msub><mi>ν</mi><mn>1</mn></msub></math> is finite, the distribution of <math><msub><mi>ν</mi><mn>1</mn></msub><mi>X</mi></math> approaches <math><msup><mi>χ</mi><mn>2</mn></msup><mo>(</mo><msub><mi>ν</mi><mn>1</mn></msub><mo>)</mo></math>.
/// - If <math><msub><mi>ν</mi><mn>1</mn></msub><mo>=</mo><mi>∞</mi></math> and <math><msub><mi>ν</mi><mn>2</mn></msub></math> is finite, the distribution of <math><msub><mi>ν</mi><mn>2</mn></msub><mo>/</mo><mi>X</mi></math> approaches <math><msup><mi>χ</mi><mn>2</mn></msup><mo>(</mo><msub><mi>ν</mi><mn>2</mn></msub><mo>)</mo></math>.
/// - If both approach infinity, the distribution is degenerate at <math><mi>x</mi><mo>=</mo><mn>1</mn></math>.
///
/// # Examples
/// ```
/// use abax::fcdf;
///
/// // v1=2, v2=2 simplifies to F(x) = x / (1 + x)
/// let p = fcdf(1.0, 2.0, 2.0, false);
/// assert!((p - 0.5).abs() < 1e-15);
///
/// // Upper tail: P(X > 1.0)
/// let q = fcdf(1.0, 2.0, 2.0, true);
/// assert!((q - 0.5).abs() < 1e-15);
/// ```
pub fn fcdf(x: f64, v1: f64, v2: f64, upper: bool) -> f64 {
    let x = if upper { 1.0 / f64::max(0.0, x) } else { x };
    let (v1, v2) = if upper { (v2, v1) } else { (v1, v2) };

    if v1 <= 0.0 || v2 <= 0.0 || x.is_nan() || v1.is_nan() || v2.is_nan() {
        return f64::NAN;
    }

    if x == f64::INFINITY {
        return 1.0;
    }

    if x > 0.0 && v1.is_finite() && v2.is_finite() {
        match v2 <= x * v1 {
            true  => {
                return betainc(v2 / (v2 + x * v1), v2 / 2.0, v1 / 2.0, false);
            },
            false => {
                let num = v1 * x;
                let xx = num / (num + v2);
                return betainc(xx, v1 / 2.0, v2 / 2.0, true);
            },
        }
    }

    if !v1.is_finite() || !v2.is_finite() {
        if x > 0.0 && v1.is_finite() && !v2.is_finite() && v2 > 0.0 {
            return gammainc(v1 * x / 2.0, v1 / 2.0, true, false);
        }
        if x > 0.0 && !v1.is_finite() && v1 > 0.0 && v2.is_finite() {
            return gammainc(v2 / x / 2.0, v2 / 2.0, false, false);
        }
        if x > 0.0 && !v1.is_finite() && v1 > 0.0 && !v2.is_finite() && v2 > 0.0 {
            if upper && x == 1.0 {
                return 0.0;
            } else {
                return if x >= 1.0 { 1.0 } else { 0.0 }
            }
        }
    }

    0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chi2cdf;

    #[test]
    fn test_fcdf_basic() {
        let tol = 1e-15;
        // v1=2, v2=2, x=1 -> 1 / (1 + 1) = 0.5
        assert!((fcdf(1.0, 2.0, 2.0, false) - 0.5).abs() < tol);
        // v1=2, v2=2, x=2 -> 2 / (1 + 2) = 2/3
        assert!((fcdf(2.0, 2.0, 2.0, false) - 2.0 / 3.0).abs() < tol);

        assert!((fcdf(2.0, 5.0, 10.0, false) - 8.358050491002613e-01).abs() < 1e-14);
        assert!((fcdf(2.0, 10.0, 5.0, false) - 7.700248806501014e-01).abs() < 1e-14);
    }

    #[test]
    fn test_fcdf_upper() {
        let tol = 1e-14;
        // P(X > 2.0 | 5, 10)
        let q = fcdf(2.0, 5.0, 10.0, true);
        let p = fcdf(2.0, 5.0, 10.0, false);
        assert!((p + q - 1.0).abs() < tol);

        // SciPy: scipy.stats.f.sf(2, 5, 10)
        assert!((q - 1.641949508997387e-01).abs() < tol);
    }

    #[test]
    fn test_fcdf_x_zero() {
        assert_eq!(fcdf(0.0, 1.0, 2.0, false), 0.0);
        assert_eq!(fcdf(0.0, 1.0, 2.0, true), 1.0);
    }

    #[test]
    fn test_fcdf_boundaries() {
        assert_eq!(fcdf(f64::INFINITY, 5.0, 5.0, false), 1.0);
        assert_eq!(fcdf(f64::INFINITY, 5.0, 5.0, true), 0.0);
        assert_eq!(fcdf(-1.0, 5.0, 5.0, false), 0.0);
        assert_eq!(fcdf(-1.0, 5.0, 5.0, true), 1.0);
    }

    #[test]
    fn test_fcdf_invalid_params() {
        assert!(fcdf(1.0, 0.0, 5.0, false).is_nan());
        assert!(fcdf(1.0, 5.0, -1.0, false).is_nan());
        assert!(fcdf(f64::NAN, 5.0, 5.0, false).is_nan());
    }

    #[test]
    fn test_fcdf_infinity_v1() {
        // v1 = Inf, v2 = 5. CDF is chi2cdf(v2/x, v2, upper=true)
        let x = 2.0;
        let v2 = 5.0;
        let val = fcdf(x, f64::INFINITY, v2, false);
        let expected = chi2cdf(v2 / x, v2, true);
        assert!((val - expected).abs() < 1e-15);
    }

    #[test]
    fn test_fcdf_infinity_v2() {
        // v1 = 5, v2 = Inf. CDF is chi2cdf(v1 * x, v1)
        let x = 0.5;
        let v1 = 5.0;
        let val = fcdf(x, v1, f64::INFINITY, false);
        let expected = chi2cdf(v1 * x, v1, false);
        assert!((val - expected).abs() < 1e-15);
    }

    #[test]
    fn test_fcdf_infinity_both() {
        assert_eq!(fcdf(1.0, f64::INFINITY, f64::INFINITY, false), 1.0);
        assert_eq!(fcdf(0.9, f64::INFINITY, f64::INFINITY, false), 0.0);
        assert_eq!(fcdf(1.1, f64::INFINITY, f64::INFINITY, false), 1.0);
    }
}
