use crate::{chi2inv, betainc, betaincinv};
/// F inverse cumulative distribution function (quantile function).
///
/// Returns the quantile at probability `p` for the F-distribution with `v1` 
/// and `v2` degrees of freedom.
///
/// # Mathematical Definition
/// The F quantile function is the inverse of the CDF. It is related to the
/// inverse regularized incomplete beta function <math><msubsup><mi>I</mi><mi>p</mi><mrow><mo>-</mo><mn>1</mn></mrow></msubsup><mo>(</mo><mi>a</mi><mo>,</mo><mi>b</mi><mo>)</mo></math>:
/// <math display="block">
///   <mi>x</mi><mo>=</mo><mfrac><msub><mi>ν</mi><mn>2</mn></msub><msub><mi>ν</mi><mn>1</mn></msub></mfrac>
///   <mfrac>
///     <mrow>
///       <msubsup><mi>I</mi><mi>p</mi><mrow><mo>-</mo><mn>1</mn></mrow></msubsup><mo>(</mo><mfrac><msub><mi>ν</mi><mn>1</mn></msub><mn>2</mn></mfrac><mo>,</mo><mfrac><msub><mi>ν</mi><mn>2</mn></msub><mn>2</mn></mfrac><mo>)</mo>
///     </mrow>
///     <mrow>
///       <mn>1</mn><mo>-</mo><msubsup><mi>I</mi><mi>p</mi><mrow><mo>-</mo><mn>1</mn></mrow></msubsup><mo>(</mo><mfrac><msub><mi>ν</mi><mn>1</mn></msub><mn>2</mn></mfrac><mo>,</mo><mfrac><msub><mi>ν</mi><mn>2</mn></msub><mn>2</mn></mfrac><mo>)</mo>
///     </mrow>
///   </mfrac>
/// </math>
///
/// # Domain
/// - <math><mn>0</mn><mo>≤</mo><mi>p</mi><mo>≤</mo><mn>1</mn></math>
/// - <math><msub><mi>ν</mi><mn>1</mn></msub><mo>&gt;</mo><mn>0</mn></math>
/// - <math><msub><mi>ν</mi><mn>2</mn></msub><mo>&gt;</mo><mn>0</mn></math>
/// - Returns `NaN` if probability is out of range or degrees of freedom are non-positive.
///
/// # Special Cases
/// - If <math><msub><mi>ν</mi><mn>1</mn></msub><mo>=</mo><mi>∞</mi></math>, the quantile is computed via the inverse 
///   Chi-squared distribution: <math><mi>x</mi><mo>=</mo><msub><mi>ν</mi><mn>2</mn></msub><mo>/</mo><msubsup><mi>χ</mi><mtext>inv</mtext><mn>2</mn></msubsup><mo>(</mo><mn>1</mn><mo>-</mo><mi>p</mi><mo>,</mo><msub><mi>ν</mi><mn>2</mn></msub><mo>)</mo></math>.
/// - If <math><msub><mi>ν</mi><mn>2</mn></msub><mo>=</mo><mi>∞</mi></math>, the quantile is computed via the inverse 
///   Chi-squared distribution: <math><mi>x</mi><mo>=</mo><msubsup><mi>χ</mi><mtext>inv</mtext><mn>2</mn></msubsup><mo>(</mo><mi>p</mi><mo>,</mo><msub><mi>ν</mi><mn>1</mn></msub><mo>)</mo><mo>/</mo><msub><mi>ν</mi><mn>1</mn></msub></math>.
/// - If both approach infinity, the quantile is <math><mn>0.0</mn></math> at <math><mi>p</mi><mo>=</mo><mn>0</mn></math> 
///   and <math><mn>1.0</mn></math> for <math><mi>p</mi><mo>&gt;</mo><mn>0</mn></math>.
///
/// # Examples
/// ```
/// use abax::finv;
///
/// // v1=2, v2=2 simplifies to x = p / (1 - p)
/// let x = finv(0.5, 2.0, 2.0);
/// assert!((x - 1.0).abs() < 1e-15);
///
/// // Standard 95% critical value for (5, 10) degrees of freedom
/// let x95 = finv(0.95, 5.0, 10.0);
/// assert!((x95 - 3.325834530413011).abs() < 1e-14);
/// ```
pub fn finv(p: f64, v1: f64, v2: f64) -> f64 {
    if p < 0.0 || 1.0 < p || p.is_nan() || v1.is_nan() || v2.is_nan() || v1 <= 0.0 || v2 <= 0.0 {
        return f64::NAN;
    }

    match (v1, v2) {
        (f64::INFINITY, f64::INFINITY) => return if p == 0.0 { 0.0 } else { 1.0 },
        (f64::INFINITY, _) if v2 < f64::INFINITY => return v2 / chi2inv(1.0 - p, v2),
        (_, f64::INFINITY) if v1 < f64::INFINITY => return chi2inv(p, v1) / v1,
        _ => {
            // Get the smaller of z or 1-z to give the best precision
            let up = p > betainc(0.5, v1 / 2.0, v2 / 2.0, true);
            let t = if up {
                let z_up = betaincinv(p, v2 / 2.0, v1 / 2.0, false);
                (1.0 - z_up) / z_up
            } else {
                let z_lo = betaincinv(p, v1 / 2.0, v2 / 2.0, true);
                z_lo / (1.0 - z_lo)
            };
            return t * v2 / v1;
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fcdf;

    #[test]
    fn test_finv_basic() {
        let tol = 1e-15;
        // v1=2, v2=2, p=0.5 -> x = 0.5 / 0.5 = 1.0
        assert!((finv(0.5, 2.0, 2.0) - 1.0).abs() < tol);
        // p=0.75 -> x = 0.75 / 0.25 = 3.0
        assert!((finv(0.75, 2.0, 2.0) - 3.0).abs() < tol);

        assert!((finv(0.95, 5.0, 10.0) - 3.325834530413011).abs() < 1e-14);
    }

    #[test]
    fn test_finv_roundtrip() {
        let v1 = 5.0;
        let v2 = 10.0;
        let probabilities = [0.01, 0.1, 0.5, 0.9, 0.99];
        for &p in &probabilities {
            let x = finv(p, v1, v2);
            let p_back = fcdf(x, v1, v2, false);
            assert!((p - p_back).abs() < 1e-12, "Roundtrip failed for p={}", p);
        }
    }

    #[test]
    fn test_finv_boundaries() {
        assert_eq!(finv(0.0, 5.0, 5.0), 0.0);
        assert_eq!(finv(1.0, 5.0, 5.0), f64::INFINITY);
    }

    #[test]
    fn test_finv_invalid_params() {
        assert!(finv(0.5, 0.0, 5.0).is_nan());
        assert!(finv(0.5, 5.0, -1.0).is_nan());
        assert!(finv(-0.1, 5.0, 5.0).is_nan());
        assert!(finv(1.1, 5.0, 5.0).is_nan());
        assert!(finv(f64::NAN, 5.0, 5.0).is_nan());
    }

    #[test]
    fn test_finv_infinity() {
        let p = 0.95;
        let v = 5.0;

        // v1 = Inf, v2 = 5
        // x = v2 / chi2inv(1-p, v2)
        let x1 = finv(p, f64::INFINITY, v);
        let expected1 = v / chi2inv(1.0 - p, v);
        assert!((x1 - expected1).abs() < 1e-15);

        // v1 = 5, v2 = Inf
        // x = chi2inv(p, v1) / v1
        let x2 = finv(p, v, f64::INFINITY);
        let expected2 = chi2inv(p, v) / v;
        assert!((x2 - expected2).abs() < 1e-15);

        // Both Inf: degenerate at x=1
        assert_eq!(finv(0.5, f64::INFINITY, f64::INFINITY), 1.0);
        assert_eq!(finv(0.0, f64::INFINITY, f64::INFINITY), 0.0);
    }
}
