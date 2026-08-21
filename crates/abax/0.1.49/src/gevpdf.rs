/// Calculates the Generalized Extreme Value (GEV) probability density function (PDF) for a single scalar value.
///
/// ### Mathematical Definition
/// 
/// The probability density function for the generalized extreme value distribution with location parameter <math><mi>&mu;</mi></math>,
/// scale parameter <math><mi>&sigma;</mi><mo>&gt;</mo><mn>0</mn></math>, and shape parameter <math><mi>k</mi><mo>&ne;</mo><mn>0</mn></math> is
///
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="block">
///   <mi>f</mi>
///   <mo stretchy="false">(</mo>
///   <mi>x</mi>
///   <mo>;</mo>
///   <mi>k</mi>
///   <mo>,</mo>
///   <mi>&mu;</mi>
///   <mo>,</mo>
///   <mi>&sigma;</mi>
///   <mo stretchy="false">)</mo>
///   <mo>=</mo>
///   <mfrac>
///     <mn>1</mn>
///     <mi>&sigma;</mi>
///   </mfrac>
///   <mo>&middot;</mo>
///   <mi>exp</mi>
///   <mrow>
///     <mo>(</mo>
///     <mo>&minus;</mo>
///     <msup>
///       <mrow>
///         <mo>(</mo>
///         <mn>1</mn>
///         <mo>+</mo>
///         <mi>k</mi>
///         <mrow>
///           <mo>(</mo>
///           <mfrac>
///             <mrow>
///               <mi>x</mi>
///               <mo>&minus;</mo>
///               <mi>&mu;</mi>
///             </mrow>
///             <mi>&sigma;</mi>
///           </mfrac>
///           <mo>)</mo>
///         </mrow>
///         <mo>)</mo>
///       </mrow>
///       <mrow>
///         <mo>&minus;</mo>
///         <mn>1</mn>
///         <mo>/</mo>
///         <mi>k</mi>
///       </mrow>
///     </msup>
///     <mo>)</mo>
///   </mrow>
///   <msup>
///     <mrow>
///       <mo>(</mo>
///       <mn>1</mn>
///       <mo>+</mo>
///       <mi>k</mi>
///       <mrow>
///         <mo>(</mo>
///         <mfrac>
///           <mrow>
///             <mi>x</mi>
///             <mo>&minus;</mo>
///             <mi>&mu;</mi>
///           </mrow>
///           <mi>&sigma;</mi>
///         </mfrac>
///         <mo>)</mo>
///       </mrow>
///       <mo>)</mo>
///     </mrow>
///     <mrow>
///       <mo>&minus;</mo>
///       <mn>1</mn>
///       <mo>&minus;</mo>
///       <mn>1</mn>
///       <mo>/</mo>
///       <mi>k</mi>
///     </mrow>
///   </msup>
/// </math>
///
/// for
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="block">
///   <mn>1</mn>
///   <mo>+</mo>
///   <mi>k</mi>
///   <mfrac>
///     <mrow>
///       <mi>x</mi>
///       <mo>&minus;</mo>
///       <mi>&mu;</mi>
///     </mrow>
///     <mi>&sigma;</mi>
///   </mfrac>
///   <mo>&gt;</mo>
///   <mn>0</mn>
/// </math>
/// <math><mi>k</mi><mo>&gt;</mo><mn>0</mn></math> corresponds to the Type II case, while <math><mi>k</mi><mo>&lt;</mo><mn>0</mn></math> corresponds to the Type III case. For <math><mi>k</mi><mo>=</mo><mn>0</mn></math>,
/// corresponding to the Type I case, the density is
///
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="block">
///   <mi>f</mi>
///   <mo stretchy="false">(</mo>
///   <mi>x</mi>
///   <mo>;</mo>
///   <mn>0</mn>
///   <mo>,</mo>
///   <mi>&mu;</mi>
///   <mo>,</mo>
///   <mi>&sigma;</mi>
///   <mo stretchy="false">)</mo>
///   <mo>=</mo>
///   <mfrac>
///     <mn>1</mn>
///     <mi>&sigma;</mi>
///   </mfrac>
///   <mi>exp</mi>
///   <mrow>
///     <mo>(</mo>
///     <mo>&minus;</mo>
///     <mi>exp</mi>
///     <mrow>
///       <mo>(</mo>
///       <mo>&minus;</mo>
///       <mfrac>
///         <mrow>
///           <mi>x</mi>
///           <mo>&minus;</mo>
///           <mi>&mu;</mi>
///         </mrow>
///         <mi>&sigma;</mi>
///       </mfrac>
///       <mo>)</mo>
///     </mrow>
///     <mo>&minus;</mo>
///     <mfrac>
///       <mrow>
///         <mi>x</mi>
///         <mo>&minus;</mo>
///         <mi>&mu;</mi>
///         </mrow>
///       <mi>&sigma;</mi>
///     </mfrac>
///     <mo>)</mo>
///   </mrow>
/// </math>
///
/// ### Arguments
/// * `x` - The evaluation point.
/// * `k` - Shape parameter.
/// * `sigma` - Scale parameter (returns `f64::NAN` if <math><mi>&sigma;</mi><mo>&le;</mo><mn>0</mn></math>).
/// * `mu` - Location parameter.
pub fn gevpdf(x: f64, k: f64, sigma: f64, mu: f64) -> f64 {
    if sigma <= 0.0 {
        return f64::NAN;
    }

    let mut z = (x - mu) / sigma;
    if z == f64::NEG_INFINITY {
        z = f64::INFINITY;
    }

    if k.abs() < f64::EPSILON {
        // Gumbel distribution limit case
        (-(-z).exp() - z).exp() / sigma
    } else {
        let t = z * k;

        // Domain support boundary check: 1 + k*(x-mu)/sigma > 0
        if t <= -1.0 {
            0.0
        } else {
            let log1p_t = t.ln_1p();
            let term1 = (-(1.0 / k) * log1p_t).exp();
            let term2 = (1.0 + 1.0 / k) * log1p_t;
            
            (-term1 - term2).exp() / sigma
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_near {
        ($left:expr, $right:expr, $tol:expr) => {
            assert!(
                ($left - $right).abs() < $tol,
                "Assertion failed: {} is not near {} within tolerance {}",
                $left, $right, $tol
            );
        };
    }

    #[test]
    fn test_gevpdf_basic() {
        // Standard shape case
        let res = gevpdf(0.5, 0.1, 1.0, 0.0);
        assert_near!(res, 3.164452432721054e-01, 1e-14);
    }

    #[test]
    fn test_gevpdf_invalid_sigma() {
        assert!(gevpdf(1.0, 0.1, -1.0, 0.0).is_nan());
        assert!(gevpdf(1.0, 0.1, 0.0, 0.0).is_nan());
    }

    #[test]
    fn test_gevpdf_k_zero() {
        let res = gevpdf(0.0, 0.0, 1.0, 0.0);
        assert_near!(res, (-1.0f64).exp(), 1e-9);
    }

    #[test]
    fn test_gevpdf_outside_support() {
        // 1 + 0.5 * (-3.0) = -0.5 <= -1.0 support limit
        let res = gevpdf(-3.0, 0.5, 1.0, 0.0);
        assert_eq!(res, 0.0);
    }
}