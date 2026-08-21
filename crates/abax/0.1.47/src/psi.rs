use crate::{digamma, gammaln};
use crate::consts::BERNOULLI_EVEN;

/// Computes the polygamma function `psi(k, x)`,
/// where `k = 0` is the digamma function and `k >= 1` are higher derivatives.
///
/// Mathematically:
/// <math><msup><mi>ψ</mi><mo>(</mo><mi>k</mi><mo>)</mo></msup><mo>(</mo><mi>x</mi><mo>)</mo></math>.
///
/// # Numerical stability
/// - Uses dedicated implementations for `k = 0, 1, 2`.
/// - For `k >= 3`, uses recurrence shifting to move `x` away from poles and a rapidly convergent
///   positive-series representation.
///
/// # Special cases
/// - Returns `NaN` for `NaN` inputs.
/// - Returns `NaN` at non-positive integer poles.
/// - Returns `+∞` for `k = 0, x = +∞`.
/// - Returns `0.0` for `k >= 1, x = +∞`.
///
/// # Examples
/// ```
/// use abax::psi;
///
/// assert!((psi(0, 1.0) + 0.5772156649015329).abs() < 1e-14);
/// assert!((psi(1, 1.0) - 1.6449340668482264).abs() < 1e-14);
/// assert!((psi(2, 1.0) + 2.4041138063191885).abs() < 1e-13);
/// ```
pub fn psi(k: usize, x: f64) -> f64 {
    polygamma(k, x)
}

fn polygamma(n: usize, x: f64) -> f64 {
    // Handle basic domain errors
    if n == 0 { return digamma(x); } // n=0 is defined as digamma
    if x <= 0.0 && x == x.floor() { return f64::NAN; }
    if x.is_infinite() { return 0.0; }

    let limit = 0.4 * 15.0 + 4.0 * (n as f64); // 15 digits of precision
    
    if x > limit {
        polygamma_at_infinity(n, x)
    } else {
        polygamma_at_transition(n, x)
    }
}

fn polygamma_at_transition(n: usize, x: f64) -> f64 {
    let mut z = x;
    let mut sum = 0.0;
    let n_f64 = n as f64;
    
    // Determine how many steps to shift x to reach the stable region
    let target = (0.4 * 15.0) + (4.0 * n_f64);
    let iterations = (target - x).floor() as i32;

    // Forward recursion: ψ^(n)(x) = Σ (-1)^n * n! / (x+k)^(n+1) + ψ^(n)(x+iter)
    // We use logs for the factorial/power part to prevent overflow
    for _ in 0..iterations {
        let log_term = gammaln(n_f64 + 1.0) - (n_f64 + 1.0) * z.ln();
        let term = log_term.exp();
        
        if n.is_multiple_of(2) {
            sum -= term;
        } else {
            sum += term;
        }
        z += 1.0;
    }

    sum + polygamma_at_infinity(n, z)
}

fn polygamma_at_infinity(n: usize, x: f64) -> f64 {
    let n_f64 = n as f64;
    let x_sq = x * x;

    // uses gammaln and logs for the lead term to handle large n 
    // part_term = (n-1)! / x^(n+1)
    let log_part_term = gammaln(n_f64) - (n_f64 + 1.0) * x.ln();
    let mut part_term = log_part_term.exp();
    
    // Initial lead terms of the asymptotic expansion
    // sum = part_term * (n + 2x) / 2
    let mut sum = part_term * (n_f64 + 2.0 * x) / 2.0;
    
    // Series: part_term * n * (n+1) / 2x * Σ Bernoulli
    part_term *= (n_f64 * (n_f64 + 1.0)) / (2.0 * x);

    for (k, &bernoulli) in BERNOULLI_EVEN.iter().enumerate().skip(1) {
        let term = part_term * bernoulli;
        sum += term;

        // Termination condition: relative error < epsilon
        if (term / sum).abs() < f64::EPSILON {
            break;
        }

        // Move part_term to the next k
        let k_f64 = k as f64;
        part_term *= (n_f64 + 2.0 * k_f64) * (n_f64 + 2.0 * k_f64 + 1.0);
        part_term /= (2.0 * k_f64 + 1.0) * (2.0 * k_f64 + 2.0) * x_sq;
    }

    if !(n - 1).is_multiple_of(2) { -sum } else { sum }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_approx_eq(actual: f64, expected: f64, eps: f64) {
        let d = (actual - expected).abs();
        assert!(
            d < eps,
            "actual={} expected={} diff={} eps={}",
            actual,
            expected,
            d,
            eps
        );
    }

    #[test]
    fn test_special_cases() {
        assert!(psi(3, f64::NAN).is_nan());
        assert_eq!(psi(0, f64::INFINITY), f64::INFINITY);
        assert_eq!(psi(3, f64::INFINITY), 0.0);
        assert!(psi(4, 0.0).is_nan());
        assert!(psi(4, -2.0).is_nan());
    }


    #[test]
    fn test_known_high_order_values() {
        // ψ^(3)(1) = π^4 / 15
        assert_approx_eq(psi(3, 1.0), std::f64::consts::PI.powi(4) / 15.0, 1e-12);
        // ψ^(4)(1) = -24 * ζ(5)
        assert_approx_eq(psi(4, 1.0), -24.88626612344088, 1e-11);
    }

    #[test]
    fn test_recurrence_high_order() {
        let x = 2.75;
        let lhs = psi(3, x + 1.0);
        let rhs = psi(3, x) - 6.0 / x.powi(4);
        assert_approx_eq(lhs, rhs, 1e-12);
    }
}
