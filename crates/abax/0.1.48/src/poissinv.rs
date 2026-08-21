use crate::{gammainc, erfcinv};

/// Inverse of the Poisson cumulative distribution function for scalar values.
///
/// Returns the smallest integer value `x` such that the Poisson cdf equals or exceeds `p`.
///
/// # Mathematical Definition
/// Finds the smallest integer `x` such that:
/// <math display="block">
///   <mi>P</mi><mo>(</mo><mi>X</mi><mo>≤</mo><mi>x</mi><mo>;</mo><mi>λ</mi><mo>)</mo>
///   <mo>=</mo>
///   <munderover><mo>∑</mo><mrow><mi>i</mi><mo>=</mo><mn>0</mn></mrow><mi>x</mi></munderover>
///   <mfrac>
///     <mrow><msup><mi>λ</mi><mi>i</mi></msup><msup><mi>e</mi><mrow><mo>-</mo><mi>λ</mi></mrow></msup></mrow>
///     <mrow><mi>i</mi><mo>!</mo></mrow>
///   </mfrac>
///   <mo>≥</mo>
///   <mi>p</mi>
/// </math>
///
/// # Implementation Details
/// - For `lambda > 10.0`, an initial guess for `x` is made using a Cornish-Fisher
///   expansion (normal approximation), and then a directed search (incrementing or
///   decrementing `x`) is performed to find the exact integer.
/// - For `lambda <= 10.0`, a linear search starting from `x = 0` is performed.
/// - The cumulative probabilities are computed using `poisscdf`.
///
/// # Domain
/// - `p`: Probability, <math><mn>0</mn><mo>≤</mo><mi>p</mi><mo>≤</mo><mn>1</mn></math>.
/// - `lambda`: Non-negative real number, <math><mi>λ</mi><mo>≥</mo><mn>0</mn></math>.
/// - Returns `NaN` if `p` or `lambda` are `NaN`, `p` is outside `[0, 1]`, or `lambda < 0`.
///
/// # Examples
/// ```
/// use abax::poissinv;
///
/// // Small lambda (linear search)
/// // P(X <= 0 | lambda=1.0) approx 0.3678
/// // P(X <= 1 | lambda=1.0) approx 0.7357
/// assert_eq!(poissinv(0.5, 1.0), 1.0);
///
/// // Large lambda (normal approximation + search)
/// // P(X <= 90 | lambda=100.0) approx 0.1714
/// // P(X <= 91 | lambda=100.0) approx 0.1906
/// assert_eq!(poissinv(0.2, 100.0), 92.0);
///
/// // Edge cases
/// assert_eq!(poissinv(0.0, 5.0), 0.0);
/// assert_eq!(poissinv(1.0, 5.0), f64::INFINITY);
/// assert_eq!(poissinv(0.5, 0.0), 0.0);
/// ```
pub fn poissinv(p: f64, lambda: f64) -> f64 {
    // 1. Handle NaN propagating cases
    if p.is_nan() || lambda.is_nan() {
        return f64::NAN;
    }

    // 2. Return NaN if arguments are outside operational limits
    if lambda < 0.0 || p < 0.0 || p > 1.0 {
        return f64::NAN;
    }

    // 3. Exact evaluation boundary cases
    if p == 0.0 {
        return 0.0;
    }
    if lambda > 0.0 && p == 1.0 {
        return f64::INFINITY;
    }
    if lambda == 0.0 {
        return 0.0;
    }

    // --- Core Calculation Blocks ---

    // Pathway A: For large lambda (> 10), use Cornish-Fisher / Normal approximation 
    // as an initial guess, then adjust via directed search.
    if lambda > 10.0 {
        // x1 = -sqrt(2) * erfcinv(2 * p)
        let x1 = -2.0f64.sqrt() * erfcinv(2.0 * p);
        
        // count = max(ceil(sqrt(lambda) * x1 + lambda), 0)
        let mut count = (lambda.sqrt() * x1 + lambda).ceil().max(0.0);
        
        // Initial evaluation using upper regularized incomplete gamma (matching poisscdf)
        let y = gammainc(lambda, count + 1.0, false, false);

        if y < p {
            // Check upward loop
            loop {
                let y_new = gammainc(lambda, count + 2.0, false, false);
                count += 1.0;
                if y_new >= p {
                    break;
                }
            }
        } else {
            // Check downward loop
            loop {
                // If count reaches 0, we can't search lower
                if count <= 0.0 {
                    break;
                }
                let y_new = gammainc(lambda, count, false, false);
                if y_new < p {
                    break;
                }
                count -= 1.0;
            }
        }
        return count;
    }

    // Pathway B: For small lambda (<= 10), linear search starting exactly from x = 0
    let mut count = 0.0;
    loop {
        let cum_dist = gammainc(lambda, count + 1.0, false, false);
        if cum_dist >= p {
            break;
        }
        count += 1.0;
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::poisscdf;

    #[test]
    fn test_poissinv_basic_small_lambda() {
        // lambda = 1.0
        // P(X<=0) = 0.367879
        // P(X<=1) = 0.735758
        assert_eq!(poissinv(0.3, 1.0), 0.0);
        assert_eq!(poissinv(0.367879, 1.0), 0.0);
        assert_eq!(poissinv(0.367880, 1.0), 1.0);
        assert_eq!(poissinv(0.7, 1.0), 1.0);

        // lambda = 5.0
        // P(X<=4) = 0.440493
        // P(X<=5) = 0.615960
        assert_eq!(poissinv(0.5, 5.0), 5.0);
        assert_eq!(poissinv(0.4, 5.0), 4.0);
        assert_eq!(poissinv(0.65, 5.0), 6.0);
    }

    #[test]
    fn test_poissinv_basic_large_lambda() {
        // lambda = 20.0 (triggers normal approximation path)
        // P(X<=19) = 0.47025
        // P(X<=20) = 0.55910
        assert_eq!(poissinv(0.5, 20.0), 20.0);
        assert_eq!(poissinv(0.4, 20.0), 19.0);
        assert_eq!(poissinv(0.6, 20.0), 21.0);

        // lambda = 100.0
        // P(X<=99) = 0.4757
        // P(X<=100) = 0.5220
        assert_eq!(poissinv(0.5, 100.0), 100.0);
        assert_eq!(poissinv(0.1, 100.0), 87.0);
        assert_eq!(poissinv(0.9, 100.0), 113.0);
    }

    #[test]
    fn test_poissinv_boundary_p() {
        // p = 0.0
        assert_eq!(poissinv(0.0, 1.0), 0.0);
        assert_eq!(poissinv(0.0, 100.0), 0.0);
        assert_eq!(poissinv(0.0, 0.0), 0.0);

        // p = 1.0
        assert_eq!(poissinv(1.0, 1.0), f64::INFINITY);
        assert_eq!(poissinv(1.0, 100.0), f64::INFINITY);
        assert_eq!(poissinv(1.0, 0.0), 0.0); // Special case for lambda=0
    }

    #[test]
    fn test_poissinv_boundary_lambda() {
        // lambda = 0.0
        assert_eq!(poissinv(0.0, 0.0), 0.0);
        assert_eq!(poissinv(0.5, 0.0), 0.0);
        assert_eq!(poissinv(1.0, 0.0), 0.0);
    }

    #[test]
    fn test_poissinv_invalid_inputs() {
        assert!(poissinv(f64::NAN, 5.0).is_nan());
        assert!(poissinv(0.5, f64::NAN).is_nan());
        assert!(poissinv(-0.1, 5.0).is_nan());
        assert!(poissinv(1.1, 5.0).is_nan());
        assert!(poissinv(0.5, -1.0).is_nan());
    }

    #[test]
    fn test_poissinv_roundtrip_small_lambda() {
        let lambda = 5.0;
        for x in 0..=15 { // Test a range of x values
            let p = poisscdf(x as f64, lambda, false);
            let inv_x = poissinv(p, lambda);
            // Due to discreteness, poissinv(poisscdf(x)) might return x or x-1.
            // It should return the smallest x such that CDF >= p.
            // So, poissinv(p) should be x.
            assert_eq!(inv_x, x as f64, "Roundtrip failed for x={}, lambda={}", x, lambda);
        }
    }

    #[test]
    fn test_poissinv_roundtrip_large_lambda() {
        let lambda = 50.0;
        // Test around the mean and in the tails
        for x_offset in -20..=20 {
            let x = lambda + x_offset as f64;
            if x < 0.0 { continue; }

            let p = poisscdf(x, lambda, false);
            let inv_x = poissinv(p, lambda);
            // Due to discreteness, poissinv(poisscdf(x)) might return x or x-1.
            // It should return the smallest x such that CDF >= p.
            // So, poissinv(p) should be x.
            assert_eq!(inv_x, x, "Roundtrip failed for x={}, lambda={}", x, lambda);
        }
    }
}