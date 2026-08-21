use crate::erfc;
use std::f64::consts::SQRT_2;

/// Normal cumulative distribution function (CDF).
///
/// Given a value `x`, a mean `mu`, and a standard deviation `sigma`,
/// this function returns the probability that a normal random variable
/// is less than or equal to `x`.
///
/// # Mathematical Definition
/// For a normal distribution with mean <math><mi>μ</mi></math> and standard deviation <math><mi>σ</mi></math>:
/// - Lower tail (`upper = false`): <math><mi>Φ</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>μ</mi><mo>,</mo><mi>σ</mi><mo>)</mo><mo>=</mo><mfrac><mn>1</mn><mn>2</mn></mfrac><mi>erfc</mi><mo>(</mo><mo>-</mo><mfrac><mrow><mi>x</mi><mo>-</mo><mi>μ</mi></mrow><mrow><mi>σ</mi><msqrt><mn>2</mn></msqrt></mrow></mfrac><mo>)</mo></math>
/// - Upper tail (`upper = true`): <math><mn>1</mn><mo>-</mo><mi>Φ</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>μ</mi><mo>,</mo><mi>σ</mi><mo>)</mo><mo>=</mo><mfrac><mn>1</mn><mn>2</mn></mfrac><mi>erfc</mi><mo>(</mo><mfrac><mrow><mi>x</mi><mo>-</mo><mi>μ</mi></mrow><mrow><mi>σ</mi><msqrt><mn>2</mn></msqrt></mrow></mfrac><mo>)</mo></math>
///
/// # Domain
/// - `sigma >= 0`
/// - Returns `NaN` if `sigma` is negative or any input is `NaN`.
/// - If `sigma` is 0, the distribution is treated as a Dirac delta concentrated at `mu`.
///   - Returns 0 (lower) / 1 (upper) if `x < mu`.
///   - Returns 1 (lower) / 0 (upper) if `x >= mu`.
///
/// # Examples
/// ```
/// use abax::normcdf;
///
/// // Standard normal median is 0.5
/// assert!((normcdf(0.0, 0.0, 1.0, false) - 0.5).abs() < 1e-15);
/// // One sigma upper bound
/// assert!((normcdf(1.0, 0.0, 1.0, false) - 0.8413447460685429).abs() < 1e-15);
/// // Upper tail one sigma
/// assert!((normcdf(1.0, 0.0, 1.0, true) - 0.15865525393145705).abs() < 1e-15);
/// ```
pub fn normcdf(x: f64, mu: f64, sigma: f64, upper: bool) -> f64 {
    if x.is_nan() || mu.is_nan() || sigma.is_nan() || sigma < 0.0 {
        return f64::NAN;
    }
 
    if sigma == 0.0 {
        return if upper {
            if x < mu { 1.0 } else { 0.0 }
        } else {
            if x < mu { 0.0 } else { 1.0 }
        };
    }
 
    let z = (x - mu) / sigma;
    if upper {
        0.5 * erfc(z / SQRT_2)
    } else {
        0.5 * erfc(-z / SQRT_2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normcdf_standard() {
        let tol = 1e-14;
        assert!((normcdf(0.0, 0.0, 1.0, false) - 0.5).abs() < tol);
        assert!((normcdf(1.959963984540054, 0.0, 1.0, false) - 0.975).abs() < tol);
        assert!((normcdf(-1.959963984540054, 0.0, 1.0, false) - 0.025).abs() < tol);
    }
 
    #[test]
    fn test_normcdf_upper() {
        let tol = 1e-14;
        assert!((normcdf(0.0, 0.0, 1.0, true) - 0.5).abs() < tol);
        assert!((normcdf(1.959963984540054, 0.0, 1.0, true) - 0.025).abs() < tol);
        assert!((normcdf(-1.959963984540054, 0.0, 1.0, true) - 0.975).abs() < tol);
    }

    #[test]
    fn test_normcdf_zero_sigma() {
        assert_eq!(normcdf(5.0, 5.0, 0.0, false), 1.0);
        assert_eq!(normcdf(4.99, 5.0, 0.0, false), 0.0);
        assert_eq!(normcdf(4.99, 5.0, 0.0, true), 1.0);
    }

    #[test]
    fn test_normcdf_invalid() {
        assert!(normcdf(0.5, 0.0, -1.0, false).is_nan());
    }
}
