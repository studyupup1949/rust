use crate::erfcinv;
use std::f64::consts::SQRT_2;

/// Inverse of the lognormal cumulative distribution function (CDF).
///
/// Given a probability `p`, a mean `mu`, and a standard deviation `sigma`
/// of the associated normal distribution, this function returns the
/// value `x` such that the probability of a lognormal random variable
/// being less than or equal to `x` is `p`.
///
/// # Mathematical Definition
/// The inverse lognormal CDF is derived from the inverse normal CDF.
/// If `Y` is a normal random variable with mean `mu` and standard deviation `sigma`,
/// then `X = exp(Y)` is a lognormal random variable.
///
/// The inverse CDF `x` for a given probability `p` is:
/// <math display="block"><mi>x</mi><mo>=</mo><msup><mi>e</mi><mrow><mi>μ</mi><mo>+</mo><mi>σ</mi><msup><mi>Φ</mi><mrow><mo>-</mo><mn>1</mn></mrow></msup><mo>(</mo><mi>p</mi><mo>)</mo></mrow></msup></math>
/// where <math><msup><mi>Φ</mi><mrow><mo>-</mo><mn>1</mn></mrow></msup><mo>(</mo><mi>p</mi><mo>)</mo></math> is the inverse of the standard normal CDF.
///
/// This can also be expressed using the inverse complementary error function:
/// <math display="block"><mi>x</mi><mo>=</mo><msup><mi>e</mi><mrow><mi>μ</mi><mo>-</mo><mi>σ</mi><msqrt><mn>2</mn></msqrt><msup><mi>erfc</mi><mrow><mo>-</mo><mn>1</mn></mrow></msup><mo>(</mo><mn>2</mn><mi>p</mi><mo>)</mo></mrow></msup></math>
///
/// # Domain
/// - `0 <= p <= 1`
/// - `sigma > 0`
/// - Returns `NaN` if `p` is out of range or `sigma` is non-positive, or any input is `NaN`.
///
/// # Examples
/// ```
/// use abax::logninv;
///
/// // Median of a standard lognormal distribution (mu=0, sigma=1) is 1.0
/// assert!((logninv(0.5, 0.0, 1.0) - 1.0).abs() < 1e-15);
///
/// // Approximately 68% of values are between exp(mu - sigma) and exp(mu + sigma)
/// // For mu=0, sigma=1, this is between exp(-1) and exp(1)
/// assert!((logninv(0.158655253931457, 0.0, 1.0) - (-1.0f64).exp()).abs() < 1e-15);
/// assert!((logninv(0.8413447460685429, 0.0, 1.0) - (1.0f64).exp()).abs() < 1e-15);
/// ```
pub fn logninv(p: f64, mu: f64, sigma: f64) -> f64 {
    if p.is_nan() || mu.is_nan() || sigma.is_nan() || !(0.0..=1.0).contains(&p) || sigma <= 0.0 {
        return f64::NAN;
    }

    if p == 0.0 {
        return 0.0;
    }
    if p == 1.0 {
        return f64::INFINITY;
    }

    // The MATLAB code uses: logx0 = -sqrt(2).*erfcinv(2*p);
    // This `logx0` is equivalent to the z-score (inverse standard normal CDF).
    // So, z = -SQRT_2 * erfcinv(2.0 * p);
    // Then x = exp(sigma * z + mu);
    let z = -SQRT_2 * erfcinv(2.0 * p);

    (sigma * z + mu).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-14;

    #[test]
    fn test_logninv_standard() {
        // Median of standard lognormal (mu=0, sigma=1) is exp(0) = 1.0
        assert!((logninv(0.5, 0.0, 1.0) - 1.0).abs() < TOL);

        // Values corresponding to +/- 1 standard deviation in the normal distribution
        // For normal(0,1), CDF(1) approx 0.84134, CDF(-1) approx 0.15866
        // So logninv(0.84134, 0, 1) should be exp(1)
        // And logninv(0.15866, 0, 1) should be exp(-1)
        assert!((logninv(0.8413447460685429, 0.0, 1.0) - (1.0f64).exp()).abs() < TOL);
        assert!((logninv(0.158655253931457, 0.0, 1.0) - (-1.0f64).exp()).abs() < TOL);
    }

    #[test]
    fn test_logninv_varied_params() {
        // mu = 1.0, sigma = 0.5
        // Median should be exp(mu) = exp(1.0)
        assert!((logninv(0.5, 1.0, 0.5) - (1.0f64).exp()).abs() < TOL);

        // Test with different mu and sigma
        let p_val = 0.9;
        let mu_val = 2.0;
        let sigma_val = 0.7;
        // Reference value from WolframAlpha: LognormalInverseCDF[0.9, 2, 0.7] approx 12.3396
        assert!((logninv(p_val, mu_val, sigma_val) - 1.812126473436266e1).abs() < TOL);
    }

    #[test]
    fn test_logninv_boundaries_and_invalid() {
        assert_eq!(logninv(0.0, 0.0, 1.0), 0.0);
        assert_eq!(logninv(1.0, 0.0, 1.0), f64::INFINITY);
        assert!(logninv(-0.1, 0.0, 1.0).is_nan());
        assert!(logninv(1.1, 0.0, 1.0).is_nan());
        assert!(logninv(0.5, 0.0, -1.0).is_nan());
        assert!(logninv(0.5, 0.0, 0.0).is_nan());
        assert!(logninv(f64::NAN, 0.0, 1.0).is_nan());
        assert!(logninv(0.5, f64::NAN, 1.0).is_nan());
        assert!(logninv(0.5, 0.0, f64::NAN).is_nan());
    }
}