/// Weibull cumulative distribution function (CDF) for scalar values.
///
/// Evaluates the CDF of the Weibull distribution with scale parameter `a` 
/// and shape parameter `b` at the value `x`.
///
/// # Arguments
/// * `x` - The value at which to evaluate the CDF.
/// * `a` - Scale parameter (a > 0).
/// * `b` - Shape parameter (b > 0).
/// * `upper` - If true, computes the upper tail probability (survival function).
///             If false, computes the standard lower tail probability.
///
/// # Returns
/// The cumulative probability at `x`. Returns `f64::NAN` if parameters are invalid.
///
/// # Mathematical Formula
/// For <math xmlns="http://www.w3.org/1998/Math/MathML" display="inline"><mi>x</mi><mo>&#x2265;</mo><mn>0</mn></math>, 
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="inline"><mi>a</mi><mo>&gt;</mo><mn>0</mn></math>, 
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="inline"><mi>b</mi><mo>&gt;</mo><mn>0</mn></math>:
///
/// Standard lower tail (`upper = false`):
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="block">
///   <mi>F</mi>
///   <mo stretchy="false">(</mo>
///   <mi>x</mi>
///   <mo>;</mo>
///   <mi>a</mi>
///   <mo>,</mo>
///   <mi>b</mi>
///   <mo stretchy="false">)</mo>
///   <mo>=</mo>
///   <mn>1</mn>
///   <mo>&#x2212;</mo>
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
///
/// Upper tail (`upper = true`):
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="block">
///   <mi>S</mi>
///   <mo stretchy="false">(</mo>
///   <mi>x</mi>
///   <mo>;</mo>
///   <mi>a</mi>
///   <mo>,</mo>
///   <mi>b</mi>
///   <mo stretchy="false">)</mo>
///   <mo>=</mo>
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
pub fn wblcdf(x: f64, a: f64, b: f64, upper: bool) -> f64 {
    // 1. Return NaN for out of range parameters or incoming NaNs
    if x.is_nan() || a.is_nan() || b.is_nan() || a <= 0.0 || b <= 0.0 {
        return f64::NAN;
    }

    // 2. Handle negative x values safely.
    // The probability mass below 0 is exactly 0.0. 
    // Therefore, lower tail is 0.0 and upper tail is 1.0.
    if x < 0.0 {
        return if upper { 1.0 } else { 0.0 };
    }

    let z = (x / a).powf(b);

    if upper {
        // Upper tail probability: exp(-z)
        (-z).exp()
    } else {
        // Standard lower tail probability: 1 - exp(-z)
        // expm1(-z) computes exp(-z) - 1 accurately for small values of z.
        // Therefore, -(exp(-z) - 1) equals 1 - exp(-z).
        -(-z).exp_m1()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_weibull_cdf() {
        // For a=1, b=1 (Exponential distribution), F(1) = 1 - exp(-1)
        let res_lower = wblcdf(1.0, 1.0, 1.0, false);
        let expected_lower = 1.0 - (-1.0f64).exp();
        assert!((res_lower - expected_lower).abs() < 1e-15);

        // Survival curve version: S(1) = exp(-1)
        let res_upper = wblcdf(1.0, 1.0, 1.0, true);
        let expected_upper = (-1.0f64).exp();
        assert!((res_upper - expected_upper).abs() < 1e-15);
        
        // Sum of complements must be exactly 1.0
        assert_eq!(res_lower + res_upper, 1.0);
    }

    #[test]
    fn test_negative_x() {
        // Left boundary edge case checks
        assert_eq!(wblcdf(-2.5, 1.0, 2.0, false), 0.0);
        assert_eq!(wblcdf(-2.5, 1.0, 2.0, true), 1.0);
        assert_eq!(wblcdf(-0.0, 1.0, 1.0, false), 0.0);
    }

    #[test]
    fn test_zero_x() {
        // At exactly x = 0, lower tail should be 0, upper tail should be 1
        assert_eq!(wblcdf(0.0, 5.0, 1.5, false), 0.0);
        assert_eq!(wblcdf(0.0, 5.0, 1.5, true), 1.0);
    }

    #[test]
    fn test_invalid_parameters() {
        assert!(wblcdf(1.0, 0.0, 1.0, false).is_nan());
        assert!(wblcdf(1.0, 1.0, -1.0, false).is_nan());
        assert!(wblcdf(f64::NAN, 1.0, 1.0, false).is_nan());
    }

    #[test]
    fn test_extreme_right_tail() {
        // Far right tail limit evaluation
        assert_eq!(wblcdf(100.0, 1.0, 2.0, false), 1.0);
        assert_eq!(wblcdf(100.0, 1.0, 2.0, true), 0.0);
    }
}