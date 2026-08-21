use crate::consts::BERNOULLI_EVEN;

/// Computes the tetragamma function, <math><msup><mi>ψ</mi><mo>(</mo><mn>2</mn><mo>)</mo></msup><mo>(</mo><mi>x</mi><mo>)</mo></math>,
/// the second derivative of the digamma function.
///
/// # Numerical stability
/// This implementation uses:
/// - Reflection for negative values.
/// - Recurrence shifting for small positive values.
/// - Asymptotic expansion for large values.
///
/// # Special cases
/// - Returns `f64::NAN` for `NaN`.
/// - Returns `f64::NAN` at non-positive integer poles.
/// - Returns `0.0` for `+∞`.
///
/// # Examples
/// ```
/// use abax::tetragamma;
///
/// assert!((tetragamma(1.0) + 2.4041138063191885).abs() < 1e-13);
/// assert!((tetragamma(0.5) + 16.828796644234316).abs() < 1e-12);
/// ```
pub fn tetragamma(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    if x.is_infinite() {
        return if x.is_sign_positive() { 0.0 } else { f64::NAN };
    }
    if x <= 0.0 && x == x.floor() {
        return f64::NAN;
    }

    if x < 0.5 {
        let pix = std::f64::consts::PI * x;
        let sin_pix = pix.sin();
        let cos_pix = pix.cos();
        let csc2 = 1.0 / (sin_pix * sin_pix);
        // ψ2(x) = ψ2(1-x) - 2π^3 cot(πx)csc^2(πx)
        return tetragamma(1.0 - x)
            - 2.0 * std::f64::consts::PI.powi(3) * (cos_pix / sin_pix) * csc2;
    }

    let mut xx = x;
    let mut acc = 0.0;
    while xx < 10.0 {
        acc -= 2.0 / (xx * xx * xx);
        xx += 1.0;
    }

    let inv = 1.0 / xx;
    let inv2 = inv * inv;

    // ψ2(x) = -1/x^2 - 1/x^3 - Σ_{n>=1}(2n+1)B_{2n}/x^(2n+2)
    let mut series = -inv2 - inv2 * inv;
    let mut p = inv2 * inv2; // 1/x^4

    for (k, &b2n) in BERNOULLI_EVEN.iter().skip(1).enumerate() {
        let n = (k + 1) as f64;
        let coeff = 2.0 * n + 1.0;
        let term = -coeff * b2n * p;
        series += term;
        p *= inv2;
        if term.abs() < 1e-18 {
            break;
        }
    }

    acc + series
}

#[cfg(test)]
mod tests {
    use super::*;


    fn assert_approx_eq(actual: f64, expected: f64, eps: f64) {
        let d = (actual - expected).abs();
        assert!(
            d < eps,
            "actual={} expected={} diff={} eps={}",
            actual,
            expected,
            d,
            eps
        );
    }

    #[test]
    fn test_special_cases() {
        assert!(tetragamma(f64::NAN).is_nan());
        assert_eq!(tetragamma(f64::INFINITY), 0.0);
        assert!(tetragamma(-1.0).is_nan());
        assert!(tetragamma(0.0).is_nan());
    }

    #[test]
    fn test_known_values() {
        assert_approx_eq(tetragamma(1.0), -2.4041138063191885, 1e-13);
        assert_approx_eq(tetragamma(0.5), -16.828796644234316, 1e-12);
        assert_approx_eq(tetragamma(5.0), -0.04878973224511449, 1e-14);
    }

    #[test]
    fn test_recurrence() {
        let x = 2.75;
        let lhs = tetragamma(x + 1.0);
        let rhs = tetragamma(x) + 2.0 / (x * x * x);
        assert_approx_eq(lhs, rhs, 1e-13);
    }
}
