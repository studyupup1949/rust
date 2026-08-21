use crate::consts::SQRT_2PI;

/// Normal probability density function (PDF).
///
/// Given a value `x`, a mean `mu`, and a standard deviation `sigma`,
/// this function returns the probability density at `x`.
///
/// # Mathematical Definition
/// For a normal distribution with mean <math><mi>μ</mi></math> and standard deviation <math><mi>σ</mi></math>:
/// <math display="block"><mi>f</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>μ</mi><mo>,</mo><mi>σ</mi><mo>)</mo><mo>=</mo><mfrac><mn>1</mn><mrow><mi>σ</mi><msqrt><mn>2</mn><mi>π</mi></msqrt></mrow></mfrac><msup><mi>e</mi><mrow><mo>-</mo><mfrac><mn>1</mn><mn>2</mn></mfrac><msup><mrow><mo>(</mo><mfrac><mrow><mi>x</mi><mo>-</mo><mi>μ</mi></mrow><mi>σ</mi></mfrac><mo>)</mo></mrow><mn>2</mn></msup></mrow></msup></math>
///
/// # Domain
/// - `sigma > 0`
/// - Returns `NaN` if `sigma <= 0` or any input is `NaN`.
///
/// # Examples
/// ```
/// use abax::normpdf;
///
/// // Peak of standard normal is 1/sqrt(2*pi) approx 0.39894228
/// assert!((normpdf(0.0, 0.0, 1.0) - 0.3989422804014327).abs() < 1e-15);
/// ```
pub fn normpdf(x: f64, mu: f64, sigma: f64) -> f64 {
    if x.is_nan() || mu.is_nan() || sigma.is_nan() || sigma <= 0.0 {
        return f64::NAN;
    }

    let z = (x - mu) / sigma;
    (-0.5 * z * z).exp() / (sigma * SQRT_2PI)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normpdf_standard() {
        let tol = 1e-15;
        // Peak at 0
        assert!((normpdf(0.0, 0.0, 1.0) - 0.3989422804014327).abs() < tol);
        // 1 sigma away
        assert!((normpdf(1.0, 0.0, 1.0) - 0.24197072451914337).abs() < tol);
        // Symmetry
        assert_eq!(normpdf(1.0, 0.0, 1.0), normpdf(-1.0, 0.0, 1.0));
    }

    #[test]
    fn test_normpdf_varied() {
        let tol = 1e-15;
        assert!((normpdf(5.0, 5.0, 2.0) - 0.19947114020071635).abs() < tol);
    }

    #[test]
    fn test_normpdf_invalid() {
        assert!(normpdf(0.0, 0.0, 0.0).is_nan());
        assert!(normpdf(0.0, 0.0, -1.0).is_nan());
    }
}
