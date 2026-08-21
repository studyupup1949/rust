/// Exponential cumulative distribution function (CDF).
///
/// Returns the probability that an exponential random variable with mean (scale)
/// parameter `mu` is less than or equal to `x`.
///
/// # Mathematical Definition
/// For an exponential distribution with mean <math><mi>μ</mi></math>:
/// - Lower tail (`upper = false`):
/// <math display="block">
///   <mi>F</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>μ</mi><mo>)</mo>
///   <mo>=</mo>
///   <mn>1</mn>
///   <mo>-</mo>
///   <msup>
///     <mi>e</mi>
///     <mrow>
///       <mo>-</mo>
///       <mi>x</mi>
///       <mo>/</mo>
///       <mi>μ</mi>
///     </mrow>
///   </msup>
/// </math>
/// - Upper tail (`upper = true`):
/// <math display="block">
///   <mi>Q</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>μ</mi><mo>)</mo>
///   <mo>=</mo>
///   <msup>
///     <mi>e</mi>
///     <mrow>
///       <mo>-</mo>
///       <mi>x</mi>
///       <mo>/</mo>
///       <mi>μ</mi>
///     </mrow>
///   </msup>
/// </math>
///
/// # Parameters
/// * `x` (`f64`) - The value at which to evaluate the CDF.
/// * `mu` (`f64`) - The mean (scale) parameter of the distribution.
///   **Valid range:** <math><mi>μ</mi><mo>></mo><mn>0</mn></math>.
/// * `upper` (`bool`) - If `true`, returns the survival function (upper tail).
///
/// # Returns
/// Returns the cumulative probability at `x` as a `f64`:
/// * For <math><mi>x</mi><mo>≥</mo><mn>0</mn></math>: returns the tail probability.
/// * For <math><mi>x</mi><mo><</mo><mn>0</mn></math>: returns `0.0` for lower tail, `1.0` for upper tail.
/// * For <math><mi>μ</mi><mo>≤</mo><mn>0</mn></math>: returns `f64::NAN`.
///
/// # Examples
/// ```
/// use abax::expcdf;
/// // CDF at x = 1.0 with mean mu = 2.0
/// let p = expcdf(1.0, 2.0, false);
/// assert!((p - (1.0 - (-0.5f64).exp())).abs() < 1e-15);
///
/// // Survival function at x = 1.0
/// let q = expcdf(1.0, 2.0, true);
/// assert!((q - (-0.5f64).exp()).abs() < 1e-15);
/// ```
pub fn expcdf(x: f64, mu: f64, upper: bool) -> f64 {
    if mu <= 0.0 {
        return f64::NAN;
    }

    if x < 0.0 {
        return if upper { 1.0 } else { 0.0 };
    }

    let z = x / mu;
    if upper {
        f64::exp(-z)
    } else {
        -f64::exp_m1(-z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expcdf_standard_case() {
        let mu = 1.0;
        let x = 0.5;
        let p = expcdf(x, mu, false);
        let q = expcdf(x, mu, true);
        assert!((p - (1.0 - (-0.5f64).exp())).abs() < 1e-15);
        assert!((q - (-0.5f64).exp()).abs() < 1e-15);
        assert!((p + q - 1.0).abs() < 1e-15);
    }

    #[test]
    fn test_expcdf_boundaries() {
        let mu = 2.5;
        assert_eq!(expcdf(0.0, mu, false), 0.0);
        assert_eq!(expcdf(0.0, mu, true), 1.0);
        assert_eq!(expcdf(-1.0, mu, false), 0.0);
        assert_eq!(expcdf(-1.0, mu, true), 1.0);
    }

    #[test]
    fn test_expcdf_invalid_mu() {
        assert!(expcdf(1.0, 0.0, false).is_nan());
        assert!(expcdf(1.0, -1.0, false).is_nan());
    }
}
