use crate::{nctcdf, nctpdf, norminv, tinv};

/// Inverse of the noncentral T cumulative distribution function (CDF).
///
/// Given a probability `p`, degrees of freedom `nu`, and noncentrality parameter `delta`,
/// this function returns the value `x` such that the probability of a noncentral T
/// random variable being less than or equal to `x` is `p`.
///
/// # Mathematical Definition
/// The function finds $x$ such that:
/// <math display="block">
///   <msub><mi>F</mi><mrow><mi>NC</mi><mi>T</mi></mrow></msub>
///   <mo>(</mo><mi>x</mi><mo>;</mo><mi>ν</mi><mo>,</mo><mi>δ</mi><mo>)</mo>
///   <mo>=</mo>
///   <mi>p</mi>
/// </math>
/// where <math><msub><mi>F</mi><mrow><mi>NC</mi><mi>T</mi></mrow></msub></math> is the noncentral T CDF.
///
/// # Implementation
/// The inversion is performed using Newton's method with the standard normal
/// quantile as an initial guess. It incorporates step-size limiting and
/// back-tracking to ensure convergence.
///
/// # Domain
/// - <math><mn>0</mn><mo>≤</mo><mi>p</mi><mo>≤</mo><mn>1</mn></math>
/// - <math><mi>ν</mi><mo>&gt;</mo><mn>0</mn></math>
/// - <math><mi>δ</mi></math> must be finite.
/// - Returns `NaN` if `p` is out of range, `nu <= 0`, or `delta` is non-finite.
///
/// # Examples
/// ```
/// use abax::nctinv;
///
/// // For delta = 0, it reduces to the central Student's T distribution inverse.
/// let x = nctinv(0.5, 5.0, 0.0);
/// assert!((x - 0.0).abs() < 1e-12);
///
/// // Noncentral case
/// let x_nc = nctinv(0.5, 10.0, 2.0);
/// assert!((x_nc - 2.0536911511184894041).abs() < 1e-10);
/// ```
#[allow(non_snake_case)]
pub fn nctinv(p: f64, nu: f64, delta: f64) -> f64 {
    if delta == 0.0 {
        let x = tinv(p, nu);
        return x;
    }

    let ok_params = (nu > 0.0 && !delta.is_nan()) && !delta.is_infinite();
    if !ok_params {
        return f64::NAN;
    }
    
    match p {
        0.0 => return -f64::INFINITY,
        1.0 => return f64::INFINITY,
        _ if p < 0.0 || p > 1.0 => return f64::NAN,
        _ => (),
    }

    let crit = f64::sqrt(f64::EPSILON);
    let pk = p;
    let vk = nu;
    let dk = delta;

    // Newton's Method
    // Permit no more than count_limit iterations.
    const COUNT_LIMIT: usize = 100;
    let mut count = 0;

    // Use norminv as a starting guess for x.
    let mut xk = norminv(pk, dk, 1.0);

    let mut h = 1.0;

    // Break out of the iteration loop for the following:
    //  1) The last update is very small (compared to x or in abs.value).
    //  2) There are more than 100 iterations.

    let mut F = nctcdf(xk, vk, dk, false);
    while f64::abs(h) > crit * f64::abs(xk) && f64::abs(h) > crit && count < COUNT_LIMIT {
        count += 1;
        let f = nctpdf(xk, vk, dk);
        h = (F - pk) / f;

        // If h is inf or NaN, step closer to the original instead
        let blowup = h.is_infinite() || h.is_nan();
        if blowup {
            h = xk / 10.0;
        }

        // Avoid stepping too far
        let mut xnew = f64::max(-5.0 * f64::abs(xk), f64::min(5.0 * f64::abs(xk), xk - h));

        // Back off if the step gives a worse result
        let mut Fnew = nctcdf(xnew, vk, dk, false);
        loop {
            let worse = (f64::abs(Fnew - pk) > f64::abs(F - pk) * (1.0 + crit))
                && (f64::abs(xk - xnew) > crit * f64::abs(xk));
            if !worse {
                break;
            }
            xnew = 0.5 * (xnew + xk);
            Fnew = nctcdf(xnew, vk, dk, false);
        }

        xk = xnew;
        F = Fnew;
    }

    // Return the converged value(s).
    if cfg!(debug_assertions) {
        assert!(count < COUNT_LIMIT);
    }

    return xk;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nctinv_central() {
        let nu = 10.0;
        // Median should be 0.0
        assert!((nctinv(0.5, nu, 0.0) - 0.0).abs() < 1e-14);
        // Compare with tinv for non-median
        assert!((nctinv(0.975, nu, 0.0) - tinv(0.975, nu)).abs() < 1e-14);
    }

    #[test]
    fn test_nctinv_known_values() {
        let tol = 1e-10;
        assert!((nctinv(0.5, 5.0, 1.0) - 1.052851040947396e+00).abs() < tol);
        assert!((nctinv(0.9, 10.0, 2.0) - 3.746633570263350e+00).abs() < tol);
    }

    #[test]
    fn test_nctinv_symmetry() {
        let p = 0.3;
        let nu = 8.0;
        let delta = 1.5;
        // Inverse identity: nctinv(p, v, d) = -nctinv(1-p, v, -d)
        let x1 = nctinv(p, nu, delta);
        let x2 = nctinv(1.0 - p, nu, -delta);
        assert!((x1 + x2).abs() < 1e-12);
    }

    #[test]
    fn test_nctinv_boundaries() {
        assert_eq!(nctinv(0.0, 5.0, 1.0), f64::NEG_INFINITY);
        assert_eq!(nctinv(1.0, 5.0, 1.0), f64::INFINITY);
    }

    #[test]
    fn test_nctinv_invalid() {
        assert!(nctinv(-0.1, 5.0, 1.0).is_nan());
        assert!(nctinv(1.1, 5.0, 1.0).is_nan());
        assert!(nctinv(0.5, 0.0, 1.0).is_nan());
        assert!(nctinv(0.5, -1.0, 1.0).is_nan());
        assert!(nctinv(0.5, 5.0, f64::INFINITY).is_nan());
        assert!(nctinv(0.5, 5.0, f64::NAN).is_nan());
    }
}
