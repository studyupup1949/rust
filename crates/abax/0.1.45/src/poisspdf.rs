use crate::{stirlerr::stirlerr, binodeviance::binodeviance, gammaln};

/// Poisson probability density function (PDF).
///
/// Returns the Poisson probability density function with parameter `lambda` at the value `x`.
///
/// # Mathematical Definition
/// <math display="block">
///   <mi>f</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>λ</mi><mo>)</mo>
///   <mo>=</mo>
///   <mfrac>
///     <mrow>
///       <msup><mi>λ</mi><mi>x</mi></msup>
///       <msup><mi>e</mi><mrow><mo>-</mo><mi>λ</mi></mrow></msup>
///     </mrow>
///     <mrow>
///       <mi>x</mi><mo>!</mo>
///     </mrow>
///   </mfrac>
/// </math>
/// for <math><mi>x</mi><mo>∈</mo><mo>{</mo><mn>0</mn><mo>,</mo><mn>1</mn><mo>,</mo><mn>2</mn><mo>,</mo><mo>...</mo><mo>}</mo></math>.
///
/// # Domain
/// - `x`: Non-negative integer.
/// - `lambda`: Non-negative real number.
/// - Returns `0.0` if `x` is not a non-negative integer.
/// - Returns `NaN` if `lambda < 0` or any input is `NaN`.
///
/// # Examples
/// ```
/// use abax::poisspdf;
///
/// // P(X=2 | lambda=3) = (3^2 * e^-3) / 2! = 9 * 0.04978706836 / 2 = 0.2240418
/// assert!((poisspdf(2.0, 3.0) - 0.2240418076553877).abs() < 1e-15);
///
/// // P(X=0 | lambda=0) = 1
/// assert_eq!(poisspdf(0.0, 0.0), 1.0);
/// ```
pub fn poisspdf(x: f64, lambda: f64) -> f64 {
    // 1. Handle NaN propagating cases
    if x.is_nan() || lambda.is_nan() {
        return f64::NAN;
    }

    // 2. Invalid parameter space: lambda cannot be negative
    if lambda < 0.0 {
        return f64::NAN;
    }

    // 3. Exact evaluation edge case specified by MATLAB: x == 0 and lambda == 0
    if x == 0.0 && lambda == 0.0 {
        return 1.0;
    }

    // 4. The density function is strictly zero unless x is a non-negative integer
    if x < 0.0 || x != x.round() {
        return 0.0;
    }

    // 5. If lambda is exactly 0, any x > 0 has a probability mass of 0
    if lambda == 0.0 {
        return 0.0;
    }

    // --- Core Calculation Block ---
    // At this stage, we are guaranteed: x >= 0, x == round(x), and lambda > 0.

    // Constant: log(sqrt(2 * pi))
    const LN_SR_2PI: f64 = 0.9189385332046727; 

    // Guard against underflow or division flags via realmin checks
    let small_x = x <= lambda * f64::MIN_POSITIVE;
    let large_x = lambda < x * f64::MIN_POSITIVE;

    if small_x {
        // If x is extremely small relative to lambda
        (-lambda).exp()
    } else if large_x {
        // If lambda is extremely small relative to x
        (-lambda + x * lambda.ln() - gammaln(x + 1.0)).exp()
    } else {
        // High-precision standard pathway using Loader's algorithm
        (-LN_SR_2PI - 0.5 * x.ln() - stirlerr(x) - binodeviance(x, lambda)).exp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-15;

    #[test]
    fn test_poisspdf_basic() {
        // P(X=2 | lambda=3)
        assert!((poisspdf(2.0, 3.0) - 0.2240418076553877).abs() < TOL);
        // P(X=0 | lambda=1)
        assert!((poisspdf(0.0, 1.0) - 0.36787944117144233).abs() < TOL);
        // P(X=1 | lambda=1)
        assert!((poisspdf(1.0, 1.0) - 0.36787944117144233).abs() < TOL);
        // P(X=5 | lambda=2)
        assert!((poisspdf(5.0, 2.0) - 0.0360894088630967).abs() < TOL);
    }

    #[test]
    fn test_poisspdf_edge_cases() {
        // x=0, lambda=0
        assert_eq!(poisspdf(0.0, 0.0), 1.0);
        // x > 0, lambda=0
        assert_eq!(poisspdf(1.0, 0.0), 0.0);
        // Non-integer x
        assert_eq!(poisspdf(1.5, 1.0), 0.0);
        // Negative x
        assert_eq!(poisspdf(-1.0, 1.0), 0.0);
    }

    #[test]
    fn test_poisspdf_invalid_inputs() {
        assert!(poisspdf(f64::NAN, 1.0).is_nan());
        assert!(poisspdf(1.0, f64::NAN).is_nan());
        assert!(poisspdf(1.0, -1.0).is_nan());
    }

    #[test]
    fn test_poisspdf_large_values() {
        // Large x, lambda (triggers Loader's expansion)
        assert!((poisspdf(100.0, 50.0) - 1.630319352147727e-10).abs() < 1e-14);
        // Small lambda, large x (triggers direct log-probability formula)
        assert!((poisspdf(10.0, 0.1) - 2.493489357462383e-17).abs() < 1e-32); // Adjusted tolerance for very small numbers
    }
}