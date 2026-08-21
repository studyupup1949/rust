/// Extreme value inverse cumulative distribution function (quantile function) for scalar values.
///
/// Evaluates the inverse CDF of the Type 1 extreme value distribution (suitable for minima)
/// with location parameter `mu` and scale parameter `sigma` at the probability value `p`.
///
/// # Arguments
/// * `p` - The probability value at which to evaluate the inverse CDF (0 &le; p &le; 1).
/// * `mu` - Location parameter.
/// * `sigma` - Scale parameter (sigma > 0).
///
/// # Returns
/// The quantile value `x` at the probability `p`. Returns `f64::NAN` if inputs or parameters are invalid.
///
/// # Mathematical Formula
/// For <math xmlns="http://www.w3.org/1998/Math/MathML" display="inline"><mn>0</mn><mo>&#x2264;</mo><mi>p</mi><mo>&#x2264;</mo><mn>1</mn></math>, 
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="inline"><mi>&#x03C3;</mi><mo>&gt;</mo><mn>0</mn></math>:
///
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="block">
///   <mi>x</mi>
///   <mo>=</mo>
///   <mi>&#x03BC;</mi>
///   <mo>+</mo>
///   <mi>&#x03C3;</mi>
///   <mo>&#x22C5;</mo>
///   <mi>ln</mi>
///   <mrow>
///     <mo>(</mo>
///     <mo>&#x2212;</mo>
///     <mi>ln</mi>
///     <mo stretchy="false">(</mo>
///     <mn>1</mn>
///     <mo>&#x2212;</mo>
///     <mi>p</mi>
///     <mo stretchy="false">)</mo>
///     <mo>)</mo>
///   </mrow>
/// </math>
pub fn evinv(p: f64, mu: f64, sigma: f64) -> f64 {
    // 1. Return NaN for incoming NaNs or out of range parameters
    if p.is_nan() || mu.is_nan() || sigma.is_nan() || sigma <= 0.0 {
        return f64::NAN;
    }

    // 2. Out of bounds checking for probability domain
    if p < 0.0 || p > 1.0 {
        return f64::NAN;
    }

    // 3. Exact evaluation boundary cases
    if p == 1.0 {
        return f64::INFINITY;
    }
    // Handle lower bound domain underflow limits matching machine epsilon constraints
    if p < f64::EPSILON {
        return f64::NEG_INFINITY;
    }

    // 4. Transform probability to the intermediate quantile variable 'q'
    // ln_1p(-p) evaluates ln(1 + (-p)) = ln(1 - p) precisely for values near 0.
    let q = (-(-p).ln_1p()).ln();

    // 5. Calculate final scale translation
    sigma * q + mu
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_extreme_value_inverse() {
        // For mu=0, sigma=1, evaluating p = 1 - exp(-1) should return exactly 0.0
        let p = 1.0 - (-1.0f64).exp();
        let res = evinv(p, 0.0, 1.0);
        assert!(res.abs() < 1e-15);
    }

    #[test]
    fn test_boundary_probabilities() {
        assert_eq!(evinv(1.0, 0.0, 1.0), f64::INFINITY);
        assert_eq!(evinv(0.0, 0.0, 1.0), f64::NEG_INFINITY);
    }

    #[test]
    fn test_invalid_probabilities() {
        assert!(evinv(-0.01, 0.0, 1.0).is_nan());
        assert!(evinv(1.01, 0.0, 1.0).is_nan());
        assert!(evinv(f64::NAN, 0.0, 1.0).is_nan());
    }

    #[test]
    fn test_invalid_scale() {
        assert!(evinv(0.5, 0.0, 0.0).is_nan());
        assert!(evinv(0.5, 0.0, -2.5).is_nan());
    }

    #[test]
    fn test_round_trip() {
        // Round trip verification against evcdf lower tail structure
        let p_initial = 0.65;
        let mu = -3.0;
        let sigma = 4.5;
        
        let x = evinv(p_initial, mu, sigma);
        
        // Recover probability manually using evcdf logic branch
        let z = (x - mu) / sigma;
        let p_recovered = -(-z.exp()).exp_m1();

        assert!((p_initial - p_recovered).abs() < 1e-15);
    }
}