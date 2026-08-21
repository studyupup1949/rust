use crate::gamcdf;

/// Chi-squared cumulative distribution function (CDF).
///
/// Returns the probability that a Chi-squared random variable with <math><mi>v</mi></math>
/// degrees of freedom is less than or equal to <math><mi>x</mi></math>.
///
/// # Mathematical Definition
/// For a Chi-squared distribution with <math><mi>v</mi></math> degrees of freedom:
/// - Lower tail (`upper = false`):
/// <math display="block">
///   <mi>F</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>v</mi><mo>)</mo>
///   <mo>=</mo>
///   <mi>P</mi><mo>(</mo><mi>v</mi><mo>/</mo><mn>2</mn><mo>,</mo><mi>x</mi><mo>/</mo><mn>2</mn><mo>)</mo>
///   <mo>=</mo>
///   <mfrac><mn>1</mn><mrow><mi>Γ</mi><mo>(</mo><mi>v</mi><mo>/</mo><mn>2</mn><mo>)</mo></mrow></mfrac>
///   <msubsup><mo>∫</mo><mn>0</mn><mrow><mi>x</mi><mo>/</mo><mn>2</mn></mrow></msubsup>
///   <msup><mi>t</mi><mrow><mi>v</mi><mo>/</mo><mn>2</mn><mo>-</mo><mn>1</mn></mrow></msup><msup><mi>e</mi><mrow><mo>-</mo><mi>t</mi></mrow></msup><mi>dt</mi>
/// </math>
/// - Upper tail (`upper = true`):
/// <math display="block">
///   <mn>1</mn><mo>-</mo><mi>F</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>v</mi><mo>)</mo>
///   <mo>=</mo>
///   <mi>Q</mi><mo>(</mo><mi>v</mi><mo>/</mo><mn>2</mn><mo>,</mo><mi>x</mi><mo>/</mo><mn>2</mn><mo>)</mo>
/// </math>
///
/// # Examples
/// ```
/// use abax::chi2cdf;
///
/// // For v=2, the distribution is Exponential with mean 2: F(x) = 1 - exp(-x/2)
/// let p = chi2cdf(2.0, 2.0, false);
/// assert!((p - (1.0 - (-1.0f64).exp())).abs() < 1e-15);
/// ```
pub fn chi2cdf(x: f64, v: f64, upper: bool) -> f64 {
    gamcdf(x, v / 2.0, 2.0, upper)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chi2cdf_v2_exponential() {
        // v = 2: F(x) = 1 - exp(-x/2)
        let x = 2.0;
        let v = 2.0;
        let p = chi2cdf(x, v, false);
        let expected = 1.0 - (-1.0f64).exp();
        assert!((p - expected).abs() < 1e-15);
    }

    #[test]
    fn test_chi2cdf_tail_consistency() {
        let x = 3.0;
        let v = 4.5;
        let p = chi2cdf(x, v, false);
        let q = chi2cdf(x, v, true);
        assert!((p + q - 1.0).abs() < 1e-15);
    }

    #[test]
    fn test_chi2cdf_boundaries() {
        let v = 2.0;
        assert_eq!(chi2cdf(0.0, v, false), 0.0);
        assert_eq!(chi2cdf(0.0, v, true), 1.0);
        assert_eq!(chi2cdf(-1.0, v, false), 0.0);
        assert_eq!(chi2cdf(-1.0, v, true), 1.0);
    }

    #[test]
    fn test_chi2cdf_invalid_params() {
        assert!(chi2cdf(1.0, 0.0, false).is_nan());
        assert!(chi2cdf(1.0, -1.0, false).is_nan());
    }
}
