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
        return f64::INFINITY; 
    }

    // 2. Exact Integers (Gamma(n) = (n-1)!)
    if x > 0.0 && x == x.floor() && x <= 23.0 {
        let factorial: [f64; 23] = [
            1.0, 1.0, 2.0, 6.0, 24.0, 120.0, 720.0, 5040.0, 40320.0, 
            362880.0, 3628800.0, 39916800.0, 479001600.0, 6227020800.0, 
            87178291200.0, 1307674368000.0, 20922789888000.0, 355687428096000.0, 
            6402373705728000.0, 121645100408832000.0, 2432902008176640000.0, 
            51090942171709440000.0, 1124000727777607680000.0
        ];
        return factorial[(x - 1.0) as usize];
    }

    // 3. Reflection Formula for x < 0.5
    if x < 0.5 {
        return std::f64::consts::PI / ((std::f64::consts::PI * x).sin() * gamma(1.0 - x));
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
    for i in 1..9 {
        a += P[i] / (z + i as f64);
    }

    let t = z + G + 0.5;
    
    // Using SQRT(2*PI) constant directly for a tiny speed boost
    const SQRT_2PI: f64 = 2.5066282746310005;
    
    SQRT_2PI * t.powf(z + 0.5) * (-t).exp() * a
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
        let x = 3.14159;
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
        assert!(gamma(-1.0).is_infinite());
        
        // Limits
        assert!(gamma(f64::NAN).is_nan());
        assert!(gamma(f64::INFINITY).is_infinite());
    }
}