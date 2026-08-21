use crate::binopdf;

/// Binomial cumulative distribution function (CDF).
///
/// Returns the probability of obtaining `x` or fewer successes in `n` independent
/// Bernoulli trials, each with success probability `p`.
///
/// # Mathematical Definition
/// For a binomial distribution with trials `n` and probability `p`:
/// - Lower tail (`upper = false`):
/// <math display="block">
///   <mi>F</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>n</mi><mo>,</mo><mi>p</mi><mo>)</mo>
///   <mo>=</mo>
///   <munderover><mo>∑</mo><mrow><mi>i</mi><mo>=</mo><mn>0</mn></mrow><mrow><mo>⌊</mo><mi>x</mi><mo>⌋</mo></mrow></munderover>
///   <mrow><mo>(</mo><mfrac linethickness="0"><mi>n</mi><mi>i</mi></mfrac><mo>)</mo></mrow>
///   <msup><mi>p</mi><mi>i</mi></msup>
///   <msup><mrow><mo>(</mo><mn>1</mn><mo>-</mo><mi>p</mi><mo>)</mo></mrow><mrow><mi>n</mi><mo>-</mo><mi>i</mi></mrow></msup>
/// </math>
/// - Upper tail (`upper = true`):
/// <math display="block">
///   <mn>1</mn><mo>-</mo><mi>F</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>n</mi><mo>,</mo><mi>p</mi><mo>)</mo>
///   <mo>=</mo>
///   <munderover><mo>∑</mo><mrow><mi>i</mi><mo>=</mo><mo>⌊</mo><mi>x</mi><mo>⌋</mo><mo>+</mo><mn>1</mn></mrow><mi>n</mi></munderover>
///   <mrow><mo>(</mo><mfrac linethickness="0"><mi>n</mi><mi>i</mi></mfrac><mo>)</mo></mrow>
///   <msup><mi>p</mi><mi>i</mi></msup>
///   <msup><mrow><mo>(</mo><mn>1</mn><mo>-</mo><mi>p</mi><mo>)</mo></mrow><mrow><mi>n</mi><mo>-</mo><mi>i</mi></mrow></msup>
/// </math>
///
/// # Examples
/// ```
/// use abax::binocdf;
///
/// // P(2 or fewer successes in 4 trials with p=0.5)
/// // P(0)+P(1)+P(2) = 1/16 + 4/16 + 6/16 = 11/16 = 0.6875
/// assert!((binocdf(2.0, 4.0, 0.5, false) - 0.6875).abs() < 1e-15);
///
/// // Upper tail: P(X > 2) = P(3) + P(4) = 4/16 + 1/16 = 5/16 = 0.3125
/// assert!((binocdf(2.0, 4.0, 0.5, true) - 0.3125).abs() < 1e-15);
/// ```
pub fn binocdf(x: f64, n: f64, p: f64, upper: bool) -> f64 {
    let (x, p) = if upper { (n - 1.0 - f64::floor(x), 1.0 - p) } else { (x, p) };

    let mut y = 0.0;
    if x >= n {
        y = 1.0;
    }
    if x.is_nan() {
        y = f64::NAN;
    }

    let k2 = p == 0.0 && x >= 0.0;
    if k2 {
        y = 1.0;
    }

    let k3 = p == 1.0;
    if k3 {
        y = if x >= n { 1.0 } else { 0.0 };
    }

    let k1 = n < 0.0 || p < 0.0 || p > 1.0 || n != f64::round(n);
    if k1 {
        y = f64::NAN;
    }

    let xx = f64::floor(x);
    let k = xx >= 0.0 && xx < n && !k1 && !k2 && !k3;
    if k {
        let smallp = 1.0e-4;  // to maintain accuracy, don't switch tails below here
        let bign = 1.0e5;     // to conserve memory, sum over smaller range above here
        
        // for the simplest case, sum the probabilities from 0
        let np = n * p;
        let t = (n < bign) & ((x <= np) | (p < smallp));
        if t {
            y = sumfrom0(xx, n, p);
        } else if n < bign {
            y = 1.0 - sumfrom0(n - xx - 1.0, n, 1.0 - p);
        } else {
            y = sumA2B(xx, n, p);
        }
        
    }

    return y.clamp(0.0, 1.0);
}

fn sumfrom0(xx: f64, n: f64, p: f64) -> f64 {
    let y: f64 = (0..=xx as usize).map(|i| binopdf(i as f64, n, p)).sum();
    return y;
}

