use crate::gaminv;

/// Inverse of the Chi-squared cumulative distribution function (quantile function).
///
/// Given a probability <math><mi>p</mi></math> and degrees of freedom <math><mi>v</mi></math>,
/// this function returns the value <math><mi>x</mi></math> such that the probability of a
/// Chi-squared random variable being less than or equal to <math><mi>x</mi></math> is <math><mi>p</mi></math>.
///
/// # Mathematical Definition
/// The Chi-squared quantile function is the inverse of the CDF. It is related to the
/// inverse Gamma quantile function:
/// <math display="block"><mi>x</mi><mo>=</mo><msup><mi>F</mi><mrow><mo>&#x2212;</mo><mn>1</mn></mrow></msup><mo form="prefix" stretchy="false">(</mo><mi>p</mi><mo>|</mo><mi>&#x3bd;</mi><mo form="postfix" stretchy="false">)</mo><mo>=</mo><mo form="prefix" stretchy="false">{</mo><mi>x</mi><mo lspace="0.2222em" rspace="0.2222em">:</mo><mi>F</mi><mo form="prefix" stretchy="false">(</mo><mi>x</mi><mo>|</mo><mi>&#x3bd;</mi><mo form="postfix" stretchy="false">)</mo><mo>=</mo><mi>p</mi><mo form="postfix" stretchy="false">}</mo></math>
/// where
/// <math display="block"><mrow><mi>p</mi><mo>=</mo><mi>F</mi><mo form="prefix" stretchy="false">(</mo><mi>x</mi><mi>|</mi><mi>&#x3bd;</mi><mo form="postfix" stretchy="false">)</mo><mo>=</mo><msubsup><mo movablelimits="false">&#x222b;</mo><mn>0</mn><mi>x</mi></msubsup><mfrac><mrow><msup><mi>t</mi><mrow><mo form="prefix" stretchy="false">(</mo><mi>&#x3bd;</mi><mo>&#x2212;</mo><mn>2</mn><mo form="postfix" stretchy="false">)</mo><mo lspace="0em" rspace="0em">&#x2044;</mo><mn>2</mn></mrow></msup><msup><mi>e</mi><mrow><mo>&#x2212;</mo><mi>t</mi><mo lspace="0em" rspace="0em">&#x2044;</mo><mn>2</mn></mrow></msup></mrow><mrow><msup><mn>2</mn><mrow><mi>&#x3bd;</mi><mo lspace="0em" rspace="0em">&#x2044;</mo><mn>2</mn></mrow></msup><mi>&#x393;</mi><mo form="prefix" stretchy="false">(</mo><mi>&#x3bd;</mi><mo lspace="0em" rspace="0em">&#x2044;</mo><mn>2</mn><mo form="postfix" stretchy="false">)</mo></mrow></mfrac><mspace width="0.1667em"></mspace><mi>d</mi><mi>t</mi></mrow></math>
/// <math><mi>&#x3bd;</mi></math> is the degree of freedom, and <math><mrow><mi>&#x393;</mi><mo form="prefix" stretchy="false">(</mo><mo>&#xb7;</mo><mo form="postfix" stretchy="false">)</mo></mrow></math>
/// is the Gamma function. The result <math><mi>p</mi></math> is the probability that a single observation from
/// the chi-square distribution with <math><mi>&#x3bd;</mi></math> degrees of freedom fals in the interval <math><mo form="prefix" stretchy="false">[</mo><mn>0</mn><mo separator="true">,</mo><mi>x</mi><mo form="postfix" stretchy="false">]</mo></math>.
/// 
/// # Examples
/// ```
/// use abax::chi2inv;
///
/// // Median of Chi-squared(2) is 2 * ln(2)
/// let x = chi2inv(0.5, 2.0);
/// assert!((x - 2.0 * 2.0f64.ln()).abs() < 1e-15);
/// ```
pub fn chi2inv(p: f64, v: f64) -> f64 {
    gaminv(p, v/2.0, 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chi2cdf;

    #[test]
    fn test_chi2inv_v2_exponential() {
        // v = 2: F(x) = 1 - exp(-x/2) => x = -2 * ln(1-p)
        let p = 0.75;
        let x = chi2inv(p, 2.0);
        let expected = -2.0 * (1.0 - p).ln();
        assert!((x - expected).abs() < 1e-14);
    }

    #[test]
    fn test_chi2inv_roundtrip() {
        let v = 4.5;
        let probabilities = [0.01, 0.1, 0.5, 0.9, 0.99];
        for &p in &probabilities {
            let x = chi2inv(p, v);
            let p_back = chi2cdf(x, v, false);
            assert!((p - p_back).abs() < 1e-12);
        }
    }

    #[test]
    fn test_chi2inv_boundaries() {
        let v = 2.0;
        assert_eq!(chi2inv(0.0, v), 0.0);
        assert_eq!(chi2inv(1.0, v), f64::INFINITY);
    }

    #[test]
    fn test_chi2inv_invalid_params() {
        assert!(chi2inv(0.5, 0.0) == 0.0);
        assert!(chi2inv(0.5, -1.0).is_nan());
        assert!(chi2inv(-0.1, 2.0).is_nan());
        assert!(chi2inv(1.1, 2.0).is_nan());
        assert!(chi2inv(f64::NAN, 2.0).is_nan());
    }
}