use crate::consts::SQRT_2PI;

/// Calculates the Gamma function Γ(x) using the Lanczos approximation.
///
/// This implementation utilizes the Lanczos approximation
/// with a shift parameter <math xmlns="http://www.w3.org/1998/Math/MathML"><mi>g</mi><mo>=</mo><mn>7.0</mn></math>
/// and <math xmlns="http://www.w3.org/1998/Math/MathML"><mi>n</mi><mo>=</mo><mn>9</mn></math> coefficients.
///
/// # Mathematical Definition
/// The Gamma function is an extension of the factorial function to complex numbers.
/// For a real number <math xmlns="http://www.w3.org/1998/Math/MathML"><mi>x</mi><mo>></mo><mn>0</mn></math>,
/// it is defined by the integral:
///
/// <math display="block" xmlns="http://www.w3.org/1998/Math/MathML">
///   <mi>Γ</mi>
///   <mo stretchy="false">(</mo>
///   <mi>x</mi>
///   <mo stretchy="false">)</mo>
///   <mo>=</mo>
///   <msubsup>
///     <mo>∫</mo>
///     <mn>0</mn>
///     <mi>∞</mi>
///   </msubsup>
///   <msup>
///     <mi>t</mi>
///     <mrow>
///       <mi>x</mi>
///       <mo>−</mo>
///       <mn>1</mn>
///     </mrow>
///   </msup>
///   <msup>
///     <mi>e</mi>
///     <mrow>
///       <mo>−</mo>
///       <mi>t</mi>
///     </mrow>
///   </msup>
///   <mi>d</mi>
///   <mi>t</mi>
/// </math>
///
/// # Implementation Details
/// - **Reflection Formula**: For <math xmlns="http://www.w3.org/1998/Math/MathML"><mi>x</mi><mo>&lt;</mo><mn>0.5</mn></math>,
///   the implementation maintains stability using the reflection formula:
///   <math display="block" xmlns="http://www.w3.org/1998/Math/MathML">
///     <mi>Γ</mi>
///     <mo stretchy="false">(</mo>
///     <mi>x</mi>
///     <mo stretchy="false">)</mo>
///     <mo>=</mo>
///     <mfrac>
///       <mi>π</mi>
///       <mrow>
///         <mi>sin</mi>
///         <mo>⁡</mo>
///         <mo stretchy="false">(</mo>
///         <mi>π</mi>
///         <mi>x</mi>
///         <mo stretchy="false">)</mo>
///         <mi>Γ</mi>
///         <mo stretchy="false">(</mo>
///         <mn>1</mn>
///         <mo>−</mo>
///         <mi>x</mi>
///         <mo stretchy="false">)</mo>
///       </mrow>
///     </mfrac>
///   </math>
/// - **Exact Integers**: Returns <math xmlns="http://www.w3.org/1998/Math/MathML"><mo>(</mo><mi>x</mi><mo>-</mo><mn>1</mn><mo>)</mo><mo>!</mo></math>
///   for integers up to 23 using a pre-calculated lookup table for maximum precision.
/// - **Numerical Stability**: Handles poles at non-positive integers and special cases like `NaN` and `Infinity`.
///
/// # Examples
/// ```
/// use abax::gamma;
/// let result = gamma(5.0);
/// assert_eq!(result, 24.0); // 4!
/// ```
pub fn gamma(x: f64) -> f64 {
    // 1. Handle Special Cases & Poles
    if x.is_nan() { return f64::NAN; }
    if x.is_infinite() {
        return if x.is_sign_positive() { f64::INFINITY } else { f64::NAN };
    }

    // Poles at non-positive integers
    if x <= 0.0 && x == x.floor() {
        // Return NAN for negative integers; Signed Infinity for 0.0 is optional but common
        return if x == 0.0 { x.recip() } else { f64::NAN };
    }

    // 2. Reflection Formula for x < 0.5
    if x < 0.5 {
        return std::f64::consts::PI / ((std::f64::consts::PI * x).sin() * gamma(1.0 - x));
    }

    // 3. Exact Integers (Gamma(n) = (n-1)!)
    if x == x.floor() && x <= 23.0 {
        return (1..x as u64).map(|i| i as f64).product();
    }


    // 4. Lanczos Approximation (g=7, n=9)
    const G: f64 = 7.0;
    const P: [f64; 9] = [
        0.99999999999980993,
        676.5203681218851,
        -1259.1392167224028,
        771.32342877765313,
        -176.61502916214059,
        12.507343278686905,
        -0.13857109526572012,
        9.9843695780195716e-6,
        1.5056327351493116e-7,
    ];

    let z = x - 1.0;
    let mut a = P[0];
    for (i, &p_val) in P.iter().skip(1).enumerate() {
        a += p_val / (z + (i + 1) as f64);
    }

    let t = z + G + 0.5;

    SQRT_2PI * t.powf(z + 0.5) * (-t).exp() * a
}

