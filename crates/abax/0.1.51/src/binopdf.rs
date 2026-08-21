use crate::{stirlerr::stirlerr, binodeviance::binodeviance, gammaln};

/// Binomial probability density function (PDF).
///
/// Returns the probability of obtaining exactly `x` successes in `n` independent
/// Bernoulli trials, each with success probability `p`.
/// While the binomial distribution is discrete (often called the Probability Mass Function),
/// this function accepts `f64` for compatibility with other distribution functions.
///
/// # Mathematical Definition
/// <math display="block">
///   <mi>f</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>n</mi><mo>,</mo><mi>p</mi><mo>)</mo>
///   <mo>=</mo>
///   <mrow><mo>(</mo><mfrac linethickness="0"><mi>n</mi><mi>x</mi></mfrac><mo>)</mo></mrow>
///   <msup><mi>p</mi><mi>x</mi></msup>
///   <msup><mrow><mo>(</mo><mn>1</mn><mo>-</mo><mi>p</mi><mo>)</mo></mrow><mrow><mi>n</mi><mo>-</mo><mi>x</mi></mrow></msup>
/// </math>
/// for <math><mi>x</mi><mo>∈</mo><mo>{</mo><mn>0</mn><mo>,</mo><mn>1</mn><mo>,</mo><mo>...</mo><mo>,</mo><mi>n</mi><mo>}</mo></math>.
///
/// # Examples
/// ```
/// use abax::binopdf;
///
/// // P(2 successes in 4 trials with p=0.5) = 6 * (0.5)^4 = 0.375
/// assert!((binopdf(2.0, 4.0, 0.5) - 0.375).abs() < 1e-15);
/// ```
pub fn binopdf(x: f64, n: f64, p: f64) -> f64 {
    if x.is_nan() || n.is_nan() || p.is_nan() {
        return f64::NAN;
    }

    if x >= 0.0 && x == f64::round(x) && x <= n && n == f64::round(n) && p >= 0.0 && p <= 1.0 {
        // First deal with borderline cases
        if p == 0.0 {
            let y = if x == 0.0 { 1.0 } else { 0.0 };
            return y;
        }
        if p == 1.0 {
            let y = if x == n { 1.0 } else { 0.0 };
            return y;
        }

        // More borderline cases: x = 0 and x = n
        if x == 0.0 {
            let y = if p < 0.125 {
                f64::exp(n * f64::ln_1p(-p))
            } else {
                f64::exp(n * f64::ln(1.0 - p))
            };
            return y;
        }
        if x == n {
            let y = f64::exp(n * f64::ln(p));
            return y;
        }

        if n < 10.0 {
            // Faster method that is not accurate for large n
            let nk = gammaln(n + 1.0) - gammaln(x + 1.0) - gammaln(n - x + 1.0);
            let lny = nk + x * f64::ln(p) + (n - x) * f64::ln_1p(-p);
            let y = f64::exp(lny);
            return y;
        } else {
            let lc = stirlerr(n) - stirlerr(x) - stirlerr(n - x) - binodeviance(x, n * p) - binodeviance(n - x, n * (1.0 - p));
            let y = f64::exp(lc) * f64::sqrt(n / (2.0 * std::f64::consts::PI * x * (n - x)));
            return y;
        }        
    }

    if n < 0.0 || p < 0.0 || p > 1.0 || n != f64::round(n) {
        return f64::NAN;
    }

    return 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binopdf_basic() {
        let tol = 1e-15;
        // n=4, p=0.5, x=2 -> 0.375
        assert!((binopdf(2.0, 4.0, 0.5) - 0.375).abs() < tol);
        // n=1, p=0.5, x=0 -> 0.5
        assert!((binopdf(0.0, 1.0, 0.5) - 0.5).abs() < tol);
    }

    #[test]
    fn test_binopdf_boundaries() {
        let tol = 1e-15;
        // x = 0
        assert!((binopdf(0.0, 10.0, 0.1) - 0.3486784401).abs() < 1e-10);
        // x = n
        assert!((binopdf(10.0, 10.0, 0.1) - 1e-10).abs() < tol);
        
        // p = 0
        assert_eq!(binopdf(0.0, 5.0, 0.0), 1.0);
        assert_eq!(binopdf(1.0, 5.0, 0.0), 0.0);
        
        // p = 1
        assert_eq!(binopdf(5.0, 5.0, 1.0), 1.0);
        assert_eq!(binopdf(4.0, 5.0, 1.0), 0.0);
    }

    #[test]
    fn test_binopdf_large_n() {
        // Test Loader's expansion path (n >= 10)
        // Reference from SciPy: scipy.stats.binom.pmf(50, 100, 0.5)
        let val = binopdf(50.0, 100.0, 0.5);
        assert!((val - 0.07958923738717816).abs() < 1e-14);
    }

    #[test]
    fn test_binopdf_invalid() {
        assert!(binopdf(f64::NAN, 10.0, 0.5).is_nan());
        assert!(binopdf(5.0, -1.0, 0.5).is_nan());
        assert!(binopdf(5.0, 10.0, -0.1).is_nan());
        assert!(binopdf(5.0, 10.0, 1.1).is_nan());
        
        // Non-integers
        assert_eq!(binopdf(2.5, 4.0, 0.5), 0.0);
        assert!(binopdf(2.0, 4.5, 0.5).is_nan());
        
        // Out of range
        assert_eq!(binopdf(-1.0, 5.0, 0.5), 0.0);
        assert_eq!(binopdf(6.0, 5.0, 0.5), 0.0);
    }

    #[test]
    fn test_binopdf_symmetry() {
        // f(x; n, p) = f(n-x; n, 1-p)
        let n = 20.0;
        let p = 0.3;
        let x = 7.0;
        assert!((binopdf(x, n, p) - binopdf(n - x, n, 1.0 - p)).abs() < 1e-14);
    }
}