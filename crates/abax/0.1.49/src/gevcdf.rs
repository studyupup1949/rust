/// Generalized Extreme Value (GEV) cumulative distribution function (CDF) for scalar values.
///
/// Evaluates the CDF of the GEV distribution with shape parameter `k`, 
/// scale parameter `sigma`, and location parameter `mu` at the value `x`.
///
/// # Arguments
/// * `x` - The value at which to evaluate the CDF.
/// * `k` - Shape parameter. When k < 0, it is Type III. When k > 0, it is Type II (Fréchet). 
///         As k approaches 0, it approaches Type I.
/// * `sigma` - Scale parameter (sigma > 0).
/// * `mu` - Location parameter.
/// * `upper` - If true, computes the upper tail probability (survival function).
///             If false, computes the standard lower tail probability.
///
/// # Returns
/// The cumulative probability at `x`. Returns `f64::NAN` if parameters are invalid.
///
/// # Mathematical Formula
///
/// For variables where:
///
/// <div>
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="inline"><mi>&#x03C3;</mi><mo>&gt;</mo><mn>0</mn></math> and <math xmlns="http://www.w3.org/1998/Math/MathML" display="inline"><mn>1</mn><mo>+</mo><mi>k</mi><mrow><mo>(</mo><mfrac><mrow><mi>x</mi><mo>&#x2212;</mo><mi>&#x03BC;</mi></mrow><mi>&#x03C3;</mi></mfrac><mo>)</mo></mrow><mo>&gt;</mo><mn>0</mn></math>
/// </div>
///
/// Standard lower tail (`upper = false`):
///
/// <div>
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="block">
///   <mi>F</mi>
///   <mo stretchy="false">(</mo>
///   <mi>x</mi>
///   <mo>;</mo>
///   <mi>k</mi>
///   <mo>,</mo>
///   <mi>&#x03BC;</mi>
///   <mo>,</mo>
///   <mi>&#x03C3;</mi>
///   <mo stretchy="false">)</mo>
///   <mo>=</mo>
///   <mi>exp</mi>
///   <mrow>
///     <mo>(</mo>
///     <mo>&#x2212;</mo>
///     <msup>
///       <mrow>
///         <mo>[</mo>
///         <mn>1</mn>
///         <mo>+</mo>
///         <mi>k</mi>
///         <mrow>
///           <mo>(</mo>
///           <mfrac>
///             <mrow>
///               <mi>x</mi>
///               <mo>&#x2212;</mo>
///               <mi>&#x03BC;</mi>
///             </mrow>
///             <mi>&#x03C3;</mi>
///           </mfrac>
///           <mo>)</mo>
///         </mrow>
///         <mo>]</mo>
///       </mrow>
///       <mrow>
///         <mo>&#x2212;</mo>
///         <mfrac>
///           <mn>1</mn>
///           <mi>k</mi>
///         </mfrac>
///       </mrow>
///     </msup>
///     <mo>)</mo>
///   </mrow>
/// </math>
/// </div>
pub fn gevcdf(x: f64, k: f64, sigma: f64, mu: f64, upper: bool) -> f64 {
    // 1. Return NaN for out of range parameters or incoming NaNs
    if x.is_nan() || k.is_nan() || sigma.is_nan() || mu.is_nan() || sigma <= 0.0 {
        return f64::NAN;
    }

    let z = (x - mu) / sigma;

    // 2. Handle the Type I Extreme Value limit case (k approaches 0)
    if k.abs() < f64::EPSILON {
        return if upper {
            // Upper tail: 1 - exp(-exp(-z))
            -(-(-z).exp()).exp_m1()
        } else {
            // Standard lower tail: exp(-exp(-z))
            (-(-z).exp()).exp()
        };
    }

    // 3. Evaluate the core expression argument for cases where k != 0
    let t = z * k;

    // Check if the coordinate lies outside the distribution support (1 + k*z <= 0)
    if t <= -1.0 {
        return if upper {
            if k >= 0.0 { 1.0 } else { 0.0 }
        } else {
            if k < 0.0 { 1.0 } else { 0.0 }
        };
    }

    // 4. Compute standard evaluation pathways
    // Using ln_1p(t) yields accurate computations for ln(1 + t) near zero.
    let log_argument = -(1.0 / k) * t.ln_1p();
    let inner_exponent = log_argument.exp();

    if upper {
        // Upper tail probability: 1 - exp(-exp(-(1/k)*ln(1+k*z)))
        (-inner_exponent).exp_m1().abs()
    } else {
        // Standard lower tail probability: exp(-exp(-(1/k)*ln(1+k*z)))
        (-inner_exponent).exp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gev_type1_limit() {
        // For k=0, mu=0, sigma=1 at x=0, F(0) = exp(-exp(0)) = exp(-1)
        let res_lower = gevcdf(0.0, 0.0, 1.0, 0.0, false);
        let expected_lower = (-1.0f64).exp();
        assert!((res_lower - expected_lower).abs() < 1e-15);

        let res_upper = gevcdf(0.0, 0.0, 1.0, 0.0, true);
        let expected_upper = 1.0 - (-1.0f64).exp();
        assert!((res_upper - expected_upper).abs() < 1e-15);
    }

    #[test]
    fn test_gev_type2_frechet() {
        // Positive k (Type II distribution support boundary check: 1 + k*z > 0)
        let k = 0.5;
        let sigma = 1.5;
        let mu = 2.0;
        
        // Let x value violate the lower boundary condition for k > 0
        // 1 + 0.5 * (x - 2.0) / 1.5 <= 0  => x <= -1.0
        assert_eq!(gevcdf(-2.0, k, sigma, mu, false), 0.0);
        assert_eq!(gevcdf(-2.0, k, sigma, mu, true), 1.0);

        // Valid support query check
        let res = gevcdf(5.0, k, sigma, mu, false);
        assert!(res > 0.0 && res < 1.0);
    }

    #[test]
    fn test_gev_type3_weibull_mirror() {
        // Negative k (Type III distribution support boundary check)
        let k = -0.5;
        let sigma = 1.0;
        let mu = 3.0;

        // 1 + (-0.5) * (x - 3.0) / 1.0 <= 0 => 1 <= 0.5 * (x - 3) => x >= 5.0
        assert_eq!(gevcdf(6.0, k, sigma, mu, false), 1.0);
        assert_eq!(gevcdf(6.0, k, sigma, mu, true), 0.0);
    }

    #[test]
    fn test_invalid_parameters() {
        assert!(gevcdf(1.0, 0.1, 0.0, 0.0, false).is_nan());
        assert!(gevcdf(1.0, 0.1, -1.0, 0.0, false).is_nan());
        assert!(gevcdf(f64::NAN, 0.1, 1.0, 0.0, false).is_nan());
    }
}