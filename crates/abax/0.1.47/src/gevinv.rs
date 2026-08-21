/// Generalized Extreme Value (GEV) inverse cumulative distribution function (quantile function) for scalar values.
///
/// Evaluates the inverse CDF of the GEV distribution with shape parameter `k`, 
/// scale parameter `sigma`, and location parameter `mu` at the probability value `p`.
///
/// # Arguments
/// * `p` - The probability value at which to evaluate the inverse CDF (0 &le; p &le; 1).
/// * `k` - Shape parameter. When k < 0, it is Type III. When k > 0, it is Type II (Fréchet).
/// * `sigma` - Scale parameter (sigma > 0).
/// * `mu` - Location parameter.
///
/// # Returns
/// The quantile value `x` at the probability `p`. Returns `f64::NAN` if inputs or parameters are invalid.
///
/// # Mathematical Formula
///
/// For variables where:
///
/// <div>
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="inline"><mn>0</mn><mo>&#x2212;</mo><mi>p</mi><mo>&#x2212;</mo><mn>1</mn></math> and <math xmlns="http://www.w3.org/1998/Math/MathML" display="inline"><mi>&#x03C3;</mi><mo>&gt;</mo><mn>0</mn></math>
/// </div>
///
/// Quantile calculation formula for <math xmlns="http://www.w3.org/1998/Math/MathML" display="inline"><mi>k</mi><mo>&#x2260;</mo><mn>0</mn></math>:
///
/// <div>
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="block">
///   <mi>x</mi>
///   <mo>=</mo>
///   <mi>&#x03BC;</mi>
///   <mo>+</mo>
///   <mfrac>
///     <mi>&#x03C3;</mi>
///     <mi>k</mi>
///   </mfrac>
///   <mo>&#x22C5;</mo>
///   <mrow>
///     <mo>[</mo>
///     <msup>
///       <mrow>
///         <mo>(</mo>
///         <mo>&#x2212;</mo>
///         <mi>ln</mi>
///         <mo stretchy="false">(</mo>
///         <mi>p</mi>
///         <mo stretchy="false">)</mo>
///         <mo>)</mo>
///       </mrow>
///       <mrow>
///         <mo>&#x2212;</mo>
///         <mi>k</mi>
///       </mrow>
///     </msup>
///     <mo>&#x2212;</mo>
///     <mn>1</mn>
///     <mo>]</mo>
///   </mrow>
/// </math>
/// </div>
pub fn gevinv(p: f64, k: f64, sigma: f64, mu: f64) -> f64 {
    // 1. Return NaN for incoming NaNs or out of range scale parameter
    if p.is_nan() || k.is_nan() || sigma.is_nan() || mu.is_nan() || sigma <= 0.0 {
        return f64::NAN;
    }

    // 2. Out of bounds checking for probability domain
    if p < 0.0 || p > 1.0 {
        return f64::NAN;
    }

    // 3. Exact evaluation boundary cases based on support parameters
    if p == 0.0 {
        return if k <= 0.0 {
            f64::NEG_INFINITY
        } else {
            mu - sigma / k
        };
    }

    if p == 1.0 {
        return if k < 0.0 {
            mu - sigma / k
        } else {
            f64::INFINITY
        };
    }

    // 4. Transform probability to the intermediate quantile parameter 'z'
    let log_p = -p.ln(); // equals -ln(p)
    let z = if k.abs() < f64::EPSILON {
        // Limit case as k approaches 0 (Type I Gumbel variant)
        -log_p.ln()
    } else {
        // General calculation pathway safely implementing exp_m1 
        // to compute: (exp(-k * ln(-ln(p))) - 1) / k
        let log_log_p = log_p.ln();
        (-k * log_log_p).exp_m1() / k
    };

    // 5. Final location translation scale
    mu + sigma * z
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gevcdf;

    #[test]
    fn test_gev_type1_inverse_limit() {
        // For k=0, mu=0, sigma=1, evaluating p = exp(-1) should return exactly 0.0
        let p = (-1.0f64).exp();
        let res = gevinv(p, 0.0, 1.0, 0.0);
        assert!(res.abs() < 1e-15);
    }

    #[test]
    fn test_gev_boundary_cases() {
        // k > 0 (Type II) lower bound support boundary: mu - sigma/k
        assert_eq!(gevinv(0.0, 0.5, 1.5, 2.0), -1.0);
        assert_eq!(gevinv(1.0, 0.5, 1.5, 2.0), f64::INFINITY);

        // k < 0 (Type III) upper bound support boundary: mu - sigma/k
        assert_eq!(gevinv(0.0, -0.5, 1.0, 3.0), f64::NEG_INFINITY);
        assert_eq!(gevinv(1.0, -0.5, 1.0, 3.0), 5.0);
    }

    #[test]
    fn test_invalid_inputs() {
        assert!(gevinv(-0.01, 0.1, 1.0, 0.0).is_nan());
        assert!(gevinv(1.01, 0.1, 1.0, 0.0).is_nan());
        assert!(gevinv(0.5, 0.1, -1.0, 0.0).is_nan());
    }

    #[test]
    fn test_round_trip() {
        // Round trip confirmation using custom parameter profiles
        let p_initial = 0.45;
        let k = 0.25;
        let sigma = 2.0;
        let mu = 10.0;

        // Find value via gevinv, then map backwards via manual gevcdf logic
        let x = gevinv(p_initial, k, sigma, mu);
        let p_recovered = gevcdf(x, k, sigma, mu, false);

        assert!((p_initial - p_recovered).abs() < 1e-15);
    }
}