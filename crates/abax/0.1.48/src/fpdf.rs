use crate::{betaln, chi2pdf};

/// F probability density function (PDF).
///
/// Returns the probability density at `x` for the F-distribution with `v1` 
/// and `v2` degrees of freedom.
///
/// # Mathematical Definition
/// The PDF for the F-distribution is:
/// <math display="block">
///   <mi>f</mi><mo>(</mo><mi>x</mi><mo>;</mo><msub><mi>ν</mi><mn>1</mn></msub><mo>,</mo><msub><mi>ν</mi><mn>2</mn></msub><mo>)</mo>
///   <mo>=</mo>
///   <mfrac><mn>1</mn><mrow><mi>B</mi><mo>(</mo><mfrac><msub><mi>ν</mi><mn>1</mn></msub><mn>2</mn></mfrac><mo>,</mo><mfrac><msub><mi>ν</mi><mn>2</mn></msub><mn>2</mn></mfrac><mo>)</mo></mrow></mfrac>
///   <msup><mrow><mo>(</mo><mfrac><msub><mi>ν</mi><mn>1</mn></msub><msub><mi>ν</mi><mn>2</mn></msub></mfrac><mo>)</mo></mrow><mfrac><msub><mi>ν</mi><mn>1</mn></msub><mn>2</mn></mfrac></msup>
///   <msup><mi>x</mi><mrow><mfrac><msub><mi>ν</mi><mn>1</mn></msub><mn>2</mn></mfrac><mo>-</mo><mn>1</mn></mrow></msup>
///   <msup><mrow><mo>(</mo><mn>1</mn><mo>+</mo><mfrac><msub><mi>ν</mi><mn>1</mn></msub><msub><mi>ν</mi><mn>2</mn></msub></mfrac><mi>x</mi><mo>)</mo></mrow><mrow><mo>-</mo><mfrac><mrow><msub><mi>ν</mi><mn>1</mn></msub><mo>+</mo><msub><mi>ν</mi><mn>2</mn></msub></mrow><mn>2</mn></mfrac></mrow></msup>
/// </math>
/// for <math><mi>x</mi><mo>&gt;</mo><mn>0</mn></math>.
///
/// # Domain
/// - <math><msub><mi>ν</mi><mn>1</mn></msub><mo>&gt;</mo><mn>0</mn></math>
/// - <math><msub><mi>ν</mi><mn>2</mn></msub><mo>&gt;</mo><mn>0</mn></math>
/// - Returns `NaN` if degrees of freedom are non-positive or if `x` is `NaN`.
/// - Returns `0.0` for <math><mi>x</mi><mo>&lt;</mo><mn>0</mn></math>.
///
/// # Special Cases
/// - If <math><msub><mi>ν</mi><mn>1</mn></msub><mo>=</mo><mi>∞</mi></math> and <math><msub><mi>ν</mi><mn>2</mn></msub></math> is finite, the distribution of <math><msub><mi>ν</mi><mn>2</mn></msub><mo>/</mo><mi>X</mi></math> approaches <math><msup><mi>χ</mi><mn>2</mn></msup><mo>(</mo><msub><mi>ν</mi><mn>2</mn></msub><mo>)</mo></math>.
/// - If <math><msub><mi>ν</mi><mn>2</mn></msub><mo>=</mo><mi>∞</mi></math> and <math><msub><mi>ν</mi><mn>1</mn></msub></math> is finite, the distribution of <math><msub><mi>ν</mi><mn>1</mn></msub><mi>X</mi></math> approaches <math><msup><mi>χ</mi><mn>2</mn></msup><mo>(</mo><msub><mi>ν</mi><mn>1</mn></msub><mo>)</mo></math>.
/// - If both approach infinity, the distribution is degenerate at <math><mi>x</mi><mo>=</mo><mn>1</mn></math>.
///
/// # Examples
/// ```
/// use abax::fpdf;
///
/// // v1=2, v2=2 simplifies to f(x) = (1+x)^-2
/// let pdf = fpdf(1.0, 2.0, 2.0);
/// assert!((pdf - 0.25).abs() < 1e-15);
///
/// // x=0 behavior depends on v1
/// assert_eq!(fpdf(0.0, 1.0, 5.0), f64::INFINITY);
/// assert_eq!(fpdf(0.0, 2.0, 5.0), 1.0);
/// assert_eq!(fpdf(0.0, 3.0, 5.0), 0.0);
/// ```
pub fn fpdf(x: f64, v1: f64, v2: f64) -> f64 {
    if x < 0.0 {
        return 0.0;
    }

    if v1 <= 0.0 || v2 <= 0.0 || x.is_nan() || v1.is_nan() || v2.is_nan() {
        return f64::NAN;
    }

    if (x > 0.0 && x < f64::INFINITY) && (v1 > 0.0 && v1 < f64::INFINITY) && (v2 > 0.0 && v2 < f64::INFINITY) {
        let xk = x;
        let ratio = v1 * xk / v2;
        let logtemp = -xk.ln() - 0.5 * v1 * f64::ln_1p(1.0 / ratio) - 0.5 * v2 * f64::ln_1p(ratio) - betaln(v1 / 2.0, v2 / 2.0);
        return f64::exp(logtemp);
    }
    
    if ((v1 > 1.0e8 && v1 < f64::INFINITY) || (v2 > 1.0e8 && v2 < f64::INFINITY)) && (x > 0.0 && x < f64::INFINITY) {
        return f64::NAN;
    }

    if x.is_infinite() && v1 > 0.0 && v2 > 0.0 {
        return 0.0;
    }

    if x == 0.0 && v1 > 0.0 && v2 > 0.0 {
        match v1 {
            2.0 => return 1.0,
            _ if v1 < 2.0 => return f64::INFINITY,
            _ => return 0.0,
        }
    }

    if v2 > 0.0 && v2 < f64::INFINITY && v1 == f64::INFINITY && x > 0.0 && x < f64::INFINITY {
        return v2 / x / x * chi2pdf(v2 / x, v2);
    }
        
    if v1 > 0.0 && v1 < f64::INFINITY && v2 == f64::INFINITY && x > 0.0 && x < f64::INFINITY {
        return v1 * chi2pdf(v1 * x, v1);
    }

    if v1 == f64::INFINITY && v2 == f64::INFINITY && x > 0.0 && x < f64::INFINITY {
        return if x == 1.0 { f64::INFINITY } else { 0.0 };
    }

    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fpdf_basic() {
        let tol = 1e-15;
        // v1=2, v2=2, x=1 -> 1/(1+1)^2 = 0.25
        assert!((fpdf(1.0, 2.0, 2.0) - 0.25).abs() < tol);

        // Reference values from SciPy: scipy.stats.f.pdf(2, 5, 10)
        assert!((fpdf(2.0, 5.0, 10.0) - 1.620057421801150e-01).abs() < 1e-14);
        // scipy.stats.f.pdf(2, 10, 5)
        assert!((fpdf(2.0, 10.0, 5.0) - 1.719017506926560e-01).abs() < 1e-14);
    }

    #[test]
    fn test_fpdf_x_zero() {
        assert_eq!(fpdf(0.0, 1.0, 2.0), f64::INFINITY);
        assert_eq!(fpdf(0.0, 2.0, 2.0), 1.0);
        assert_eq!(fpdf(0.0, 4.0, 2.0), 0.0);
    }

    #[test]
    fn test_fpdf_boundaries() {
        assert_eq!(fpdf(f64::INFINITY, 5.0, 5.0), 0.0);
        assert_eq!(fpdf(-1.0, 5.0, 5.0), 0.0);
    }

    #[test]
    fn test_fpdf_invalid_params() {
        assert!(fpdf(1.0, 0.0, 5.0).is_nan());
        assert!(fpdf(1.0, 5.0, -1.0).is_nan());
        assert!(fpdf(f64::NAN, 5.0, 5.0).is_nan());
    }

    #[test]
    fn test_fpdf_infinity_v1() {
        // v1 = Inf, v2 = 5. PDF is v2/x^2 * chi2pdf(v2/x, v2)
        let x = 2.0;
        let v2 = 5.0;
        let val = fpdf(x, f64::INFINITY, v2);
        let expected = (v2 / (x * x)) * chi2pdf(v2 / x, v2);
        assert!((val - expected).abs() < 1e-15);
    }

    #[test]
    fn test_fpdf_infinity_v2() {
        // v1 = 5, v2 = Inf. PDF is v1 * chi2pdf(v1 * x, v1)
        let x = 0.5;
        let v1 = 5.0;
        let val = fpdf(x, v1, f64::INFINITY);
        let expected = v1 * chi2pdf(v1 * x, v1);
        assert!((val - expected).abs() < 1e-15);
    }

    #[test]
    fn test_fpdf_infinity_both() {
        assert_eq!(fpdf(1.0, f64::INFINITY, f64::INFINITY), f64::INFINITY);
        assert_eq!(fpdf(0.5, f64::INFINITY, f64::INFINITY), 0.0);
        assert_eq!(fpdf(1.5, f64::INFINITY, f64::INFINITY), 0.0);
    }
}
