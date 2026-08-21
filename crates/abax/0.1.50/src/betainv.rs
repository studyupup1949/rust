use crate::betaincinv;

/// Inverse of the beta cumulative distribution function (CDF).
///
/// Given a probability `p`, and shape parameters `a` and `b`,
/// this function returns the value `x` such that the probability of a Beta
/// random variable being less than or equal to `x` is `p`.
///
/// # Mathematical Definition
/// The function finds <math><mi>x</mi></math> such that:
/// <math display="block">
///   <msub><mi>I</mi><mi>x</mi></msub><mo>(</mo><mi>a</mi><mo>,</mo><mi>b</mi><mo>)</mo>
///   <mo>=</mo>
///   <mi>p</mi>
/// </math>
/// where <math><msub><mi>I</mi><mi>x</mi></msub><mo>(</mo><mi>a</mi><mo>,</mo><mi>b</mi><mo>)</mo></math> is the regularized incomplete beta function.
///
/// # Domain
/// - <math><mn>0</mn><mo>≤</mo><mi>p</mi><mo>≤</mo><mn>1</mn></math>
/// - <math><mi>a</mi><mo>&gt;</mo><mn>0</mn></math>, <math><mi>b</mi><mo>&gt;</mo><mn>0</mn></math>.
/// - Returns `NaN` if `p` is out of range, `a <= 0`, `b <= 0`, or any input is `NaN`.
///
/// # Examples
/// ```
/// use abax::betainv;
///
/// // For a=1, b=1, it's a uniform distribution on, so inv(p) = p.
/// assert_eq!(betainv(0.5, 1.0, 1.0), 0.5);
///
/// // Known value for Beta(1, 3): inv(0.488) = 0.2
/// let x = betainv(0.488, 1.0, 3.0);
/// assert!((x - 0.2).abs() < 1e-12);
/// ```
pub fn betainv(p: f64, a: f64, b: f64) -> f64 {
    if p.is_nan() || p < 0.0 || p > 1.0 {
        return f64::NAN;
    }
    if a.is_nan() || a <= 0.0 || a.is_infinite() {
        return f64::NAN;
    }
    if b.is_nan() || b <= 0.0 || b.is_infinite() {
        return f64::NAN;
    }

    let q = betaincinv(p, a, b, true);
    return q;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::betacdf;

    #[test]
    fn test_betainv_uniform_identity() {
        // For a=1, b=1, Beta distribution is Uniform(0,1), so inv(p) = p
        assert!((betainv(0.1, 1.0, 1.0) - 0.1).abs() < 1e-15);
        assert!((betainv(0.5, 1.0, 1.0) - 0.5).abs() < 1e-15);
        assert!((betainv(0.9, 1.0, 1.0) - 0.9).abs() < 1e-15);
    }

    #[test]
    fn test_betainv_known_values() {
        let tol = 1e-12;
        // For Beta(1, 3), CDF is 1 - (1-x)^3.
        // If p = 0.488, then 0.488 = 1 - (1-x)^3 => (1-x)^3 = 0.512 => 1-x = 0.8 => x = 0.2
        assert!((betainv(0.488, 1.0, 3.0) - 0.2).abs() < tol);

        // For Beta(2, 2), median is 0.5
        assert!((betainv(0.5, 2.0, 2.0) - 0.5).abs() < tol);
    }

    #[test]
    fn test_betainv_roundtrip() {
        let a = 2.5;
        let b = 1.5;
        let probabilities = [0.001, 0.1, 0.5, 0.9, 0.999];
        for &p in &probabilities {
            let x = betainv(p, a, b);
            let p_back = betacdf(x, a, b, false);
            assert!((p - p_back).abs() < 1e-10, "p: {}, x: {}, p_back: {}", p, x, p_back);
        }
    }

    #[test]
    fn test_betainv_boundaries() {
        let a = 2.0;
        let b = 3.0;
        assert_eq!(betainv(0.0, a, b), 0.0);
        assert_eq!(betainv(1.0, a, b), 1.0);
        assert!(betainv(0.5, 0.0, b).is_nan());
        assert!(betainv(0.5, a, 0.0).is_nan());
    }
}
