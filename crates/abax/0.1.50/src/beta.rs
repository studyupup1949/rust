use crate::gammaln;

/// Beta function <math><mrow><mi>B</mi><mo>(</mo><mi>z</mi><mo>,</mo><mi>w</mi><mo>)</mo><mo>=</mo><mfrac><mrow><mi>Γ</mi><mo>(</mo><mi>z</mi><mo>)</mo><mi>Γ</mi><mo>(</mo><mi>w</mi><mo>)</mo></mrow><mrow><mi>Γ</mi><mo>(</mo><mi>z</mi><mo>+</mo><mi>w</mi><mo>)</mo></mrow></mfrac></mrow></math>.
///
/// This implementation evaluates the function using the natural logarithm of the Gamma function
/// to avoid intermediate overflow for large arguments:
/// <math><mrow><mi>B</mi><mo>(</mo><mi>z</mi><mo>,</mo><mi>w</mi><mo>)</mo><mo>=</mo><mi>exp</mi><mo>(</mo><mi>ln</mi><mi>Γ</mi><mo>(</mo><mi>z</mi><mo>)</mo><mo>+</mo><mi>ln</mi><mi>Γ</mi><mo>(</mo><mi>w</mi><mo>)</mo><mo>-</mo><mi>ln</mi><mi>Γ</mi><mo>(</mo><mi>z</mi><mo>+</mo><mi>w</mi><mo>)</mo><mo>)</mo></mrow></math>.
///
/// # Domain
/// - `z > 0`, `w > 0`
/// - Returns `NaN` if either argument is less than or equal to zero.
///
/// # Examples
/// ```
/// use abax::beta;
///
/// let result = beta(2.0, 3.0);
/// assert!((result - 1.0/12.0).abs() < 1e-15);
/// ```
pub fn beta(z: f64, w: f64) -> f64 {
    if z <= 0.0 || w <= 0.0 {
        return f64::NAN;
    }
    (gammaln(z) + gammaln(w) - gammaln(z + w)).exp()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_beta_values() {
        let tol = 1e-14;
        // B(1, 1) = 1
        assert!((beta(1.0, 1.0) - 1.0).abs() < tol);
        // B(2, 1) = 1/2
        assert!((beta(2.0, 1.0) - 0.5).abs() < tol);
        // B(3, 2) = 1/12
        assert!((beta(3.0, 2.0) - 1.0 / 12.0).abs() < tol);
        // B(0.5, 0.5) = pi
        assert!((beta(0.5, 0.5) - PI).abs() < tol);
    }

    #[test]
    fn test_beta_symmetry() {
        assert_eq!(beta(2.5, 4.2), beta(4.2, 2.5));
    }

    #[test]
    fn test_beta_domain() {
        assert!(beta(0.0, 1.0).is_nan());
        assert!(beta(1.0, 0.0).is_nan());
        assert!(beta(-0.5, 0.5).is_nan());
        assert!(beta(0.5, -0.5).is_nan());
        assert!(beta(f64::NAN, 1.0).is_nan());
    }
}
