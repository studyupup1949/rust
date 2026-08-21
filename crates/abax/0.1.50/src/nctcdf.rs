use crate::{betainc, gammaln, normcdf, tcdf};

/// Noncentral T cumulative distribution function (CDF).
///
/// Returns the cumulative distribution function for the noncentral T distribution
/// with `nu` degrees of freedom and noncentrality parameter `delta` at the value `x`.
///
/// # Mathematical Definition
/// The noncentral T distribution is the distribution of the random variable:
/// <math display="block">
///   <mi>T</mi>
///   <mo>=</mo>
///   <mfrac>
///     <mrow>
///       <mi>Z</mi>
///       <mo>+</mo>
///       <mi>δ</mi>
///     </mrow>
///     <msqrt>
///       <mi>V</mi>
///       <mo>/</mo>
///       <mi>ν</mi>
///     </msqrt>
///   </mfrac>
/// </math>
/// where <math><mi>Z</mi></math> is a standard normal distribution and <math><mi>V</mi></math> is a chi-squared distribution 
/// with <math><mi>ν</mi></math> degrees of freedom.
///
/// # Domain
/// - `nu > 0`
/// - Returns `NaN` if any input is `NaN` or `nu <= 0`.
///
/// # Examples
/// ```
/// use abax::nctcdf;
///
/// // delta = 0 reduces to the central Student's T distribution
/// let p = nctcdf(0.0, 5.0, 0.0, false);
/// assert!((p - 0.5).abs() < 1e-15);
/// ```

