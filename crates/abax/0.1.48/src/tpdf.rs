use crate::{gammaln, normpdf};
use std::f64::consts::PI;

/// Student's T probability density function (PDF).
///
/// Given a value `x` and degrees of freedom `v`, this function returns
/// the probability density at `x`.
///
/// # Mathematical Definition
/// For a Student's T distribution with `v` degrees of freedom:
/// <math display="block"><mi>f</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>v</mi><mo>)</mo><mo>=</mo><mfrac><mrow><mi>Γ</mi><mo>(</mo><mfrac><mrow><mi>v</mi><mo>+</mo><mn>1</mn></mrow><mn>2</mn></mfrac><mo>)</mo></mrow><mrow><msqrt><mi>v</mi><mi>π</mi></msqrt><mi>Γ</mi><mo>(</mo><mfrac><mi>v</mi><mn>2</mn></mfrac><mo>)</mo></mrow></mfrac><msup><mrow><mo>(</mo><mn>1</mn><mo>+</mo><mfrac><msup><mi>x</mi><mn>2</mn></msup><mi>v</mi></mfrac><mo>)</mo></mrow><mrow><mo>-</mo><mfrac><mrow><mi>v</mi><mo>+</mo><mn>1</mn></mrow><mn>2</mn></mfrac></mrow></msup></math>
///
/// # Domain
/// - `v > 0`
/// - Returns `NaN` if `v <= 0` or any input is `NaN`.
/// - As `v` approaches infinity, the distribution approaches the standard normal distribution.
///
/// # Examples
/// ```
/// use abax::tpdf;
///
/// // v=1 is the standard Cauchy distribution: 1 / (pi * (1 + x^2))
/// assert!((tpdf(0.0, 1.0) - 1.0 / std::f64::consts::PI).abs() < 1e-15);
/// ```
pub fn tpdf(x: f64, v: f64) -> f64 {
    if x.is_nan() || v.is_nan() || v <= 0.0 {
        return f64::NAN;
    }

    if v.is_infinite() {
        // approaches normal as v -> Inf
        return normpdf(x, 0.0, 1.0);
    }

    // Use gammaln function to avoid overflows.
    let term = (gammaln((v + 1.0) / 2.0) - gammaln(v / 2.0)).exp();
    term / ((v * PI).sqrt() * (1.0 + (x * x) / v).powf((v + 1.0) / 2.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normpdf;

    #[test]
    fn test_tpdf_cauchy() {
        // v = 1 is Cauchy distribution
        let x = 1.5;
        let expected = 1.0 / (PI * (1.0 + x * x));
        assert!((tpdf(x, 1.0) - expected).abs() < 1e-15);

        let x = 0.0;
        let expected = 1.0 / PI;
        assert!((tpdf(x, 1.0) - expected).abs() < 1e-15);
    }

    #[test]
    fn test_tpdf_infinite_df() {
        // v = Inf should match standard normal
        let x = 0.5;
        assert_eq!(tpdf(x, f64::INFINITY), normpdf(x, 0.0, 1.0));
    }

    #[test]
    fn test_tpdf_large_df() {
        // v = 100 should be very close to standard normal
        let x = 1.0;
        let t = tpdf(x, 100.0);
        let n = 2.4076589692854598e-1;
        assert!((t - n).abs() < 1e-14);
    }

    #[test]
    fn test_tpdf_invalid_df() {
        assert!(tpdf(1.0, 0.0).is_nan());
        assert!(tpdf(1.0, -1.0).is_nan());
        assert!(tpdf(f64::NAN, 1.0).is_nan());
        assert!(tpdf(1.0, f64::NAN).is_nan());
    }

    #[test]
    fn test_tpdf_symmetry() {
        assert_eq!(tpdf(1.0, 5.0), tpdf(-1.0, 5.0));
        assert_eq!(tpdf(2.5, 2.0), tpdf(-2.5, 2.0));
    }

    #[test]
    fn test_tpdf_known_value() {
        // Reference value for v=5, x=1
        let val = tpdf(1.0, 5.0);
        assert!((val - 2.1967979735098057e-1).abs() < 1e-12);
    }
}