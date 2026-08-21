/// Calculates the probability density function (PDF) of the exponential distribution
/// at a given value `x` with scale parameter `mu`.
///
/// The exponential distribution PDF is defined as:
///
/// <math xmlns="http://www.w3.org/1998/Math/MathML">
///   <mrow>
///     <mi>f</mi>
///     <mo>(</mo>
///     <mi>x</mi>
///     <mo>;</mo>
///     <mi>μ</mi>
///     <mo>)</mo>
///     <mo>=</mo>
///     <mfrac>
///       <mn>1</mn>
///       <mi>μ</mi>
///     </mfrac>
///     <msup>
///       <mi>e</mi>
///       <mrow>
///         <mo>-</mo>
///         <mfrac>
///           <mi>x</mi>
///           <mi>μ</mi>
///         </mfrac>
///       </mrow>
///     </msup>
///   </mrow>
/// </math>
///
/// for <math xmlns="http://www.w3.org/1998/Math/MathML"><mi>x</mi><mo>≥</mo><mn>0</mn></math>
/// and <math xmlns="http://www.w3.org/1998/Math/MathML"><mi>μ</mi><mo>></mo><mn>0</mn></math>,
/// where μ is the mean (scale parameter) of the distribution.
///
/// Note that the rate parameter
/// <math xmlns="http://www.w3.org/1998/Math/MathML"><mi>λ</mi><mo>=</mo><mfrac><mn>1</mn><mi>μ</mi></mfrac></math>.
///
/// # Parameters
///
/// * `x` (`f64`) - The value at which to evaluate the PDF.
///   **Valid range:** any real number. Negative values return `0.0`.
///
/// * `mu` (`f64`) - The mean (scale) parameter of the distribution.
///   **Valid range:**
///   <math xmlns="http://www.w3.org/1998/Math/MathML"><mi>μ</mi><mo>></mo><mn>0</mn></math>.
///   Values of <math xmlns="http://www.w3.org/1998/Math/MathML"><mi>μ</mi><mo>≤</mo><mn>0</mn></math>
///   are invalid and will return `f64::NAN`.
///
/// # Returns
///
/// Returns the probability density at `x` as a `f64`:
/// * For
///   <math xmlns="http://www.w3.org/1998/Math/MathML"><mi>x</mi><mo>≥</mo><mn>0</mn></math>
///   and
///   <math xmlns="http://www.w3.org/1998/Math/MathML"><mi>μ</mi><mo>></mo><mn>0</mn></math>:
///   returns
///   <math xmlns="http://www.w3.org/1998/Math/MathML"><mfrac><mn>1</mn><mi>μ</mi></mfrac><msup><mi>e</mi><mrow><mo>-</mo><mi>x</mi><mo>/</mo><mi>μ</mi></mrow></msup></math>
/// * For
///   <math xmlns="http://www.w3.org/1998/Math/MathML"><mi>x</mi><mo><</mo><mn>0</mn></math>
///   (any μ): returns `0.0`
/// * For
///   <math xmlns="http://www.w3.org/1998/Math/MathML"><mi>μ</mi><mo>≤</mo><mn>0</mn></math>
///   (any x): returns `f64::NAN`
///
/// # Examples
///
/// ```
/// use abax::exppdf;
/// // PDF at x = 1.0 with mean mu = 2.0
/// let pdf = exppdf(1.0, 2.0);
/// assert!((pdf - 0.3032653298563167).abs() < 1e-10);
///
/// // PDF at x = 0 is 1/mu
/// let pdf = exppdf(0.0, 3.0);
/// assert!((pdf - 1.0/3.0).abs() < 1e-10);
///
/// // Negative x returns 0
/// assert_eq!(exppdf(-1.0, 2.0), 0.0);
///
/// // Invalid mu returns NaN
/// assert!(exppdf(1.0, -1.0).is_nan());
/// assert!(exppdf(1.0, 0.0).is_nan());
/// 
/// // Large positive x produces very small but non-zero values
/// let pdf = exppdf(10.0, 1.0);
/// assert!(pdf > 0.0 && pdf < 1e-4);
/// ```
pub fn exppdf(x: f64, mu: f64) -> f64 {
    if mu <= 0.0 {
        return f64::NAN;
    }
    if x < 0.0 {
        return 0.0;
    }
    let z = x / mu;
    f64::exp(-z) / mu
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exppdf_standard_case() {
        // Test with mu = 1.0 (standard exponential)
        let result = exppdf(0.5, 1.0);
        let expected = (-0.5f64).exp();
        assert!((result - expected).abs() < 1e-10);
    }

    #[test]
    fn test_exppdf_zero_x() {
        // At x = 0, PDF should be 1/mu
        let mu = 2.5;
        let result = exppdf(0.0, mu);
        assert!((result - 1.0 / mu).abs() < 1e-10);
    }

    #[test]
    fn test_exppdf_various_values() {
        // Test several known values
        let test_cases = vec![
            (0.0, 1.0, 1.0),
            (1.0, 2.0, 0.3032653298563167),
            (2.0, 3.0, 0.1711390396995308),
            (0.5, 0.5, 0.7357588823428847),
            (3.0, 1.5, 0.09022352215741275),
        ];

        for (x, mu, expected) in test_cases {
            let result = exppdf(x, mu);
            assert!(
                (result - expected).abs() < 1e-10,
                "Failed for x={}, mu={}: got {}, expected {}",
                x,
                mu,
                result,
                expected
            );
        }
    }

    #[test]
    fn test_exppdf_negative_x() {
        // Negative x should return 0
        assert_eq!(exppdf(-0.5, 1.0), 0.0);
        assert_eq!(exppdf(-1.0, 2.5), 0.0);
        assert_eq!(exppdf(-100.0, 3.0), 0.0);
    }

    #[test]
    fn test_exppdf_invalid_mu() {
        // mu <= 0 should return NaN
        assert!(exppdf(1.0, 0.0).is_nan());
        assert!(exppdf(1.0, -1.0).is_nan());
        assert!(exppdf(1.0, -0.5).is_nan());
        assert!(exppdf(0.0, -1.0).is_nan());
        assert!(exppdf(-1.0, -1.0).is_nan());
    }

    #[test]
    fn test_exppdf_large_x() {
        // Large x should produce very small but positive values
        let result = exppdf(20.0, 1.0);
        assert!(result > 0.0);
        assert!(result < 1e-8);
        assert!((result - (-20.0f64).exp()).abs() < 1e-15);
    }

    #[test]
    fn test_exppdf_small_mu() {
        // Very small mu (but positive) should work
        let result = exppdf(0.001, 0.001);
        assert!((result - 1.0 / (0.001 * std::f64::consts::E)).abs() < 1e-10);
    }

    #[test]
    fn test_exppdf_consistency() {
        // The PDF should be decreasing for fixed mu
        let mu = 2.0;
        let pdf0 = exppdf(0.0, mu);
        let pdf1 = exppdf(1.0, mu);
        let pdf2 = exppdf(2.0, mu);
        assert!(pdf0 > pdf1);
        assert!(pdf1 > pdf2);
    }
}