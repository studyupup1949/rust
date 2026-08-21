use crate::gammainc;

/// Poisson cumulative distribution function (CDF).
///
/// Returns the probability that a Poisson random variable with parameter `lambda`
/// is less than or equal to `x` (lower tail) or greater than `x` (upper tail).
///
/// # Mathematical Definition
/// For a Poisson distribution with parameter <math><mi>λ</mi></math>:
/// - Lower tail (`upper = false`):
/// <math display="block">
///   <mi>F</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>λ</mi><mo>)</mo>
///   <mo>=</mo>
///   <munderover><mo>∑</mo><mrow><mi>i</mi><mo>=</mo><mn>0</mn></mrow><mrow><mo>⌊</mo><mi>x</mi><mo>⌋</mo></mrow></munderover>
///   <mfrac>
///     <mrow><msup><mi>λ</mi><mi>i</mi></msup><msup><mi>e</mi><mrow><mo>-</mo><mi>λ</mi></mrow></msup></mrow>
///     <mrow><mi>i</mi><mo>!</mo></mrow>
///   </mfrac>
///   <mo>=</mo>
///   <mi>Q</mi><mrow><mo>(</mo><mo>⌊</mo><mi>x</mi><mo>⌋</mo><mo>+</mo><mn>1</mn><mo>,</mo><mi>λ</mi><mo>)</mo></mrow>
/// </math>
/// - Upper tail (`upper = true`):
/// <math display="block">
///   <mn>1</mn><mo>-</mo><mi>F</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>λ</mi><mo>)</mo>
///   <mo>=</mo>
///   <munderover><mo>∑</mo><mrow><mi>i</mi><mo>=</mo><mo>⌊</mo><mi>x</mi><mo>⌋</mo><mo>+</mo><mn>1</mn></mrow><mi>∞</mi></munderover>
///   <mfrac>
///     <mrow><msup><mi>λ</mi><mi>i</mi></msup><msup><mi>e</mi><mrow><mo>-</mo><mi>λ</mi></mrow></msup></mrow>
///     <mrow><mi>i</mi><mo>!</mo></mrow>
///   </mfrac>
///   <mo>=</mo>
///   <mi>P</mi><mrow><mo>(</mo><mo>⌊</mo><mi>x</mi><mo>⌋</mo><mo>+</mo><mn>1</mn><mo>,</mo><mi>λ</mi><mo>)</mo></mrow>
/// </math>
/// where <math><mi>P</mi></math> and <math><mi>Q</mi></math> are the regularized lower and upper incomplete gamma functions.
///
/// # Examples
/// ```
/// use abax::poisscdf;
///
/// // P(X <= 2 | lambda=3) approx 0.42319
/// assert!((poisscdf(2.0, 3.0, false) - 0.42319008112684364).abs() < 1e-15);
///
/// // Upper tail: P(X > 2 | lambda=3) approx 0.57681
/// assert!((poisscdf(2.0, 3.0, true) - 0.5768099188731564).abs() < 1e-15);
/// ```
pub fn poisscdf(x: f64, lambda: f64, upper: bool) -> f64 {
    // 1. Handle NaN propagating cases
    if x.is_nan() || lambda.is_nan() {
        return f64::NAN;
    }

    // forces floor(x) right at the beginning of the evaluation
    let x_floor = x.floor();

    // 2. Return NaN for invalid or indeterminate inputs
    if lambda < 0.0 || (x_floor == f64::INFINITY && lambda == f64::INFINITY) {
        return f64::NAN;
    }

    // 3. Return early for x == Inf as long as lambda is valid and finite
    if x_floor == f64::INFINITY && lambda > 0.0 && lambda.is_finite() {
        return if upper { 0.0 } else { 1.0 };
    }

    // 4. Return early for x < 0 as long as lambda is valid and finite
    if x_floor < 0.0 && lambda > 0.0 && lambda.is_finite() {
        return if upper { 1.0 } else { 0.0 };
    }

    // 5. Filter out remaining edge cases (e.g., lambda is infinity but x is finite)
    if x_floor < 0.0 || !lambda.is_finite() {
        return 0.0; 
    }

    // --- Core Calculation Block ---
    // At this stage: x_floor >= 0 and finite lambda.
    let alpha = x_floor + 1.0;

    if upper {
        gammainc(lambda, alpha, true, false) 
    } else {
        gammainc(lambda, alpha, false, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poisscdf_basic() {
        let tol = 1e-15;
        // Reference values from MATLAB/SciPy
        assert!((poisscdf(2.0, 3.0, false) - 0.42319008112684364).abs() < tol);
        assert!((poisscdf(2.0, 3.0, true) - 0.5768099188731564).abs() < tol);
        assert!((poisscdf(0.0, 1.0, false) - 0.36787944117144233).abs() < tol);
    }

    #[test]
    fn test_poisscdf_lambda_zero() {
        // Spike at 0: F(x) is 1 for x >= 0, 0 otherwise
        assert_eq!(poisscdf(0.0, 0.0, false), 1.0);
        assert_eq!(poisscdf(-1.0, 0.0, false), 0.0);
        assert_eq!(poisscdf(0.0, 0.0, true), 0.0);
    }

    #[test]
    fn test_poisscdf_boundaries() {
        assert_eq!(poisscdf(-1.0, 5.0, false), 0.0);
        assert_eq!(poisscdf(-1.0, 5.0, true), 1.0);
        assert_eq!(poisscdf(f64::INFINITY, 5.0, false), 1.0);
        assert_eq!(poisscdf(f64::INFINITY, 5.0, true), 0.0);
        assert_eq!(poisscdf(5.0, f64::INFINITY, false), 0.0);
        assert_eq!(poisscdf(5.0, f64::INFINITY, true), 0.0);
    }

    #[test]
    fn test_poisscdf_invalid() {
        assert!(poisscdf(2.0, -1.0, false).is_nan());
        assert!(poisscdf(f64::NAN, 3.0, false).is_nan());
        assert!(poisscdf(2.0, f64::NAN, false).is_nan());
        assert!(poisscdf(f64::INFINITY, f64::INFINITY, false).is_nan());
    }
}