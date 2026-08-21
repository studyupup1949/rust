use crate::consts::*;

/// Computes the natural logarithm of the Gamma function, <math><mi>ln</mi><mo>(</mo><mi>Γ</mi><mo>(</mo><mi>x</mi><mo>)</mo><mo>)</mo></math>.
///
/// This implementation provides high precision across the positive real axis:
/// - **Small values (<math><mi>x</mi><mo>&lt;</mo><msup><mn>10</mn><mrow><mo>-</mo><mn>5</mn></mrow></msup></math>)**: Utilizes a Taylor series expansion involving the Euler-Mascheroni
///   constant and Riemann Zeta values.
/// - **Large values**: Employs Stirling's asymptotic expansion using high-precision coefficients.
/// - **Intermediate values (<math><mi>x</mi><mo>&lt;</mo><mn>10</mn></math>)**: Uses the recurrence relation <math><mi>Γ</mi><mo>(</mo><mi>x</mi><mo>+</mo><mn>1</mn><mo>)</mo><mo>=</mo><mi>x</mi><mi>Γ</mi><mo>(</mo><mi>x</mi><mo>)</mo></math>
///   to shift the argument into the optimal range for the Stirling approximation.
///
/// # Mathematical Context
/// The Gamma function <math><mi>Γ</mi><mo>(</mo><mi>x</mi><mo>)</mo></math> extends the factorial function to real numbers: <math><mi>Γ</mi><mo>(</mo><mi>n</mi><mo>)</mo><mo>=</mo><mo>(</mo><mi>n</mi><mo>-</mo><mn>1</mn><mo>)</mo><mo>!</mo></math>
/// for positive integers <math><mi>n</mi></math>.
///
/// # Edge Cases
/// - Returns `f64::NAN` if <math><mi>x</mi><mo>≤</mo><mn>0</mn></math> or <math><mi>x</mi></math> is `NaN`.
/// - Returns `f64::INFINITY` if <math><mi>x</mi></math> is `INFINITY`.
///
/// # Examples
/// ```
/// use abax::gammaln;
/// assert_eq!(gammaln(1.0), 0.0);
/// assert!((gammaln(5.0) - 3.178053830347945).abs() < 1e-14);
/// ```
pub fn gammaln(x: f64) -> f64 {
    match x {
        _ if x <= 0.0 || x.is_nan() => f64::NAN,
        _ if x.is_infinite() => f64::INFINITY,
        _ if x < 0.00001 => {
            // Taylor
            -f64::ln(x) + x * (-EULER_MASCHERONI + x * RIEMANN_ZETA[2] / 2.0)
        }
        1.0 | 2.0 => 0.0,
        _ => {
            // Stirling
            let mut current_x = x;
            let mut correction = 1.0;
            // gamma(z + 1) = z * gamma(z)
            while current_x < 10.0 {
                correction /= current_x;
                current_x += 1.0;
            }

            let inv_x_sq = 1.0 / (current_x * current_x);
            LN_2PI * 0.5 + f64::ln(correction) + f64::ln(current_x) * (current_x - 0.5) - current_x
                + ((((((inv_x_sq * STIRLING_ASYMPTOTIC_SERIES[6]
                    + STIRLING_ASYMPTOTIC_SERIES[5])
                    * inv_x_sq
                    + STIRLING_ASYMPTOTIC_SERIES[4])
                    * inv_x_sq
                    + STIRLING_ASYMPTOTIC_SERIES[3])
                    * inv_x_sq
                    + STIRLING_ASYMPTOTIC_SERIES[2])
                    * inv_x_sq
                    + STIRLING_ASYMPTOTIC_SERIES[1])
                    * inv_x_sq
                    + STIRLING_ASYMPTOTIC_SERIES[0])
                    / current_x
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to assert that two floats are approximately equal.
    fn assert_approx_eq(actual: f64, expected: f64, epsilon: f64) {
        if actual.is_nan() && expected.is_nan() {
            return;
        }
        if actual.is_infinite() && expected.is_infinite() {
            assert_eq!(actual.is_sign_positive(), expected.is_sign_positive());
            return;
        }
        let diff = (actual - expected).abs();
        assert!(
            diff < epsilon,
            "Assertion failed: actual {} != expected {} (diff {} > epsilon {})",
            actual,
            expected,
            diff,
            epsilon
        );
    }

    #[test]
    fn test_gammaln_special_cases() {
        assert!(gammaln(f64::NAN).is_nan());
        assert!(gammaln(0.0).is_nan());
        assert!(gammaln(-1.0).is_nan());
        assert_eq!(gammaln(f64::INFINITY), f64::INFINITY);
    }

    #[test]
    fn test_gammaln_small_values() {
        // Triggers the Taylor expansion path (x < 0.00001)
        assert_approx_eq(gammaln(1e-7), 16.11809559323676, 1e-10);
    }

    #[test]
    fn test_gammaln_integers() {
        // ln(Gamma(1)) = ln(1) = 0
        assert_eq!(gammaln(1.0), 0.0);
        // ln(Gamma(2)) = ln(1) = 0
        assert_eq!(gammaln(2.0), 0.0);
        // ln(Gamma(3)) = ln(2)
        assert_approx_eq(gammaln(3.0), std::f64::consts::LN_2, 1e-14);
        // ln(Gamma(10)) = ln(9!) = ln(362880)
        assert_approx_eq(gammaln(10.0), 12.801827480081469, 1e-14);
    }

    #[test]
    fn test_gammaln_half_integers() {
        // ln(Gamma(0.5)) = ln(sqrt(pi))
        assert_approx_eq(gammaln(0.5), 0.5723649429247001, 1e-14);
        // ln(Gamma(1.5)) = ln(0.5 * sqrt(pi))
        assert_approx_eq(gammaln(1.5), -0.12078223763524522, 1e-14);
    }

    #[test]
    fn test_gammaln_extreme_values() {
        // Extreme small (positive)
        // As x -> 0, Gamma(x) ~ 1/x, so ln(Gamma(x)) ~ -ln(x)
        let x_tiny = 1e-300;
        assert_approx_eq(gammaln(x_tiny), -f64::ln(x_tiny), 1e-12);

        // Extreme large
        // ln(Gamma(1e100)) is approximately 2.292585...e102
        // We use a large epsilon because of the magnitude of the result
        assert_approx_eq(gammaln(1e100), 2.2925850929940457e102, 1e88);
    }
}
