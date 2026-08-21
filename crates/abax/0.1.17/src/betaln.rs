use crate::gammaln;

/// Natural logarithm of the Beta function.
///
/// The Beta function is defined as:
/// B(z, w) = Γ(z)Γ(w) / Γ(z + w)
///
/// This function computes ln(B(z, w)) using the logarithmic Gamma function
/// to maintain numerical stability and avoid overflow/underflow.
///
/// # Domain
/// - `z > 0`, `w > 0`
/// - Invalid inputs return `NaN`.
pub fn betaln(z: f64, w: f64) -> f64 {
    if z <= 0.0 || w <= 0.0 || z.is_nan() || w.is_nan() {
        return f64::NAN;
    }
    gammaln(z) + gammaln(w) - gammaln(z + w)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_betaln_values() {
        // B(1, 1) = 1, ln(1) = 0
        assert!((betaln(1.0, 1.0)).abs() < 1e-14);
        // B(2, 1) = 1/2, ln(0.5) = -0.6931471805599453
        assert!((betaln(2.0, 1.0) - (-0.6931471805599453)).abs() < 1e-14);
        // Symmetry: B(z, w) = B(w, z)
        assert!((betaln(2.5, 3.5) - betaln(3.5, 2.5)).abs() < 1e-14);
    }

    #[test]
    fn test_betaln_domain() {
        assert!(betaln(0.0, 1.0).is_nan());
        assert!(betaln(1.0, -1.0).is_nan());
        assert!(betaln(f64::NAN, 1.0).is_nan());
    }
}
