/// Extreme value cumulative distribution function (CDF) for scalar values.
///
/// Evaluates the CDF of the Type 1 extreme value distribution (suitable for minima)
/// with location parameter `mu` and scale parameter `sigma` at the value `x`.
///
/// # Arguments
/// * `x` - The value at which to evaluate the CDF.
/// * `mu` - Location parameter.
/// * `sigma` - Scale parameter (sigma > 0).
/// * `upper` - If true, computes the upper tail probability (survival function).
///             If false, computes the standard lower tail probability.
///
/// # Returns
/// The cumulative probability at `x`. Returns `f64::NAN` if parameters are invalid.
///
/// # Mathematical Formula
/// For <math xmlns="http://www.w3.org/1998/Math/MathML" display="inline"><mi>&#x03C3;</mi><mo>&gt;</mo><mn>0</mn></math>:
///
/// Standard lower tail (`upper = false`):
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="block">
///   <mi>F</mi>
///   <mo stretchy="false">(</mo>
///   <mi>x</mi>
///   <mo>;</mo>
///   <mi>&#x03BC;</mi>
///   <mo>,</mo>
///   <mi>&#x03C3;</mi>
///   <mo stretchy="false">)</mo>
///   <mo>=</mo>
///   <mn>1</mn>
///   <mo>&#x2212;</mo>
///   <mi>exp</mi>
///   <mrow>
///     <mo>(</mo>
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
///
/// Upper tail (`upper = true`):
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="block">
///   <mi>S</mi>
///   <mo stretchy="false">(</mo>
///   <mi>x</mi>
///   <mo>;</mo>
///   <mi>&#x03BC;</mi>
///   <mo>,</mo>
///   <mi>&#x03C3;</mi>
///   <mo stretchy="false">)</mo>
///   <mo>=</mo>
///   <mi>exp</mi>
///   <mrow>
///     <mo>(</mo>
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
pub fn evcdf(x: f64, mu: f64, sigma: f64, upper: bool) -> f64 {
    // 1. Return NaN for out of range parameters or incoming NaNs
    if x.is_nan() || mu.is_nan() || sigma.is_nan() || sigma <= 0.0 {
        return f64::NAN;
    }

    let z = (x - mu) / sigma;
    let ez = z.exp();

    if upper {
        // Upper tail probability: exp(-exp(z))
        (-ez).exp()
    } else {
        // Standard lower tail probability: 1 - exp(-exp(z))
        // exp_m1(-ez) computes exp(-ez) - 1 accurately for small values.
        // Therefore, -(exp(-ez) - 1) equals 1 - exp(-ez).
        -(-ez).exp_m1()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_extreme_value_cdf() {
        // For mu=0, sigma=1, at x=0: z = 0, exp(0) = 1
        // F(0) = 1 - exp(-1)
        let res_lower = evcdf(0.0, 0.0, 1.0, false);
        let expected_lower = 1.0 - (-1.0f64).exp();
        assert!((res_lower - expected_lower).abs() < 1e-15);

        // Upper tail version: S(0) = exp(-1)
        let res_upper = evcdf(0.0, 0.0, 1.0, true);
        let expected_upper = (-1.0f64).exp();
        assert!((res_upper - expected_upper).abs() < 1e-15);

        // Complementary check
        assert_eq!(res_lower + res_upper, 1.0);
    }

    #[test]
    fn test_invalid_scale() {
        assert!(evcdf(1.0, 0.0, 0.0, false).is_nan());
        assert!(evcdf(1.0, 0.0, -0.5, false).is_nan());
        assert!(evcdf(f64::NAN, 0.0, 1.0, false).is_nan());
    }

    #[test]
    fn test_extreme_tails() {
        // As z -> Inf, F(x) -> 1 and S(x) -> 0
        assert_eq!(evcdf(1000.0, 0.0, 1.0, false), 1.0);
        assert_eq!(evcdf(1000.0, 0.0, 1.0, true), 0.0);

        // As z -> -Inf, F(x) -> 0 and S(x) -> 1
        assert_eq!(evcdf(-1000.0, 0.0, 1.0, false), 0.0);
        assert_eq!(evcdf(-1000.0, 0.0, 1.0, true), 1.0);
    }

    #[test]
    fn test_shifted_distribution() {
        let res_lower = evcdf(5.0, 5.0, 2.0, false);
        let expected_lower = 1.0 - (-1.0f64).exp();
        assert!((res_lower - expected_lower).abs() < 1e-15);
    }
}