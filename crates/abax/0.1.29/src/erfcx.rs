use crate::erfc::erfc;

/// Calculates the scaled complementary error function `erfcx(x) = exp(x^2) * erfc(x)`.
///
/// The scaled complementary error function is useful for large values of `x` where
/// `erfc(x)` would underflow to zero and `exp(x^2)` would overflow to infinity.
/// As `x` increases, `erfcx(x)` decays slowly toward zero as `1 / (x * sqrt(pi))`.
///
/// # Numerical stability
/// - For `x < 0`, the function grows very rapidly and is calculated as
///   `erfcx(x) = 2 * exp(x^2) - erfcx(|x|)`.
/// - For small `x >= 0`, it is calculated using the product `exp(x^2) * erfc(x)`.
/// - For large `x`, it is calculated using a continued fraction expansion to
///   maintain precision and avoid overflow/underflow.
///
/// # Examples
/// ```
/// use abax::erfcx;
///
/// assert!((erfcx(0.0) - 1.0).abs() < 1e-15);
/// assert!((erfcx(1.0) - 0.4275835761558071).abs() < 1e-15);
/// ```
pub fn erfcx(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    if x.is_infinite() {
        return if x.is_sign_positive() {
            0.0
        } else {
            f64::INFINITY
        };
    }

    if x < 0.0 {
        let x2 = x * x;
        // exp(x^2) overflows for x^2 > log(f64::MAX) ~= 709.78
        if x2 > 709.78 {
            return f64::INFINITY;
        }
        // Reflection formula: erfcx(x) = exp(x^2) * (2 - erfc(|x|)) = 2*exp(x^2) - erfcx(|x|)
        return 2.0 * x2.exp() - erfcx(-x);
    }

    // For positive x, erfcx is monotonically decreasing from 1 to 0.
    if x < 2.5 {
        // Standard definition is safe for small x.
        return (x * x).exp() * erfc(x);
    }

    // For large x, use Laplace's continued fraction expansion:
    // erfcx(x) = (1/sqrt(pi)) * [1 / (x + (1/2 / (x + (1 / (x + (3/2 / (x + ...)))))))]
    erfcx_cf(x)
}

/// Evaluates the continued fraction for erfcx(x) using the modified Lentz's method.
fn erfcx_cf(x: f64) -> f64 {
    const INV_SQRT_PI: f64 = 0.5641895835477563;
    const TINY: f64 = 1.0e-100;

    let mut f = TINY;
    let mut c = f;
    let mut d = 0.0;

    for i in 1..200 {
        // Coefficients for the continued fraction:
        // a_1 = 1, b_1 = x
        // a_n = (n-1)/2, b_n = x  for n > 1
        let a = if i == 1 { 1.0 } else { (i as f64 - 1.0) * 0.5 };
        let b = x;

        d = b + a * d;
        if d == 0.0 { d = TINY; }
        c = b + a / c;
        if c == 0.0 { c = TINY; }

        d = 1.0 / d;
        let delta = c * d;
        f *= delta;

        if (delta - 1.0).abs() < 1e-16 {
            break;
        }
    }

    f * INV_SQRT_PI
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f64, expected: f64, tol: f64) {
        let diff = (actual - expected).abs();
        assert!(diff <= tol, "actual={actual:?}, expected={expected:?}, diff={diff:?}");
    }

    #[test]
    fn test_special_cases() {
        assert!(erfcx(f64::NAN).is_nan());
        assert_eq!(erfcx(f64::INFINITY), 0.0);
        assert_eq!(erfcx(f64::NEG_INFINITY), f64::INFINITY);
        assert_close(erfcx(0.0), 1.0, 1e-15);
    }

    #[test]
    fn test_known_values() {
        assert_close(erfcx(0.5), 0.615697223530000, 1e-14);
        assert_close(erfcx(1.0), 0.4275835761558071, 1e-15);
        assert_close(erfcx(5.0), 0.1116123285081272, 1e-15);
        assert_close(erfcx(10.0), 0.0561459483563720, 1e-15);
        assert_close(erfcx(-1.0), 5.008977599118573, 1e-14);
    }
}