/// Weibull inverse cumulative distribution function (quantile function) for scalar values.
///
/// Evaluates the inverse CDF of the Weibull distribution with scale parameter `a` 
/// and shape parameter `b` at the probability value `p`.
///
/// # Arguments
/// * `p` - The probability value at which to evaluate the inverse CDF (0 &le; p &le; 1).
/// * `a` - Scale parameter (a > 0).
/// * `b` - Shape parameter (b > 0).
///
/// # Returns
/// The quantile value `x` at the probability `p`. Returns `f64::NAN` if inputs or parameters are invalid.
///
/// # Mathematical Formula
/// For <math xmlns="http://www.w3.org/1998/Math/MathML" display="inline"><mn>0</mn><mo>&#x2264;</mo><mi>p</mi><mo>&#x2264;</mo><mn>1</mn></math>, 
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="inline"><mi>a</mi><mo>&gt;</mo><mn>0</mn></math>, 
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="inline"><mi>b</mi><mo>&gt;</mo><mn>0</mn></math>:
///
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="block">
///   <mi>x</mi>
///   <mo>=</mo>
///   <msup>
///     <mrow>
///       <mi>a</mi>
///       <mo>&#x22C5;</mo>
///       <mo>[</mo>
///       <mo>&#x2212;</mo>
///       <mi>ln</mi>
///       <mo stretchy="false">(</mo>
///       <mn>1</mn>
///       <mo>&#x2212;</mo>
///       <mi>p</mi>
///       <mo stretchy="false">)</mo>
///       <mo>]</mo>
///     </mrow>
///     <mrow>
///       <mfrac>
///         <mn>1</mn>
///         <mi>b</mi>
///       </mfrac>
///     </mrow>
///   </msup>
/// </math>
pub fn wblinv(p: f64, a: f64, b: f64) -> f64 {
    // 1. Return NaN for incoming NaNs or out of range parameters
    if p.is_nan() || a.is_nan() || b.is_nan() || a <= 0.0 || b <= 0.0 {
        return f64::NAN;
    }

    // 2. Out of bounds checking for probability domain
    if p < 0.0 || p > 1.0 {
        return f64::NAN;
    }

    // 3. Exact evaluation boundary cases
    if p == 0.0 {
        return 0.0;
    }
    if p == 1.0 {
        return f64::INFINITY;
    }

    // 4. Transform probability to the intermediate quantile variable 'q'
    // log1p(-p) evaluates ln(1 + (-p)) = ln(1 - p) precisely for values near 0.
    let q = -(-p).log1p();

    // 5. Calculate final scale translation
    a * q.powf(1.0 / b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_weibull_inverse() {
        // For a=1, b=1 (Exponential distribution), inverse CDF at p=0.5 is -ln(0.5)
        let res = wblinv(0.5, 1.0, 1.0);
        let expected = -0.5f64.ln();
        assert!((res - expected).abs() < 1e-15);
    }

    #[test]
    fn test_boundary_probabilities() {
        assert_eq!(wblinv(0.0, 2.5, 1.5), 0.0);
        assert_eq!(wblinv(1.0, 2.5, 1.5), f64::INFINITY);
    }

    #[test]
    fn test_invalid_probabilities() {
        assert!(wblinv(-0.01, 1.0, 1.0).is_nan());
        assert!(wblinv(1.01, 1.0, 1.0).is_nan());
        assert!(wblinv(f64::NAN, 1.0, 1.0).is_nan());
    }

    #[test]
    fn test_invalid_parameters() {
        assert!(wblinv(0.5, 0.0, 1.0).is_nan());
        assert!(wblinv(0.5, 1.0, -2.0).is_nan());
        assert!(wblinv(0.5, -1.0, 1.0).is_nan());
    }

    #[test]
    fn test_round_trip() {
        // Round trip verification from a custom parameter group
        let p_initial = 0.75;
        let a = 1200.0;
        let b = 2.5;
        
        // Manual verification logic mapping (wblcdf lower tail structure)
        let x = wblinv(p_initial, a, b);
        let z = (x / a).powf(b);
        let p_recovered = -(-z).exp_m1();

        assert!((p_initial - p_recovered).abs() < 1e-15);
    }
}