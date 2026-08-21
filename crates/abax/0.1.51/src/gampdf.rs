use crate::stirlerr::stirlerr;
use crate::binodeviance::binodeviance;
use crate::gammaln;

fn f(z: f64, a: f64) -> f64 {
    // z term dominates
    if a <= f64::MIN_POSITIVE * z {
        return f64::exp(-z);
    }

    // Normal expansion through logs
    if z < f64::MIN_POSITIVE * a {
        return f64::exp(a * f64::ln(z) -z - gammaln(a + 1.0));
    }

    // Loader's saddle point expansion
    let lnsr2pi = 0.9189385332046727; // ln(sqrt(2*pi))
    f64::exp(-lnsr2pi - 0.5 * f64::ln(a) - stirlerr(a) - binodeviance(a,z))
}

/// Gamma probability density function (PDF).
///
/// Returns the probability density at `x` for the Gamma distribution
/// with shape parameter `a` and scale parameter `b`.
///
/// # Mathematical Definition
/// For a Gamma distribution with shape <math><mi>a</mi></math> and scale <math><mi>b</mi></math>:
/// <math display="block">
///   <mi>f</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>a</mi><mo>,</mo><mi>b</mi><mo>)</mo>
///   <mo>=</mo>
///   <mfrac>
///     <mrow>
///       <msup><mi>x</mi><mrow><mi>a</mi><mo>-</mo><mn>1</mn></mrow></msup>
///       <msup><mi>e</mi><mrow><mo>-</mo><mi>x</mi><mo>/</mo><mi>b</mi></mrow></msup>
///     </mrow>
///     <mrow>
///       <msup><mi>b</mi><mi>a</mi></msup>
///       <mi>Γ</mi><mo>(</mo><mi>a</mi><mo>)</mo>
///     </mrow>
///   </mfrac>
/// </math>
///
/// # Domain
/// - <math><mi>x</mi><mo>≥</mo><mn>0</mn></math>
/// - <math><mi>a</mi><mo>&gt;</mo><mn>0</mn></math>
/// - <math><mi>b</mi><mo>&gt;</mo><mn>0</mn></math>
/// - Returns `NaN` if any input is `NaN`, <math><mi>a</mi><mo>≤</mo><mn>0</mn></math>, or <math><mi>b</mi><mo>≤</mo><mn>0</mn></math>.
///
/// # Examples
/// ```
/// use abax::gampdf;
/// assert!((gampdf(1.0, 1.0, 1.0) - 0.36787944117144233).abs() < 1e-15);
/// ```
pub fn gampdf(x: f64, a: f64, b: f64) -> f64 {
    if a < 0.0 || b <= 0.0 || a.is_nan() || b.is_nan() || x.is_nan() {
        return f64::NAN;
    }

    // Scale
    let z = x / b;

    // Special cases
    if z == 0.0 && a == 1.0 && b > 0.0 {
        return 1.0 / b;
    }
    if z == 0.0 && a < 1.0 && b > 0.0 {
        return f64::INFINITY;
    }

    // Normal cases
    if z > 0.0 && z < f64::INFINITY && a > 0.0 && b > 0.0 {
        let a = a - 1.0;
        match a < 0.0 {
            true => f(z, a + 1.0) * f64::exp(f64::ln(a + 1.0) - f64::ln(z)) / b,
            false => f(z, a) / b,
        }
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gampdf_invalid_params() {
        assert!(gampdf(1.0, -1.0, 1.0).is_nan());
        assert!(gampdf(1.0, 1.0, 0.0).is_nan());
        assert!(gampdf(f64::NAN, 1.0, 1.0).is_nan());
    }

    #[test]
    fn test_gampdf_exponential_case() {
        // For a=1, Gamma reduces to Exponential(1/b)
        // f(x) = (1/b) * exp(-x/b)
        let b = 2.0;
        let x = 1.0;
        let expected = (1.0 / b) * f64::exp(-x / b);
        assert!((gampdf(x, 1.0, b) - expected).abs() < 1e-15);
        
        // x=0, a=1
        assert!((gampdf(0.0, 1.0, b) - 1.0/b).abs() < 1e-15);
    }

    #[test]
    fn test_gampdf_poles_at_zero() {
        // If a < 1, pdf(0) is infinity
        assert_eq!(gampdf(0.0, 0.5, 1.0), f64::INFINITY);
        // If a > 1, pdf(0) is 0
        assert_eq!(gampdf(0.0, 2.0, 1.0), 0.0);
    }

    #[test]
    fn test_gampdf_known_values() {
        let tol = 1e-14;
        // Reference values from SciPy: scipy.stats.gamma.pdf(x, a, scale=b)
        // gampdf(2, 3, 1) = 2^2 * exp(-2) / Gamma(3) = 4 * exp(-2) / 2 = 2 * exp(-2)
        assert!((gampdf(2.0, 3.0, 1.0) - 0.2706705664732254).abs() < tol);
        
        // gampdf(1, 0.5, 2)
        assert!((gampdf(1.0, 0.5, 2.0) - 0.2419707245191433).abs() < tol);
    }

    #[test]
    fn test_gampdf_large_params() {
        // Testing stability for larger shape parameters
        assert!(gampdf(100.0, 100.0, 1.0).is_finite());
        assert!(gampdf(1000.0, 500.0, 2.0).is_finite());
    }
}