#[allow(non_snake_case)]
pub fn nctcdf(x: f64, nu: f64, delta: f64, upper: bool) -> f64 {
    let uppertail = upper;
    let mut p = 0.0;
    let sep = f64::EPSILON;

    if x.is_nan() || nu.is_nan() || delta.is_nan() {
        return f64::NAN;
    }

    if nu <= 0.0 || delta.is_infinite() {
        return f64::NAN;
    }
    if delta == 0.0 {
        return tcdf(x, nu, uppertail);
    }
    if x == f64::INFINITY {
        return if uppertail { 0.0 } else { 1.0 };
    }
    if nu > 2.0e6 {
        let s = 1.0 - 1.0 / (4.0 * nu);
        let d = (1.0 + x * x / (2.0 * nu)).sqrt();
        return normcdf(x * s, delta, d, uppertail);
    }
    if x < 0.0 {
        return nctcdf(-x, nu, -delta, !uppertail);
    }

    let xsq = x * x;
    let denom = nu + xsq;
    let P = xsq / denom;
    let Q = nu / denom;
    let dsq = delta * delta;

    if uppertail {
        if x == 0.0 {
            p = normcdf(-delta, 0.0, 1.0, true);
        }
    } else {
        p = normcdf(-delta, 0.0, 1.0, false);
    }

    if x != 0.0 {
        let signd = delta.signum();
        let mut subtotal = 0.0;

        let mut jj = 2.0 * f64::floor(dsq / 2.0);

        let mut E1 = (0.5 * jj * (0.5 * dsq).ln() - dsq / 2.0 - gammaln(jj / 2.0 + 1.0)).exp();
        let mut E2 = signd * (0.5 * (jj + 1.0) * (0.5 * dsq).ln() - dsq / 2.0 - gammaln((jj + 1.0) / 2.0 + 1.0)).exp();

        let mut t = P < 0.5;
        let mut B1 = 0.0;
        let mut B2 = 0.0;

        if uppertail {
            if t {
                B1 = betainc(P, (jj + 1.0) / 2.0, nu / 2.0, false);
                B2 = betainc(P, (jj + 2.0) / 2.0, nu / 2.0, false);
            }
            t = !t;
            if t {
                B1 = betainc(Q, nu / 2.0, (jj + 1.0) / 2.0, true);
                B2 = betainc(Q, nu / 2.0, (jj + 2.0) / 2.0, true);
            }
        } else {
            if t {
                B1 = betainc(P, (jj + 1.0) / 2.0, nu / 2.0, true);
                B2 = betainc(P, (jj + 2.0) / 2.0, nu / 2.0, true);
            }
            t = !t;
            if t {
                B1 = betainc(Q, nu / 2.0, (jj + 1.0) / 2.0, false);
                B2 = betainc(Q, nu / 2.0, (jj + 2.0) / 2.0, false);
            }
        }

        let mut R1 = (gammaln((jj + 1.0) / 2.0 + nu / 2.0) - gammaln((jj + 3.0) / 2.0) - gammaln(nu / 2.0) + 
            ((jj + 1.0) / 2.0) * P.ln() + (nu / 2.0) * Q.ln()).exp();
        let mut R2 = (gammaln((jj + 2.0) / 2.0 + nu / 2.0) - gammaln((jj + 4.0) / 2.0) - gammaln(nu / 2.0) + 
            ((jj + 2.0) / 2.0) * P.ln() + (nu / 2.0) * Q.ln()).exp();

        let E10 = E1;
        let E20 = E2;
        let B10 = B1;
        let B20 = B2;
        let R10 = R1;
        let R20 = R2;
        let j0 = jj;

        loop {
            let twoterms = E1 * B1 + E2 * B2;
            subtotal += twoterms;
            if twoterms.abs() <= (subtotal.abs() + sep) * sep {
                break;
            }
            jj += 2.0;
            E1 *= dsq / jj;
            E2 *= dsq / (jj + 1.0);
            if uppertail {
                B1 = betainc(P, (jj + 1.0) / 2.0, nu / 2.0, false);
                B2 = betainc(P, (jj + 2.0) / 2.0, nu / 2.0, false);
            } else {
                B1 -= R1;
                B2 -= R2;
                R1 *= P * (jj + nu - 1.0) / (jj + 1.0);
                R2 *= P * (jj + nu) / (jj + 2.0);
            }
        }
        E1 = E10;
        E2 = E20;
        B1 = B10;
        B2 = B20;
        R1 = R10;
        R2 = R20;
        jj = j0;
        let mut todo = jj > 0.0;
        while todo {
            let JJ = jj;
            E1 *= JJ / dsq;
            E2 *= (JJ + 1.0) / dsq;
            R1 *= (JJ + 1.0) / ((JJ + nu - 1.0) * P);
            R2 *= (JJ + 2.0) / ((JJ + nu) * P);
            if uppertail {
                B1 = betainc(P, (JJ - 1.0) / 2.0, nu / 2.0, false);
                B2 = betainc(P, JJ / 2.0, nu / 2.0, false);
            } else {
                B1 += R1;
                B2 += R2;
            }
            let twoterms = E1 * B1 + E2 * B2;
            subtotal += twoterms;
            jj -= 2.0;
            todo = twoterms.abs() > (subtotal.abs() + sep) * sep && jj > 0.0;
        }
        p = f64::min(1.0, f64::max(0.0, p + subtotal / 2.0));
    }
    
    p.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nctcdf_central() {
        let x = 1.5;
        let nu = 10.0;
        // Central T (delta=0)
        let p_nct = nctcdf(x, nu, 0.0, false);
        let p_t = tcdf(x, nu, false);
        assert!((p_nct - p_t).abs() < 1e-15);
    }

    #[test]
    fn test_nctcdf_known_values() {
        let tol = 1e-10;
        // Comparison with standard statistical tables/software (e.g., R nctcdf)
        println!("{:.16e}", nctcdf(2.0, 5.0, 1.0, false));
        // nu=5, delta=1, x=2
        assert!((nctcdf(2.0, 5.0, 1.0, false) - 7.7807466261621487e-1).abs() < tol);
        // nu=10, delta=2, x=2
        assert!((nctcdf(2.0, 10.0, 2.0, false) - 4.8097315281790721e-1).abs() < tol);
    }

    #[test]
    fn test_nctcdf_symmetry() {
        let x = 1.0;
        let nu = 8.0;
        let delta = 1.5;
        // F(x, v, delta) = 1 - F(-x, v, -delta)
        let p1 = nctcdf(x, nu, delta, false);
        let p2 = nctcdf(-x, nu, -delta, true);
        assert!((p1 - p2).abs() < 1e-14);
    }

    #[test]
    fn test_nctcdf_limits() {
        assert_eq!(nctcdf(f64::INFINITY, 5.0, 1.0, false), 1.0);
        assert_eq!(nctcdf(f64::NEG_INFINITY, 5.0, 1.0, false), 0.0);
        assert!(nctcdf(1.0, 0.0, 1.0, false).is_nan());
    }

    #[test]
    fn fff() {
        let x = nctcdf(-1.0, 5.0, -1.0, false);
        println!("{:16}", x);
    }
}
