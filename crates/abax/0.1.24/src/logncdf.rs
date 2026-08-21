use crate::normcdf;

/// Lognormal cumulative distribution function (CDF).
///
/// Given a value `x`, a mean `mu`, and a standard deviation `sigma`
/// of the associated normal distribution, this function returns the
/// probability that a lognormal random variable is less than or equal to `x`.
///
/// # Mathematical Definition
/// For a lognormal distribution with parameters <math><mi>μ</mi></math> and <math><mi>σ</mi></math>:
/// - Lower tail (`upper = false`): <math><mi>F</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>μ</mi><mo>,</mo><mi>σ</mi><mo>)</mo><mo>=</mo><mi>Φ</mi><mo>(</mo><mfrac><mrow><mi>ln</mi><mi>x</mi><mo>-</mo><mi>μ</mi></mrow><mi>σ</mi></mfrac><mo>)</mo></math>
/// - Upper tail (`upper = true`): <math><mn>1</mn><mo>-</mo><mi>F</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>μ</mi><mo>,</mo><mi>σ</mi><mo>)</mo><mo>=</mo><mn>1</mn><mo>-</mo><mi>Φ</mi><mo>(</mo><mfrac><mrow><mi>ln</mi><mi>x</mi><mo>-</mo><mi>μ</mi></mrow><mi>σ</mi></mfrac><mo>)</mo></math>
///
/// where <math><mi>Φ</mi></math> is the standard normal cumulative distribution function.
///
/// # Domain
/// - `x > 0` (Values <math><mi>x</mi><mo>≤</mo><mn>0</mn></math> return 0 for lower tail, 1 for upper tail).
/// - `sigma > 0`
/// - Returns `NaN` if `sigma` is non-positive or any input is `NaN`.
///
/// # Examples
/// ```
/// use abax::logncdf;
///
/// // For a standard lognormal distribution (mu=0, sigma=1)
/// assert!((logncdf(1.0, 0.0, 1.0, false) - 0.5).abs() < 1e-15);
/// // Upper tail at the median
/// assert!((logncdf(1.0, 0.0, 1.0, true) - 0.5).abs() < 1e-15);
/// ```
pub fn logncdf(x: f64, mu: f64, sigma: f64, upper: bool) -> f64 {
    if x.is_nan() || mu.is_nan() || sigma.is_nan() || sigma <= 0.0 {
        return f64::NAN;
    }

    if x <= 0.0 {
        return if upper { 1.0 } else { 0.0 };
    }

    normcdf(x.ln(), mu, sigma, upper)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logncdf_standard() {
        let tol = 1e-14;
        // Median at x=1 (ln(1)=0)
        assert!((logncdf(1.0, 0.0, 1.0, false) - 0.5).abs() < tol);
        assert!((logncdf(1.0, 0.0, 1.0, true) - 0.5).abs() < tol);
    }

    #[test]
    fn test_logncdf_edge_cases() {
        assert_eq!(logncdf(0.0, 0.0, 1.0, false), 0.0);
        assert_eq!(logncdf(-1.0, 0.0, 1.0, false), 0.0);
        assert_eq!(logncdf(0.0, 0.0, 1.0, true), 1.0);

        assert!(logncdf(1.0, 0.0, 0.0, false).is_nan());
        assert!(logncdf(1.0, 0.0, -1.0, false).is_nan());
        assert!(logncdf(f64::NAN, 0.0, 1.0, false).is_nan());
    }
}
