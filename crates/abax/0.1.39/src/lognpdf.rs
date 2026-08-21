use crate::consts::SQRT_2PI;

/// Lognormal probability density function (PDF).
///
/// Given a value `x`, a mean `mu`, and a standard deviation `sigma`
/// of the associated normal distribution, this function returns the
/// probability density at `x`.
///
/// # Mathematical Definition
/// For a lognormal distribution with parameters <math><mi>μ</mi></math> and <math><mi>σ</mi></math>:
/// <math display="block"><mi>f</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>μ</mi><mo>,</mo><mi>σ</mi><mo>)</mo><mo>=</mo><mfrac><mn>1</mn><mrow><mi>x</mi><mi>σ</mi><msqrt><mn>2</mn><mi>π</mi></msqrt></mrow></mfrac><msup><mi>e</mi><mrow><mo>-</mo><mfrac><mrow><mo>(</mo><mi>ln</mi><mi>x</mi><mo>-</mo><mi>μ</mi><msup><mo>)</mo><mn>2</mn></msup></mrow><mrow><mn>2</mn><msup><mi>σ</mi><mn>2</mn></msup></mrow></mfrac></mrow></msup></math>
///
/// # Domain
/// - `x > 0`
/// - `sigma > 0`
/// - Returns `NaN` if `x` is non-positive, `sigma` is non-positive, or any input is `NaN`.
///
/// # Examples
/// ```
/// use abax::lognpdf;
///
/// // For a standard lognormal distribution (mu=0, sigma=1)
/// assert!((lognpdf(1.0, 0.0, 1.0) - 3.989422804014327e-01).abs() < 1e-15);
/// assert!((lognpdf(2.0, 0.0, 1.0) - 1.568740192789811e-01).abs() < 1e-15);
/// ```
pub fn lognpdf(x: f64, mu: f64, sigma: f64) -> f64 {
    if x.is_nan() || mu.is_nan() || sigma.is_nan() || sigma <= 0.0 {
        return f64::NAN;
    }

    if x <= 0.0 {
        return 0.0;
    }

    let log_x = x.ln();
    let z = (log_x - mu) / sigma;
    (-0.5 * z * z).exp() / (x * sigma * SQRT_2PI)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lognpdf_standard() {
        let tol = 1e-15;
        assert!((lognpdf(1.0, 0.0, 1.0) - 3.9894228040143268e-1).abs() < tol);
        assert!((lognpdf(2.0, 0.0, 1.0) - 1.5687401927898109e-1).abs() < tol);
        assert!((lognpdf(0.5, 0.0, 1.0) - 6.2749607711592437e-1).abs() < tol);
    }

    #[test]
    fn test_lognpdf_varied_params() {
        let tol = 1e-15;
        // Test with mu = 1, sigma = 0.5
        assert!((lognpdf(2.718281828459045, 1.0, 0.5) - 2.935253263474799e-01).abs() < tol); // x = e^mu
        assert!((lognpdf(1.0, 1.0, 0.5) - 1.079819330263761e-01).abs() < tol); // x=1, ln(x)=0, (0-1)/0.5 = -2, exp(-0.5*4) = exp(-2)
    }

    #[test]
    fn test_lognpdf_edge_cases() {
        assert!(lognpdf(f64::NAN, 0.0, 1.0).is_nan());
        assert!(lognpdf(1.0, f64::NAN, 1.0).is_nan());
        assert!(lognpdf(1.0, 0.0, f64::NAN).is_nan());
        assert!(lognpdf(1.0, 0.0, 0.0).is_nan());
        assert!(lognpdf(1.0, 0.0, -1.0).is_nan());
        assert_eq!(lognpdf(0.0, 0.0, 1.0), 0.0);
        assert_eq!(lognpdf(-1.0, 0.0, 1.0), 0.0);
    }
}