#[allow(non_snake_case)]
fn sumA2B(x: f64, N: f64, p: f64) -> f64 {
    let mut y: f64 = 0.0;

    // See how far we need to look into each tail
    let mu = N * p;
    let std = f64::sqrt(N * p * (1.0 - p));
    let mut t1 = 40.0;
    let mut t2 = 10.0;
    while binopdf(f64::floor(mu - t1 * std), N, p) > eps(0.0) {
        t1 = 1.5 * t1;
    }
    while binopdf(f64::ceil(mu + t2 * std), N, p) > eps(1.0) {
        t2 = 1.5 * t2;
    }

    let a = f64::max(0.0, f64::floor(mu - t1 * std));
    let b = f64::ceil(mu + t2 * std);

    if x < a {
        y = 0.0;
    }
    if x > b {
        y = 1.0;
    }
    
    if x >= a && x <= b {
        let mut sums = (a as usize .. x as usize + 1).map(|i| binopdf(i as f64, N, p)).collect::<Vec<_>>();
        for i in 1..sums.len() {
            sums[i] += sums[i - 1];
        }
        y = sums[x as usize - a as usize];
    }

    return y;
}

fn eps(x: f64) -> f64 {
    let next_up = x.next_up();
    return next_up - x;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binocdf_ex() {
        let x = 100.0;
        let n = 123456.0;
        let p = 0.001;
        let y = binocdf(x, n, p, false);
        assert!((y - 1.695618562854513e-02).abs() < 1.0e-15);
    }

    #[test]
    fn test_binocdf_basic() {
        let tol = 1e-15;
        // n=4, p=0.5, x=2 -> 0.6875
        assert!((binocdf(2.0, 4.0, 0.5, false) - 0.6875).abs() < tol);
        // n=4, p=0.5, x=2, upper -> 0.3125
        assert!((binocdf(2.0, 4.0, 0.5, true) - 0.3125).abs() < tol);
    }

    #[test]
    fn test_binocdf_boundaries() {
        // x < 0
        assert_eq!(binocdf(-1.0, 10.0, 0.5, false), 0.0);
        assert_eq!(binocdf(-1.0, 10.0, 0.5, true), 1.0);
        // x >= n
        assert_eq!(binocdf(10.0, 10.0, 0.5, false), 1.0);
        assert_eq!(binocdf(10.0, 10.0, 0.5, true), 0.0);
        assert_eq!(binocdf(11.0, 10.0, 0.5, false), 1.0);
    }

    #[test]
    fn test_binocdf_edge_p() {
        // p = 0
        assert_eq!(binocdf(0.0, 5.0, 0.0, false), 1.0);
        assert_eq!(binocdf(-1.0, 5.0, 0.0, false), 0.0);
        assert_eq!(binocdf(5.0, 5.0, 0.0, false), 1.0);
        
        // p = 1
        assert_eq!(binocdf(4.0, 5.0, 1.0, false), 0.0);
        assert_eq!(binocdf(5.0, 5.0, 1.0, false), 1.0);
        assert_eq!(binocdf(4.0, 5.0, 1.0, true), 1.0);
    }

    #[test]
    fn test_binocdf_invalid() {
        assert!(binocdf(2.0, -1.0, 0.5, false).is_nan());
        assert!(binocdf(2.0, 4.0, -0.1, false).is_nan());
        assert!(binocdf(2.0, 4.0, 1.1, false).is_nan());
        assert!(binocdf(2.0, 4.5, 0.5, false).is_nan());
        assert!(binocdf(f64::NAN, 4.0, 0.5, false).is_nan());
    }

    #[test]
    fn test_binocdf_sum_switch() {
        // Check p < smallp path
        let val = binocdf(1.0, 100.0, 1e-5, false);
        // approx P(0) + P(1) = (1-p)^n + n*p*(1-p)^(n-1)
        let p: f64 = 1e-5;
        let n: f64 = 100.0;
        let expected = (1.0 - p).powf(n) + n * p * (1.0 - p).powf(n - 1.0);
        assert!((val - expected).abs() < 1e-14);
    }

    #[test]
    fn test_binocdf_large_n() {
        // Check sumA2B path (n >= 1.0e5)
        // Reference from SciPy: scipy.stats.binom.cdf(50000, 100000, 0.5)
        // This is inclusive of the median mass.
        let val = binocdf(50000.0, 100000.0, 0.5, false);
        assert!((val - 5.012615631070982e-01).abs() < 1e-8);
    }
}