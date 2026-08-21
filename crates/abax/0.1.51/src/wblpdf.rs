/// Weibull probability density function (PDF) for scalar values.
///
/// Evaluates the PDF of the Weibull distribution with scale parameter `a` 
/// and shape parameter `b` at the value `x`.
///
/// # Arguments
/// * `x` - The value at which to evaluate the PDF.
/// * `a` - Scale parameter (a > 0).
/// * `b` - Shape parameter (b > 0).
///
/// # Returns
/// The probability density at `x`. Returns `f64::NAN` if parameters are invalid.
///
/// # Mathematical Formula
/// For <math xmlns="http://www.w3.org/1998/Math/MathML" display="inline"><mi>x</mi><mo>&#x2265;</mo><mn>0</mn></math>, 
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="inline"><mi>a</mi><mo>&gt;</mo><mn>0</mn></math>, 
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="inline"><mi>b</mi><mo>&gt;</mo><mn>0</mn></math>:
///
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="block">
///   <mi>f</mi>
///   <mo stretchy="false">(</mo>
///   <mi>x</mi>
///   <mo>;</mo>
///   <mi>a</mi>
///   <mo>,</mo>
///   <mi>b</mi>
///   <mo stretchy="false">)</mo>
///   <mo>=</mo>
///   <mfrac>
///     <mi>b</mi>
///     <mi>a</mi>
///   </mfrac>
///   <msup>
///     <mrow>
///       <mo>(</mo>
///       <mfrac>
///         <mi>x</mi>
///         <mi>a</mi>
///       </mfrac>
///       <mo>)</mo>
///     </mrow>
///     <mrow>
///       <mi>b</mi>
///       <mo>&#x2212;</mo>
///       <mn>1</mn>
///     </mrow>
///   </msup>
///   <msup>
///     <mi>e</mi>
///     <mrow>
///       <mo>&#x2212;</mo>
///       <msup>
///         <mrow>
///           <mo>(</mo>
///           <mfrac>
///             <mi>x</mi>
///             <mi>a</mi>
///           </mfrac>
///           <mo>)</mo>
///         </mrow>
///         <mi>b</mi>
///       </msup>
///     </mrow>
///   </msup>
/// </math>
pub fn wblpdf(x: f64, a: f64, b: f64) -> f64 {
    // 1. Return NaN for out of range parameters or incoming NaNs
    if x.is_nan() || a.is_nan() || b.is_nan() || a <= 0.0 || b <= 0.0 {
        return f64::NAN;
    }

    // 2. Handle negative x values safely.
    if x < 0.0 {
        return 0.0;
    }

    let z = x / a;

    // 3. Handle extreme right tail to prevent 0 * Inf situations.
    let zb = z.powf(b);
    let w = (-zb).exp();

    if w == 0.0 {
        return 0.0;
    }

    // 4. Standard evaluation path
    z.powf(b - 1.0) * w * b / a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_weibull() {
        // Evaluating standard parameters where a=1, b=1 (equivalent to standard exponential)
        let result = wblpdf(1.0, 1.0, 1.0);
        let expected = (-1.0f64).exp();
        assert!((result - expected).abs() < 1e-15);
    }

    #[test]
    fn test_negative_x() {
        assert_eq!(wblpdf(-5.5, 2.0, 1.5), 0.0);
        assert_eq!(wblpdf(-0.0, 1.0, 1.0), 1.0);
    }

    #[test]
    fn test_zero_x() {
        // For b > 1, PDF at 0 is 0
        assert_eq!(wblpdf(0.0, 2.0, 2.0), 0.0);
        
        // For b == 1, PDF at 0 is b/a
        assert_eq!(wblpdf(0.0, 2.0, 1.0), 0.5);
        
        // For b < 1, PDF at 0 approaches Infinity
        assert_eq!(wblpdf(0.0, 1.0, 0.5), f64::INFINITY);
    }

    #[test]
    fn test_invalid_parameters() {
        assert!(wblpdf(1.0, 0.0, 1.0).is_nan());
        assert!(wblpdf(1.0, 1.0, -0.5).is_nan());
        assert!(wblpdf(f64::NAN, 1.0, 1.0).is_nan());
    }

    #[test]
    fn test_extreme_right_tail() {
        assert_eq!(wblpdf(1000.0, 1.0, 2.0), 0.0);
    }
}