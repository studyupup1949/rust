use crate::gammainc;

/// Gamma cumulative distribution function (CDF).
///
/// Returns the probability that a Gamma random variable with shape parameter `a`
/// and scale parameter `b` is less than or equal to `x`.
///
/// # Mathematical Definition
/// For a Gamma distribution with shape <math><mi>a</mi></math> and scale <math><mi>b</mi></math>:
/// - Lower tail (`upper = false`):
/// <math display="block">
///   <mi>F</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>a</mi><mo>,</mo><mi>b</mi><mo>)</mo>
///   <mo>=</mo>
///   <mi>P</mi><mo>(</mo><mi>a</mi><mo>,</mo><mi>x</mi><mo>/</mo><mi>b</mi><mo>)</mo>
///   <mo>=</mo>
///   <mfrac><mn>1</mn><mrow><mi>Γ</mi><mo>(</mo><mi>a</mi><mo>)</mo></mrow></mfrac>
///   <msubsup><mo>∫</mo><mn>0</mn><mrow><mi>x</mi><mo>/</mo><mi>b</mi></mrow></msubsup>
///   <msup><mi>t</mi><mrow><mi>a</mi><mo>-</mo><mn>1</mn></mrow></msup><msup><mi>e</mi><mrow><mo>-</mo><mi>t</mi></mrow></msup><mi>dt</mi>
/// </math>
/// - Upper tail (`upper = true`):
/// <math display="block">
///   <mn>1</mn><mo>-</mo><mi>F</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>a</mi><mo>,</mo><mi>b</mi><mo>)</mo>
///   <mo>=</mo>
///   <mi>Q</mi><mo>(</mo><mi>a</mi><mo>,</mo><mi>x</mi><mo>/</mo><mi>b</mi><mo>)</mo>
/// </math>
///
/// # Domain
/// - <math><mi>x</mi><mo>≥</mo><mn>0</mn></math> (Returns 0 for the lower tail if <math><mi>x</mi><mo>&lt;</mo><mn>0</mn></math>).
/// - <math><mi>a</mi><mo>&gt;</mo><mn>0</mn></math> (Shape parameter).
/// - <math><mi>b</mi><mo>&gt;</mo><mn>0</mn></math> (Scale parameter).
/// - Returns `NaN` if `a <= 0` or `b <= 0`.
///
/// # Examples
/// ```
/// use abax::gamcdf;
///
/// // For a=1, Gamma reduces to Exponential distribution: 1 - exp(-x/b)
/// let p = gamcdf(1.0, 1.0, 1.0, false);
/// assert!((p - (1.0 - (-1.0f64).exp())).abs() < 1e-15);
/// ```
pub fn gamcdf(x: f64, a: f64, b: f64, upper: bool) -> f64 {
    let a = if a < 0.0 { f64::NAN } else { a };
    let b = if b <= 0.0 { f64::NAN } else { b };
    let x = if x < 0.0 { 0.0 } else { x };

    let z = x / b;
    gammainc(z, a, !upper, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gamcdf_exponential_identity() {
        // If a=1, Gamma(1, b) is an Exponential distribution with mean b.
        // CDF is 1 - exp(-x/b).
        let a = 1.0;
        let b = 2.5;
        let x = 1.0;
        let p = gamcdf(x, a, b, false);
        let expected = 1.0 - (-x / b).exp();
        assert!((p - expected).abs() < 1e-15);
    }

    #[test]
    fn test_gamcdf_tail_consistency() {
        let x = 2.0;
        let a = 3.0;
        let b = 1.0;
        let p = gamcdf(x, a, b, false);
        let q = gamcdf(x, a, b, true);
        assert!((p + q - 1.0).abs() < 1e-15);
    }

    #[test]
    fn test_gamcdf_invalid_params() {
        assert!(gamcdf(1.0, 0.0, 1.0, false).is_nan());
        assert!(gamcdf(1.0, -1.0, 1.0, false).is_nan());
        assert!(gamcdf(1.0, 1.0, 0.0, false).is_nan());
        assert!(gamcdf(1.0, 1.0, -1.0, false).is_nan());
    }
}
