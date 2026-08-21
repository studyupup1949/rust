use crate::{norminv, erfcinv, binocdf, binopdf};

/// Inverse of the binomial cumulative distribution function (CDF).
///
/// Returns the smallest integer `x` such that the binomial CDF evaluated at `x`
/// with parameters `n` (trials) and `p` (success probability) is greater than or equal to `y`.
///
/// # Parameters
/// * `y` - Probability value in the range `[0, 1]`.
/// * `n` - Number of independent trials (non-negative integer).
/// * `p` - Probability of success in each trial `[0, 1]`.
///
/// # Examples
/// ```
/// use abax::binoinv;
///
/// // For 10 trials with p=0.5, the smallest x such that P(X <= x) >= 0.5 is 5.
/// assert_eq!(binoinv(0.5, 10.0, 0.5), 5.0);
/// ```
pub fn binoinv(y: f64, n: f64, p: f64) -> f64 {
    if y.is_nan() || n.is_nan() || p.is_nan() || n < 0.0 || p < 0.0 || p > 1.0 || n != n.round() || y < 0.0 || y > 1.0 {
        return f64::NAN;
    }

    // Boundary cases
    if y == 1.0 || (y != 0.0 && p == 1.0) { return n; }
    if y == 0.0 || (y != 1.0 && p == 0.0) { return 0.0; }

    // 1. Initial Approximation
    let eta = erfcinv(2.0 * y) * (2.0 / (n + 1.0)).sqrt();
    let xi = xi_asymptotic_inversion_of_binomial_cdf(eta, p);

    let cannot_asymptotic = eta < -(-2.0 * (1.0 - p).ln()).sqrt()
        || eta > (-2.0 * p.ln()).sqrt()
        || xi <= 0.0 || xi >= 1.0
        || (eta >= 0.0 && p < xi)
        || (eta < 0.0 && p > xi);

    let mut x_approx = if !cannot_asymptotic {
        asymptotic_inversion_of_binomial_cdf(eta, xi, n, p)
    } else {
        // Note: Poisson approximation is skipped as poissinv is not yet implemented.
        // Falling back to Cornish-Fisher expansion.
        cornish_fisher_for_binomial_inverse(y, n, p)
    };

    x_approx = x_approx.clamp(0.0, n);

    // 2. Refinement for Discrete Distribution
    // We need the smallest x such that binocdf(x, n, p) >= y.
    let mut x = x_approx;
    let mut cum_dist = binocdf(x, n, p, false);
    let mut pdf_val = binopdf(x, n, p);

    let eps_val = |v: f64| v.next_up() - v;

    if cum_dist + 10.0 * eps_val(pdf_val) < y {
        // The approximation was too low, increment x.
        while x < n && cum_dist + 10.0 * eps_val(pdf_val) < y {
            x += 1.0;
            pdf_val = binopdf(x, n, p);
            cum_dist += pdf_val;
        }
    } else {
        // The approximation might be too high, check if x-1 also satisfies the condition.
        while x > 0.0 {
            pdf_val = binopdf(x, n, p);
            let c_temp = cum_dist - pdf_val;
            if c_temp + 10.0 * eps_val(pdf_val) < y {
                break;
            }
            cum_dist = c_temp;
            x -= 1.0;
        }
    }

    x
}

/// Computes `xi` to approximate binomial inverse using an asymptotic inversion algorithm.
///
/// This function is based on the algorithm described in:
/// Gil, A., Segura, J., & Temme, N. M. (2020). Asymptotic inversion of the
/// binomial and negative binomial cumulative distribution functions.
///
/// This function implements Equations 2.17 and 2.18 from the referenced paper.
///
/// # Arguments
/// * `eta` - Initial value of eta, computed using Equation 3.24 from the paper.
/// * `p` - Probability of success in each trial.
///
/// # Returns
/// The calculated `xi` value.
fn xi_asymptotic_inversion_of_binomial_cdf(eta: f64, p: f64) -> f64 {
    let a1 = 1.0;
    let a2 = (1.0 / 6.0) * (2.0 * p - 1.0);
    let a3 = (1.0 / 72.0) * (2.0 * p.powi(2) - 2.0 * p - 1.0);
    let a4 = (-1.0 / 540.0) * (2.0 * p.powi(3) - 3.0 * p.powi(2) - 3.0 * p + 2.0);
    let a5 = (1.0 / 17280.0) * (4.0 * p.powi(4) - 8.0 * p.powi(3) - 48.0 * p.powi(2) + 52.0 * p - 23.0);

    let etahat = eta / (p * (1.0 - p)).sqrt();
    let xi = p - p * (1.0 - p) * (
        a1 * etahat
        + a2 * etahat.powi(2)
        + a3 * etahat.powi(3)
        + a4 * etahat.powi(4)
        + a5 * etahat.powi(5)
    );
    xi
}