/// Evaluates the Gamma function Γ(x) using the approximation from 
/// W. J. Cody (Argonne National Laboratory, 1989).
#[allow(dead_code)]
fn gamma_cody(x: f64) -> f64 {
    let mut x = x;    
    // 1. Handle special cases
    if x.is_nan() {
        return f64::NAN;
    }
    if x.is_infinite() {
        return if x.is_sign_positive() { f64::INFINITY } else { f64::NAN };
    }

    // Poles at non-positive integers
    if x <= 0.0 && x == x.trunc() {
        return if x == 0.0 { f64::INFINITY * x.signum() } else { f64::NAN };
    }

    // Coefficients for 1 <= x <= 2
    const P: [f64; 8] = [
        -1.71618513886549492533811e+0, 2.47656508055759199108314e+1,
        -3.79804256470945635097577e+2, 6.29331155312818442661052e+2,
        8.66966202790413211295064e+2, -3.14512729688483675254357e+4,
        -3.61444134186911729807069e+4, 6.64561438202405440627855e+4,
    ];
    const Q: [f64; 8] = [
        -3.08402300119738975254353e+1, 3.15350626979604161529144e+2,
        -1.01515636749021914166146e+3, -3.10777167157231109440444e+3,
        2.25381184209801510330112e+4, 4.75584627752788110767815e+3,
        -1.34659959864969306392456e+5, -1.15132259675553483497211e+5,
    ];
    
    // Coefficients for asymptotic series x >= 12
    const C: [f64; 7] = [
        -1.910444077728e-03, 8.4171387781295e-04,
        -5.952379913043012e-04, 7.93650793500350248e-04,
        -2.777777777777681622553e-03, 8.333333333333333331554247e-02,
        5.7083835261e-03,
    ];
    let spi = 0.9189385332046727417803297; // 0.5 * ln(2 * pi)

    let mut fact = 1.0;
    let mut is_negative = false;

    // 2. Catch negative x and map to positive using reflection formula
    if x < 0.0 {
        is_negative = true;
        let y = -x;
        let y1 = y.trunc();
        let res_frac = y - y1;
        
        // Reflection formula factor: -pi / (sin(pi * res) * (1 - 2*rem(y1, 2)))
        let rem_y1_2 = y1 % 2.0;
        fact = -std::f64::consts::PI / ((std::f64::consts::PI * res_frac).sin() * (1.0 - 2.0 * rem_y1_2));
        
        // Map x to positive range calculation
        x = y + 1.0; 
    }

    let mut res;

    // 3. Evaluate based on region
    if x >= 12.0 {
        // Asymptotic approximation for x >= 12
        let y = x;
        let ysq = y * y;
        let mut sum = C[6];
        for &coefficient in C.iter().take(6) {
            sum = sum / ysq + coefficient;
        }
        sum = sum / y - y + spi;
        sum += (y - 0.5) * y.ln();
        res = sum.exp();
    } else {
        // Argument reduction for x < 12
        let mut x1 = 1.0;
        let mut was_less_than_one = false;
        
        // Map x in [0, 1] to [1, 2]
        if x < 1.0 {
            x1 = x;
            x += 1.0;
            was_less_than_one = true;
        }

        // Map x in [1, 12] to [1, 2]
        let xn = x.trunc() - 1.0;
        x -= xn;

        // Evaluate rational approximation for 1 <= x <= 2
        let z = x - 1.0;
        let mut xnum = 0.0;
        let mut xden = 1.0;
        for i in 0..8 {
            xnum = (xnum + P[i]) * z;
            xden = xden * z + Q[i];
        }
        res = xnum / xden + 1.0;

        // Adjust result for case 0.0 < original x < 1.0
        if was_less_than_one {
            res /= x1;
        }

        // Adjust result for case 2.0 < original x < 12.0
        // Re-apply the integer offsets we subtracted earlier
        for _ in 0..(xn as i32) {
            res *= x;
            x += 1.0;
        }
    }

    // 4. Final adjustments for original negative values
    if is_negative {
        res = fact / res;
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-14;

    #[test]
    fn test_exact_integers() {
        // Gamma(n) = (n-1)!
        assert_eq!(gamma(1.0), 1.0);
        assert_eq!(gamma(2.0), 1.0);
        assert_eq!(gamma(3.0), 2.0);
        assert_eq!(gamma(4.0), 6.0);
        assert_eq!(gamma(5.0), 24.0);
        assert_eq!(gamma(10.0), 362880.0);
    }

    #[test]
    fn test_half_integers() {
        // Gamma(1/2) = sqrt(pi)
        let sqrt_pi = std::f64::consts::PI.sqrt();
        assert!((gamma(0.5) - sqrt_pi).abs() < EPSILON);

        // Gamma(3/2) = 1/2 * sqrt(pi)
        assert!((gamma(1.5) - 0.5 * sqrt_pi).abs() < EPSILON);

        // Gamma(5/2) = 3/4 * sqrt(pi)
        assert!((gamma(2.5) - 0.75 * sqrt_pi).abs() < EPSILON);
    }

    #[test]
    fn test_recurrence_relation() {
        // Gamma(x + 1) = x * Gamma(x)
        let x = std::f64::consts::PI;
        let lhs = gamma(x + 1.0);
        let rhs = x * gamma(x);
        assert!((lhs - rhs).abs() / lhs < EPSILON);
    }

    #[test]
    fn test_negative_values() {
        // Test a few known negative points using reflection
        // Gamma(-0.5) = -2 * sqrt(pi)
        let expected = -2.0 * std::f64::consts::PI.sqrt();
        assert!((gamma(-0.5) - expected).abs() < EPSILON);
    }

    #[test]
    fn test_special_cases() {
        // Poles (Returns Infinity or NaN based on your implementation)
        assert!(gamma(0.0).is_infinite());
        assert!(gamma(-1.0).is_nan());

        // Limits
        assert!(gamma(f64::NAN).is_nan());
        assert!(gamma(f64::INFINITY).is_infinite());
    }
}
