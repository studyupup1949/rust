use crate::gampdf;

/// Chi-squared probability density function (PDF).
///
/// Returns the probability density at `x` for the Chi-squared distribution
/// with <math><mi>v</mi></math> degrees of freedom.
///
/// # Mathematical Definition
/// The Chi-squared distribution with <math><mi>v</mi></math> degrees of freedom is a special case
/// of the Gamma distribution with shape <math><mi>a</mi><mo>=</mo><mi>v</mi><mo>/</mo><mn>2</mn></math> and scale <math><mi>b</mi><mo>=</mo><mn>2</mn></math>:
/// <math display="block">
///   <mi>f</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>v</mi><mo>)</mo>
///   <mo>=</mo>
///   <mfrac>
///     <mrow>
///       <msup><mi>x</mi><mrow><mi>v</mi><mo>/</mo><mn>2</mn><mo>-</mo><mn>1</mn></mrow></msup>
///       <msup><mi>e</mi><mrow><mo>-</mo><mi>x</mi><mo>/</mo><mn>2</mn></mrow></msup>
///     </mrow>
///     <mrow>
///       <msup><mn>2</mn><mrow><mi>v</mi><mo>/</mo><mn>2</mn></mrow></msup>
///       <mi>Γ</mi><mo>(</mo><mi>v</mi><mo>/</mo><mn>2</mn><mo>)</mo>
///     </mrow>
///   </mfrac>
/// </math>
/// for <math><mi>x</mi><mo>&gt;</mo><mn>0</mn></math>.
///
/// # Examples
/// ```
/// use abax::chi2pdf;
///
/// // For v=2, the distribution is Exponential with mean 2: f(x) = 0.5 * exp(-x/2)
/// let x = 2.0;
/// let pdf = chi2pdf(x, 2.0);
/// assert!((pdf - 0.5 * (-1.0f64).exp()).abs() < 1e-15);
/// ```
pub fn chi2pdf(x: f64, v: f64) -> f64 {
    gampdf(x, v / 2.0, 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chi2pdf_standard_cases() {
        let tol = 1e-14;

        // v = 2: Exponential(0.5) => 0.5 * exp(-x/2)
        // at x = 0, pdf = 0.5
        assert!((chi2pdf(0.0, 2.0) - 0.5).abs() < tol);
        // at x = 2, pdf = 0.5 * exp(-1)
        assert!((chi2pdf(2.0, 2.0) - 0.5 * (-1.0f64).exp()).abs() < tol);

        // v = 4: f(x) = 0.25 * x * exp(-x/2)
        // at x = 2, pdf = 0.25 * 2 * exp(-1) = 0.5 * exp(-1)
        assert!((chi2pdf(2.0, 4.0) - 0.5 * (-1.0f64).exp()).abs() < tol);
    }

    #[test]
    fn test_chi2pdf_v1_relation() {
        // v = 1: f(x) = (1/sqrt(2*pi)) * x^(-1/2) * exp(-x/2)
        // at x = 1, pdf = (1/sqrt(2*pi)) * exp(-0.5)
        let expected = (1.0 / (2.0 * std::f64::consts::PI).sqrt()) * (-0.5f64).exp();
        assert!((chi2pdf(1.0, 1.0) - expected).abs() < 1e-15);
    }

    #[test]
    fn test_chi2pdf_poles_at_zero() {
        // if v < 2, pdf(0) is infinity
        assert_eq!(chi2pdf(0.0, 1.0), f64::INFINITY);
        // if v = 2, pdf(0) is 0.5
        assert_eq!(chi2pdf(0.0, 2.0), 0.5);
        // if v > 2, pdf(0) is 0
        assert_eq!(chi2pdf(0.0, 3.0), 0.0);
    }

    #[test]
    fn test_chi2pdf_invalid_params() {
        assert!(chi2pdf(1.0, 0.0) == 0.0);
        assert!(chi2pdf(1.0, -1.0).is_nan());
        assert!(chi2pdf(f64::NAN, 2.0).is_nan());
        assert!(chi2pdf(1.0, f64::NAN).is_nan());
    }

    #[test]
    fn test_chi2pdf_negative_x() {
        assert_eq!(chi2pdf(-1.0, 2.0), 0.0);
    }
}
