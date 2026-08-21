/// Extreme value probability density function (PDF) for scalar values.
///
/// Evaluates the PDF of the Type 1 extreme value distribution (suitable for minima)
/// with location parameter `mu` and scale parameter `sigma` at the value `x`.
///
/// # Arguments
/// * `x` - The value at which to evaluate the PDF.
/// * `mu` - Location parameter.
/// * `sigma` - Scale parameter (sigma > 0).
///
/// # Returns
/// The probability density at `x`. Returns `f64::NAN` if parameters are invalid.
///
/// # Mathematical Formula
/// For <math xmlns="http://www.w3.org/1998/Math/MathML" display="inline"><mi>&#x03C3;</mi><mo>&gt;</mo><mn>0</mn></math>:
///
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="block">
///   <mi>f</mi>
///   <mo stretchy="false">(</mo>
///   <mi>x</mi>
///   <mo>;</mo>
///   <mi>&#x03BC;</mi>
///   <mo>,</mo>
///   <mi>&#x03C3;</mi>
///   <mo stretchy="false">)</mo>
///   <mo>=</mo>
///   <mfrac>
///     <mn>1</mn>
///     <mi>&#x03C3;</mi>
///   </mfrac>
///   <mo>&#x22C5;</mo>
///   <mi>exp</mi>
///   <mrow>
///     <mo>(</mo>
///     <mfrac>
///       <mrow>
///         <mi>x</mi>
///         <mo>&#x2212;</mo>
///         <mi>&#x03BC;</mi>
///       </mrow>
///       <mi>&#x03C3;</mi>
///     </mfrac>
///     <mo>&#x2212;</mo>
///     <mi>exp</mi>
///     <mrow>
///       <mo>(</mo>
///       <mfrac>
///         <mrow>
///           <mi>x</mi>
///           <mo>&#x2212;</mo>
///           <mi>&#x03BC;</mi>
///         </mrow>
///         <mi>&#x03C3;</mi>
///       </mfrac>
///       <mo>)</mo>
///     </mrow>
///     <mo>)</mo>
///   </mrow>
/// </math>
pub fn evpdf(x: f64, mu: f64, sigma: f64) -> f64 {
    // 1. Return NaN for out of range parameters or incoming NaNs
    if x.is_nan() || mu.is_nan() || sigma.is_nan() || sigma <= 0.0 {
        return f64::NAN;
    }

    let z = (x - mu) / sigma;

    // 2. Intercept extreme values to prevent Inf - Inf = NaN evaluations
    if z == f64::INFINITY || z > 700.0 {
        return 0.0;
    }
    
    if z == f64::NEG_INFINITY {
        return 0.0;
    }

    // 3. Standard evaluation path
    (z - z.exp()).exp() / sigma
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_extreme_value_pdf() {
        let result = evpdf(0.0, 0.0, 1.0);
        let expected = (-1.0f64).exp();
        assert!((result - expected).abs() < 1e-15);
    }

    #[test]
    fn test_invalid_scale() {
        assert!(evpdf(1.0, 0.0, 0.0).is_nan());
        assert!(evpdf(1.0, 0.0, -1.5).is_nan());
        assert!(evpdf(f64::NAN, 0.0, 1.0).is_nan());
    }

    #[test]
    fn test_extreme_tails() {
        assert_eq!(evpdf(1000.0, 0.0, 1.0), 0.0);
        assert_eq!(evpdf(f64::INFINITY, 0.0, 1.0), 0.0);
        assert_eq!(evpdf(-1000.0, 0.0, 1.0), 0.0);
        assert_eq!(evpdf(f64::NEG_INFINITY, 0.0, 1.0), 0.0);
    }

    #[test]
    fn test_shifted_distribution() {
        let result = evpdf(10.0, 10.0, 2.0);
        let expected = (-1.0f64).exp() / 2.0;
        assert!((result - expected).abs() < 1e-15);
    }
}