/// Computes first-order Cornish-Fisher expansion for binomial inverse approximation.
///
/// # Arguments
/// * `y` - Probability values at which to compute the inverse CDF.
/// * `n` - Number of trials in the binomial distribution.
/// * `p` - Probability of success in each trial. NOTE: `p` value must not be equal to 0 or 1.
///
/// # Note
/// * The caller must ensure that the result lies within the interval `[0, n]`.
/// * The caller should check for `NaN` values in the result and handle them as appropriate.
fn cornish_fisher_for_binomial_inverse(y: f64, n: f64, p: f64) -> f64 {
    let q = 1.0 - p;
    let mu = n * p;
    let sigma = (n * p * q).sqrt();
    let gamma = (q - p) / sigma; // skew term
    let z = norminv(y, 0.0, 1.0);
    // first order approximation
    (mu + sigma * (z + gamma * (z.powi(2) - 1.0) / 6.0) + 0.5).floor()
}

/// Computes `eta1` using a Taylor series expansion for binomial inverse approximation.
///
/// This function implements the series expansion given in Equation 3.30 of:
/// Gil, A., Segura, J., & Temme, N. M. (2020). Asymptotic inversion of the
/// binomial and negative binomial cumulative distribution functions.
///
/// Note: The signs for the 1st and 3rd coefficients have been corrected from the
/// original paper to yield values closer to the higher-order Equation 3.29.
///
/// # Arguments
/// * `eta0` - Initial value of eta (Equation 3.24).
/// * `xi`   - Initial approximation for XI (Equations 2.17 and 2.18).
pub(crate) fn eta_taylor_expansion_for_binomial_inverse(eta0: f64, xi: f64) -> f64 {
    let lambda = (xi * (1.0 - xi)).sqrt();

    let term1 = -(1.0 - 2.0 * xi) / (3.0 * lambda);
    let term2 = -((5.0 * xi.powi(2) - 5.0 * xi - 1.0) / (36.0 * lambda.powi(2))) * eta0;
    let term3 = (((2.0 * xi - 1.0) * (23.0 * xi.powi(2) - 23.0 * xi - 1.0)) / (1620.0 * lambda.powi(3)))
        * eta0.powi(2);
    let term4 = -((31.0 * xi.powi(4) - 62.0 * xi.powi(3) + 33.0 * xi.powi(2) - 2.0 * xi + 7.0)
        / (6480.0 * lambda.powi(4)))
        * eta0.powi(3);

    term1 + term2 + term3 + term4
}

/// Approximates the binomial inverse value using the asymptotic inversion algorithm.
///
/// This implements the steps described in Section 3.1 of:
/// Gil, A., Segura, J., & Temme, N. M. (2020). Asymptotic inversion of the
/// binomial and negative binomial cumulative distribution functions.
///
/// # Arguments
/// * `eta0` - Initial value of eta (Equation 3.24).
/// * `xi`   - Initial approximation for XI (Equations 2.17 and 2.18).
/// * `n`    - Number of trials in the binomial distribution.
/// * `p`    - Probability of success in each trial.
pub(crate) fn asymptotic_inversion_of_binomial_cdf(eta0: f64, xi: f64, n: f64, p: f64) -> f64 {
    // Step 3 - Compute value of eta1 using Equation 3.29 or 3.30 (modified) when p ~ xi
    // To avoid p == xi or numerical instability, a heuristic is used to choose between
    // the direct formula and the Taylor series expansion.
    let threshold = 100.0 * f64::EPSILON * xi.abs().max(f64::EPSILON);
    let use_formula = (p - xi).abs() >= threshold;

    let eta1 = if use_formula {
        let feta0 = eta0 * (xi * (1.0 - xi)).sqrt() / (p - xi);
        (1.0 / eta0) * feta0.ln()
    } else {
        eta_taylor_expansion_for_binomial_inverse(eta0, xi)
    };

    // Step 4 & 5 - Update eta and obtain a new approximation of xi
    let eta2 = eta0 + eta1 / (n + 1.0);
    let xi2 = xi_asymptotic_inversion_of_binomial_cdf(eta2, p);

    // Step 6 - Compute x and round to the nearest larger integer
    (xi2 * (n + 1.0) - 1.0).ceil()
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
