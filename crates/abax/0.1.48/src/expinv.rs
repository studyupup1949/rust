/// Inverse of the exponential cumulative distribution function (quantile function).
///
/// Given a probability `p` and a mean (scale) parameter `mu`, this function returns
/// the value `x` such that the probability of an exponential random variable being
/// less than or equal to `x` is `p`.
///
/// # Mathematical Definition
/// The quantile function is the inverse of the CDF <math><mi>F</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>μ</mi><mo>)</mo><mo>=</mo><mn>1</mn><mo>-</mo><msup><mi>e</mi><mrow><mo>-</mo><mi>x</mi><mo>/</mo><mi>μ</mi></mrow></msup></math>:
/// <math display="block">
///   <mi>x</mi>
///   <mo>=</mo>
///   <mo>-</mo>
///   <mi>μ</mi>
///   <mi>ln</mi><mo>(</mo><mn>1</mn><mo>-</mo><mi>p</mi><mo>)</mo>
/// </math>
///
/// # Parameters
/// * `p` (`f64`) - The probability for which to find the quantile.
///   **Domain:** <math><mn>0</mn><mo>≤</mo><mi>p</mi><mo>≤</mo><mn>1</mn></math>.
/// * `mu` (`f64`) - The mean (scale) parameter of the distribution.
///   **Valid range:** <math><mi>μ</mi><mo>></mo><mn>0</mn></math>.
///
/// # Returns
/// Returns the quantile at `p` as a `f64`:
/// * Returns `0.0` if `p = 0.0`.
/// * Returns `f64::INFINITY` if `p = 1.0`.
/// * Returns `f64::NAN` if `p` is outside `[0.0, 1.0]`, or `mu <= 0.0`, or any input is `NaN`.
///
/// # Examples
/// ```
/// use abax::expinv;
///
/// // Median of exponential distribution with mean 2.0: 2.0 * ln(2)
/// let x = expinv(0.5, 2.0);
/// assert!((x - 2.0 * 2.0f64.ln()).abs() < 1e-15);
///
/// // Inverse of CDF(1.0, 2.0) should be 1.0
/// use abax::expcdf;
/// let p = expcdf(1.0, 2.0, false);
/// let x_inv = expinv(p, 2.0);
/// assert!((x_inv - 1.0).abs() < 1e-15);
/// ```
pub fn expinv(p: f64, mu: f64) -> f64 {
    if mu.is_nan() || mu <= 0.0 {
        return f64::NAN;
    }

    let q: f64 = if 0.0 < p && p < 1.0 {
        if p < 0.125 { -f64::ln_1p(-p) } else { -f64::ln(1.0 - p) }
    } else if p == 1.0 {
        f64::INFINITY
    } else if p < 0.0 || 1.0 < p || p.is_nan() {
        f64::NAN
    } else {
        0.0
    };

    return mu * q;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expcdf;

    #[test]
    fn test_expinv_standard() {
        let mu = 2.0;
        let p = 0.5;
        let expected = mu * 2.0f64.ln();
        assert!((expinv(p, mu) - expected).abs() < 1e-15);
    }

    #[test]
    fn test_expinv_roundtrip() {
        let mu = 1.5;
        let values = [0.1, 0.5, 0.9, 0.99];
        for &p in &values {
            let x = expinv(p, mu);
            let p_back = expcdf(x, mu, false);
            assert!((p - p_back).abs() < 1e-15);
        }
    }

    #[test]
    fn test_expinv_boundaries() {
        let mu = 2.5;
        assert_eq!(expinv(0.0, mu), 0.0);
        assert_eq!(expinv(1.0, mu), f64::INFINITY);
        assert!(expinv(-0.1, mu).is_nan());
        assert!(expinv(1.1, mu).is_nan());
        assert!(expinv(0.5, 0.0).is_nan());
        assert!(expinv(0.5, -1.0).is_nan());
        assert!(expinv(0.5, f64::NAN).is_nan());
    }
}
