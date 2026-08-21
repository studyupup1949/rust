use crate::erfcinv;
use std::f64::consts::SQRT_2;

/// Inverse of the normal cumulative distribution function (quantile function).
///
/// Given a probability `p`, a mean `mu`, and a standard deviation `sigma`,
/// this function returns the value `x` such that the probability of a 
/// normal random variable being less than or equal to `x` is `p`.
///
/// # Mathematical Definition
/// The quantile function is expressed via the inverse error function. For <math><mi>p</mi><mo>∈</mo><mo>(</mo><mn>0</mn><mo>,</mo><mn>1</mn><mo>)</mo></math>:
/// <math display="block"><mi>x</mi><mo>=</mo><mi>μ</mi><mo>+</mo><mi>σ</mi><msqrt><mn>2</mn></msqrt><msup><mi>erf</mi><mrow><mo>-</mo><mn>1</mn></mrow></msup><mo>(</mo><mn>2</mn><mi>p</mi><mo>-</mo><mn>1</mn><mo>)</mo></math>
///
/// For a standard normal distribution (<math><mi>μ</mi><mo>=</mo><mn>0</mn><mo>,</mo><mi>σ</mi><mo>=</mo><mn>1</mn></math>), this is the Probit function:
/// <math display="block"><mi>z</mi><mo>=</mo><msup><mi>Φ</mi><mrow><mo>-</mo><mn>1</mn></mrow></msup><mo>(</mo><mi>p</mi><mo>)</mo><mo>=</mo><mo>-</mo><msqrt><mn>2</mn></msqrt><msup><mi>erfc</mi><mrow><mo>-</mo><mn>1</mn></mrow></msup><mo>(</mo><mn>2</mn><mi>p</mi><mo>)</mo></math>
///
/// # Domain
/// - `0 <= p <= 1`
/// - `sigma > 0`
/// - Returns `NaN` if `p` is out of range or `sigma` is non-positive.
///
/// # Examples
/// ```
/// use abax::norminv;
///
/// // Median of standard normal is 0
/// assert!((norminv(0.5, 0.0, 1.0) - 0.0).abs() < 1e-15);
/// // ~1 standard deviation for p=0.8413
/// assert!((norminv(0.8413447460685429, 0.0, 1.0) - 1.0).abs() < 1e-15);
/// ```
pub fn norminv(p: f64, mu: f64, sigma: f64) -> f64 {
    if p.is_nan() || mu.is_nan() || sigma.is_nan() || !(0.0..=1.0).contains(&p) || sigma <= 0.0 {
        return f64::NAN;
    }

    if p == 0.0 {
        return f64::NEG_INFINITY;
    }
    if p == 1.0 {
        return f64::INFINITY;
    }

    // Calculate the standard normal quantile (z-score)
    // Relationship: normcdf(z) = 0.5 * erfc(-z / sqrt(2))
    // Setting p = 0.5 * erfc(-z / sqrt(2)) leads to:
    let z = -SQRT_2 * erfcinv(2.0 * p);

    mu + sigma * z
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_norminv_standard() {
        let tol = 1e-14;
        assert!((norminv(0.5, 0.0, 1.0) - 0.0).abs() < tol);
        assert!((norminv(0.975, 0.0, 1.0) - 1.959963984540054).abs() < tol);
        assert!((norminv(0.158655253931457, 0.0, 1.0) + 1.0).abs() < tol);
    }

    #[test]
    fn test_norminv_boundaries_and_invalid() {
        assert_eq!(norminv(0.0, 0.0, 1.0), f64::NEG_INFINITY);
        assert_eq!(norminv(1.0, 0.0, 1.0), f64::INFINITY);
        assert!(norminv(-0.1, 0.0, 1.0).is_nan());
        assert!(norminv(1.1, 0.0, 1.0).is_nan());
        assert!(norminv(0.5, 0.0, -1.0).is_nan());
        assert!(norminv(0.5, 0.0, 0.0).is_nan());
    }
}
