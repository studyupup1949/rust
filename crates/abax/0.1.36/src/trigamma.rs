use crate::consts::BERNOULLI_EVEN;

/// Computes the trigamma function, <math><msup><mi>ψ</mi><mo>(</mo><mn>1</mn><mo>)</mo></msup><mo>(</mo><mi>x</mi><mo>)</mo></math>,
/// the first derivative of the digamma function:
/// <math><msup><mi>ψ</mi><mo>(</mo><mn>1</mn><mo>)</mo></msup><mo>(</mo><mi>x</mi><mo>)</mo><mo>=</mo><mfrac><mrow><msup><mi>d</mi><mn>2</mn></msup></mrow><mrow><mi>d</mi><msup><mi>x</mi><mn>2</mn></msup></mrow></mfrac><mi>ln</mi><mi>Γ</mi><mo>(</mo><mi>x</mi><mo>)</mo></math>.
///
/// # Numerical stability
/// This implementation combines three stable strategies:
/// - Reflection for negative values:
///   <math><msup><mi>ψ</mi><mo>(</mo><mn>1</mn><mo>)</mo></msup><mo>(</mo><mi>x</mi><mo>)</mo><mo>=</mo><msup><mi>π</mi><mn>2</mn></msup><msup><mi>csc</mi><mn>2</mn></msup><mo>(</mo><mi>π</mi><mi>x</mi><mo>)</mo><mo>-</mo><msup><mi>ψ</mi><mo>(</mo><mn>1</mn><mo>)</mo></msup><mo>(</mo><mn>1</mn><mo>-</mo><mi>x</mi><mo>)</mo></math>.
/// - Recurrence for small positive values:
///   <math><msup><mi>ψ</mi><mo>(</mo><mn>1</mn><mo>)</mo></msup><mo>(</mo><mi>x</mi><mo>)</mo><mo>=</mo><msup><mi>ψ</mi><mo>(</mo><mn>1</mn><mo>)</mo></msup><mo>(</mo><mi>x</mi><mo>+</mo><mn>1</mn><mo>)</mo><mo>+</mo><mfrac><mn>1</mn><msup><mi>x</mi><mn>2</mn></msup></mfrac></math>.
/// - Asymptotic expansion for large values:
///   <math><msup><mi>ψ</mi><mo>(</mo><mn>1</mn><mo>)</mo></msup><mo>(</mo><mi>x</mi><mo>)</mo><mo>≈</mo><mfrac><mn>1</mn><mi>x</mi></mfrac><mo>+</mo><mfrac><mn>1</mn><mrow><mn>2</mn><msup><mi>x</mi><mn>2</mn></msup></mrow></mfrac><mo>+</mo><munderover><mo>∑</mo><mrow><mi>n</mi><mo>=</mo><mn>1</mn></mrow><mi>∞</mi></munderover><mfrac><msub><mi>B</mi><mrow><mn>2</mn><mi>n</mi></mrow></msub><msup><mi>x</mi><mrow><mn>2</mn><mi>n</mi><mo>+</mo><mn>1</mn></mrow></msup></mfrac></math>.
///
/// # Special cases
/// - Returns `f64::NAN` for `NaN`.
/// - Returns `f64::NAN` at non-positive integer poles.
/// - Returns `0.0` for `+∞`.
///
/// # Examples
/// ```
/// use abax::trigamma;
///
/// assert!((trigamma(1.0) - 1.6449340668482264).abs() < 1e-14);
/// assert!((trigamma(0.5) - 4.934802200544679).abs() < 1e-14);
/// ```
pub fn trigamma(x: f64) -> f64 {
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
        let csc2 = 1.0 / (sin_pix * sin_pix);
        return std::f64::consts::PI * std::f64::consts::PI * csc2 - trigamma(1.0 - x);
    }

    let mut xx = x;
    let mut acc = 0.0;

    while xx < 10.0 {
        acc += 1.0 / (xx * xx);
        xx += 1.0;
    }

    let inv = 1.0 / xx;
    let inv2 = inv * inv;

    // psi1(x) = 1/x + 1/(2x^2) + Σ_{n>=1} B_{2n}/x^(2n+1)
    let mut series = inv + 0.5 * inv2;
    let mut p = inv * inv2; // 1/x^3

    for &b2n in BERNOULLI_EVEN.iter().skip(1) {
        let term = b2n * p;
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
        assert!(trigamma(f64::NAN).is_nan());
        assert_eq!(trigamma(f64::INFINITY), 0.0);
        assert!(trigamma(-1.0).is_nan());
        assert!(trigamma(0.0).is_nan());
    }

    #[test]
    fn test_known_values() {
        assert_approx_eq(trigamma(1.0), 1.6449340668482264, 1e-14);
        assert_approx_eq(trigamma(0.5), 4.934802200544679, 1e-14);
        assert_approx_eq(trigamma(5.0), 0.22132295573711533, 1e-14);
    }

    #[test]
    fn test_recurrence() {
        let x = 2.75;
        let lhs = trigamma(x + 1.0);
        let rhs = trigamma(x) - 1.0 / (x * x);
        assert_approx_eq(lhs, rhs, 1e-14);
    }

    #[test]
    fn test_reflection_noninteger_negative() {
        assert_approx_eq(trigamma(-0.5), 8.934802200544679, 1e-13);
    }
}
