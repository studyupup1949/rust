use crate::betainc;

/// Beta cumulative distribution function (CDF).
///
/// Returns the probability that a Beta random variable with shape parameters `a`
/// and `b` is less than or equal to `x`.
///
/// # Mathematical Definition
/// For a Beta distribution with shape parameters <math><mi>a</mi></math> and <math><mi>b</mi></math>:
/// - Lower tail (`upper = false`):
/// <math display="block">
///   <mi>F</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>a</mi><mo>,</mo><mi>b</mi><mo>)</mo>
///   <mo>=</mo>
///   <msub><mi>I</mi><mi>x</mi></msub><mo>(</mo><mi>a</mi><mo>,</mo><mi>b</mi><mo>)</mo>
///   <mo>=</mo>
///   <mfrac><mn>1</mn><mrow><mi>B</mi><mo>(</mo><mi>a</mi><mo>,</mo><mi>b</mi><mo>)</mo></mrow></mfrac>
///   <msubsup><mo>∫</mo><mn>0</mn><mi>x</mi></msubsup>
///   <msup><mi>t</mi><mrow><mi>a</mi><mo>-</mo><mn>1</mn></mrow></msup><msup><mrow><mo>(</mo><mn>1</mn><mo>-</mo><mi>t</mi><mo>)</mo></mrow><mrow><mi>b</mi><mo>-</mo><mn>1</mn></mrow></msup><mi>dt</mi>
/// </math>
/// - Upper tail (`upper = true`):
/// <math display="block">
///   <mn>1</mn><mo>-</mo><mi>F</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>a</mi><mo>,</mo><mi>b</mi><mo>)</mo>
///   <mo>=</mo>
///   <mn>1</mn><mo>-</mo><msub><mi>I</mi><mi>x</mi></msub><mo>(</mo><mi>a</mi><mo>,</mo><mi>b</mi><mo>)</mo>
/// </math>
/// where <math><msub><mi>I</mi><mi>x</mi></msub><mo>(</mo><mi>a</mi><mo>,</mo><mi>b</mi><mo>)</mo></math> is the regularized incomplete beta function.
///
/// # Domain
/// - <math><mn>0</mn><mo>≤</mo><mi>x</mi><mo>≤</mo><mn>1</mn></math> (Returns 0/1 for lower/upper tail if <math><mi>x</mi><mo>&lt;</mo><mn>0</mn></math>; 1/0 if <math><mi>x</mi><mo>&gt;</mo><mn>1</mn></math>).
/// - <math><mi>a</mi><mo>&gt;</mo><mn>0</mn></math>, <math><mi>b</mi><mo>&gt;</mo><mn>0</mn></math>.
/// - Returns `NaN` if <math><mi>a</mi><mo>≤</mo><mn>0</mn></math> or <math><mi>b</mi><mo>≤</mo><mn>0</mn></math>.
///
/// # Examples
/// ```
/// use abax::betacdf;
///
/// // a=1, b=1 is a uniform distribution on [0, 1]
/// assert_eq!(betacdf(0.5, 1.0, 1.0, false), 0.5);
///
/// // Known value for Beta(1, 3) at x=0.2: 1 - (1-0.2)^3 = 0.488
/// let p = betacdf(0.2, 1.0, 3.0, false);
/// assert!((p - 0.488).abs() < 1e-12);
/// ```
pub fn betacdf(x: f64, a: f64, b: f64, upper: bool) -> f64 {
    match (0.0 < a && a < f64::INFINITY) && (0.0 < b && b < f64::INFINITY) {
        true => (),
        false => return f64::NAN,
    }
    
    match upper {
        true => {
            if x <= 0.0 { return 1.0; }
            if x >= 1.0 { return 0.0; }
        },
        false => {
            if x < 0.0 { return 0.0;}
            if x > 1.0 { return 1.0;}
        },
    }

    let p = betainc(x, a, b, !upper);
    return p;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_betacdf_uniform() {
        // a=1, b=1 is Uniform(0,1)
        assert_eq!(betacdf(0.1, 1.0, 1.0, false), 0.1);
        assert_eq!(betacdf(0.5, 1.0, 1.0, false), 0.5);
        assert_eq!(betacdf(0.9, 1.0, 1.0, false), 0.9);
    }

    #[test]
    fn test_betacdf_known_values() {
        let tol = 1e-14;
        // F(x; 1, b) = 1 - (1-x)^b
        assert!((betacdf(0.2, 1.0, 3.0, false) - 0.488).abs() < tol);
        // F(0.5; 2, 2) = 0.5
        assert_eq!(betacdf(0.5, 2.0, 2.0, false), 0.5);
    }

    #[test]
    fn test_betacdf_tail_consistency() {
        let x = 0.3;
        let a = 2.5;
        let b = 1.5;
        let p = betacdf(x, a, b, false);
        let q = betacdf(x, a, b, true);
        assert!((p + q - 1.0).abs() < 1e-15);
    }

    #[test]
    fn test_betacdf_symmetry() {
        let x = 0.4;
        let a = 3.0;
        let b = 2.0;
        // F(x; a, b) = 1 - F(1-x; b, a)
        assert!((betacdf(x, a, b, false) - betacdf(1.0 - x, b, a, true)).abs() < 1e-15);
    }

    #[test]
    fn test_betacdf_boundaries() {
        assert_eq!(betacdf(-0.1, 2.0, 2.0, false), 0.0);
        assert_eq!(betacdf(1.1, 2.0, 2.0, false), 1.0);
        assert_eq!(betacdf(-0.1, 2.0, 2.0, true), 1.0);
        assert_eq!(betacdf(1.1, 2.0, 2.0, true), 0.0);
    }

    #[test]
    fn test_betacdf_invalid_params() {
        assert!(betacdf(0.5, 0.0, 1.0, false).is_nan());
        assert!(betacdf(0.5, 1.0, -1.0, false).is_nan());
        assert!(betacdf(0.5, f64::INFINITY, 1.0, false).is_nan());
    }
}