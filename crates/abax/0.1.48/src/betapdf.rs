use crate::betaln;
/// Beta probability density function (PDF).
///
/// Returns the probability density at `x` for the Beta distribution
/// with shape parameters `a` and `b`.
///
/// # Mathematical Definition
/// For a Beta distribution with shape parameters <math><mi>a</mi></math> and <math><mi>b</mi></math>:
/// <math display="block">
///   <mi>f</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>a</mi><mo>,</mo><mi>b</mi><mo>)</mo>
///   <mo>=</mo>
///   <mfrac>
///     <mrow>
///       <msup><mi>x</mi><mrow><mi>a</mi><mo>-</mo><mn>1</mn></mrow></msup>
///       <msup><mrow><mo>(</mo><mn>1</mn><mo>-</mo><mi>x</mi><mo>)</mo></mrow><mrow><mi>b</mi><mo>-</mo><mn>1</mn></mrow></msup>
///     </mrow>
///     <mrow>
///       <mi>B</mi><mo>(</mo><mi>a</mi><mo>,</mo><mi>b</mi><mo>)</mo>
///     </mrow>
///   </mfrac>
/// </math>
/// where <math><mi>B</mi><mo>(</mo><mi>a</mi><mo>,</mo><mi>b</mi><mo>)</mo></math> is the Beta function.
///
/// # Domain
/// - <math><mn>0</mn><mo>≤</mo><mi>x</mi><mo>≤</mo><mn>1</mn></math> (Returns 0 outside this range).
/// - <math><mi>a</mi><mo>&gt;</mo><mn>0</mn></math>, <math><mi>b</mi><mo>&gt;</mo><mn>0</mn></math>.
/// - Returns `NaN` if `a <= 0` or `b <= 0`.
///
/// # Examples
/// ```
/// use abax::betapdf;
///
/// // a=1, b=1 is a uniform distribution on [0, 1], so density is 1.0 everywhere.
/// assert_eq!(betapdf(0.5, 1.0, 1.0), 1.0);
///
/// // Known value for Beta(2, 3) at x=0.2
/// let pdf = betapdf(0.2, 2.0, 3.0);
/// assert!((pdf - 1.536).abs() < 1e-12);
/// ```
pub fn betapdf(x: f64, a: f64, b: f64) -> f64 {
    if a == 1.0 && x == 0.0 {
        return b;
    }
    if b == 1.0 && x == 1.0 {
        return a;
    }
    if (a < 1.0 && x == 0.0) || (b < 1.0 && x == 1.0) {
        return f64::INFINITY;
    }
    if a <= 0.0 || b <= 0.0 || a.is_nan() || b.is_nan() || x.is_nan() {
        return f64::NAN;
    }

    if a > 0.0 && b > 0.0 && x > 0.0 && x < 1.0 {
        let smallx = x < 0.1;
        let loga = (a - 1.0) * x.ln();
        let logb = if smallx {
            (b - 1.0) *  f64::ln_1p(-x)
        } else {
            (b - 1.0) * f64::ln(1.0 - x)
        };
        return f64::exp(loga + logb - betaln(a, b));
    }

    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_betapdf_uniform() {
        // a=1, b=1 is Uniform(0,1)
        assert_eq!(betapdf(0.1, 1.0, 1.0), 1.0);
        assert_eq!(betapdf(0.5, 1.0, 1.0), 1.0);
        assert_eq!(betapdf(0.9, 1.0, 1.0), 1.0);
    }

    #[test]
    fn test_betapdf_known_values() {
        let tol = 1e-14;
        // f(0.2; 2, 3) = 0.2 * (0.8)^2 / B(2, 3)
        // B(2, 3) = 1/12
        // f = 0.2 * 0.64 * 12 = 1.536
        assert!((betapdf(0.2, 2.0, 3.0) - 1.536).abs() < tol);

        // f(0.5; 0.5, 0.5) = 2 / pi
        assert!((betapdf(0.5, 0.5, 0.5) - 2.0 / std::f64::consts::PI).abs() < tol);
    }

    #[test]
    fn test_betapdf_symmetry() {
        let x = 0.3;
        let a = 2.5;
        let b = 1.5;
        assert!((betapdf(x, a, b) - betapdf(1.0 - x, b, a)).abs() < 1e-14);
    }

    #[test]
    fn test_betapdf_poles_and_boundaries() {
        // a < 1, x=0 -> inf
        assert_eq!(betapdf(0.0, 0.5, 2.0), f64::INFINITY);
        // b < 1, x=1 -> inf
        assert_eq!(betapdf(1.0, 2.0, 0.5), f64::INFINITY);

        // a = 1, x=0 -> b
        assert_eq!(betapdf(0.0, 1.0, 5.0), 5.0);
        // b = 1, x=1 -> a
        assert_eq!(betapdf(1.0, 5.0, 1.0), 5.0);

        // Outside [0, 1]
        assert_eq!(betapdf(-0.1, 2.0, 2.0), 0.0);
        assert_eq!(betapdf(1.1, 2.0, 2.0), 0.0);
    }

    #[test]
    fn test_betapdf_invalid_params() {
        assert!(betapdf(0.5, 0.0, 1.0).is_nan());
        assert!(betapdf(0.5, -1.0, 1.0).is_nan());
        assert!(betapdf(0.5, 1.0, 0.0).is_nan());
        assert!(betapdf(0.5, 1.0, -1.0).is_nan());
        assert!(betapdf(f64::NAN, 1.0, 1.0).is_nan());
        assert!(betapdf(0.5, f64::NAN, 1.0).is_nan());
    }
}