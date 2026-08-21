use crate::{betainc, betaln, beta};

/// Inverse of the regularized incomplete beta function.
///
/// Solves for `x` such that:
/// - `betainc(x, z, w, true) = y` when `lower = true`
/// - `betainc(x, z, w, false) = y` when `lower = false`
///
/// # Domain
/// - `z > 0`, `w > 0`
/// - `0 <= y <= 1`
/// - Invalid inputs return `NaN`.
pub fn betaincinv(y: f64, z: f64, w: f64, lower: bool) -> f64 {
    if y.is_nan() || z.is_nan() || w.is_nan() || z <= 0.0 || w <= 0.0 || !(0.0..=1.0).contains(&y) {
        return f64::NAN;
    }

    if y == 0.0 {
        return if lower { 0.0 } else { 1.0 };
    }
    if y == 1.0 {
        return if lower { 1.0 } else { 0.0 };
    }

    // The root finder always works on the lower tail target.
    let target = if lower { y } else { 1.0 - y };

    let mut lo = 0.0;
    let mut hi = 1.0;
    
    // Initial guess: Start with the mean of the distribution
    let mut x = z / (z + w);
    
    // Refine initial guess for tail regions to accelerate convergence
    if target < 0.1 {
        x = (target * z * beta(z, w)).powf(1.0 / z).min(x);
    } else if target > 0.9 {
        let t_upper = 1.0 - target;
        let x_upper = (t_upper * w * beta(z, w)).powf(1.0 / w).min(1.0 - x);
        x = 1.0 - x_upper;
    }
    
    x = x.clamp(1e-15, 1.0 - 1e-15);

    let ln_beta = betaln(z, w);

    // Combined Root-Finding Loop (Halley + Newton + Bisection)
    for _ in 0..100 {
        // Ensure x stays within the current bracket
        if x <= lo || x >= hi {
            x = 0.5 * (lo + hi);
        }

        let val = betainc(x, z, w, true);
        let f = val - target;

        // Convergence criteria
        if f.abs() < 1e-14 * target.max(1e-12) {
            break;
        }

        // Update Bracket for Bisection fallback
        if f > 0.0 {
            hi = x;
        } else {
            lo = x;
        }

        // Derivative of regularized incomplete beta is the PDF:
        // f'(x) = x^(z-1) * (1-x)^(w-1) / B(z, w)
        let log_f_prime = (z - 1.0) * x.ln() + (w - 1.0) * (1.0 - x).ln() - ln_beta;
        let f_prime = log_f_prime.exp();

        // Second derivative ratio for Halley's method:
        // f''(x) / f'(x) = (z-1)/x - (w-1)/(1-x)
        let deriv_ratio = (z - 1.0) / x - (w - 1.0) / (1.0 - x);
        
        let mut delta = f / (f_prime - (f * deriv_ratio / 2.0));

        // Fallback to simple Newton if Halley's step is unstable or pushes out of bounds
        if !delta.is_finite() || (x - delta) <= lo || (x - delta) >= hi {
            delta = f / f_prime;
        }

        let x_new = x - delta;
        // Final fallback to Bisection if numerical precision issues occur
        if !x_new.is_finite() || x_new <= lo || x_new >= hi {
            x = 0.5 * (lo + hi);
        } else {
            if (x_new - x).abs() < f64::EPSILON * x.max(1e-12) {
                x = x_new;
                break;
            }
            x = x_new;
        }
    }

    x.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_betaincinv_roundtrip() {
        let z = 2.5;
        let w = 1.5;
        let x_vals = [0.1, 0.3, 0.5, 0.7, 0.9];
        for &x in &x_vals {
            let y = betainc(x, z, w, true);
            let x_inv = betaincinv(y, z, w, true);
            assert!((x - x_inv).abs() < 1e-12, "Roundtrip failed for x={}", x);
        }
    }

    #[test]
    fn test_betaincinv_identities() {
        // I_x(1, 1) = x, so inv(y) = y
        assert!((betaincinv(0.42, 1.0, 1.0, true) - 0.42).abs() < 1e-15);
    }
}