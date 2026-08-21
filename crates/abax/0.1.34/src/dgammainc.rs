use crate::{gammaln, psi};

/// Computes the incomplete gamma function and its first and second derivatives
/// with respect to the shape parameter `a`.
///
/// Returns a 3-element array `[dy, y, d2y]` where:
/// - `y`: The value of the regularized incomplete gamma function (<math><mi>P</mi><mo>(</mo><mi>a</mi><mo>,</mo><mi>x</mi><mo>)</mo></math> or <math><mi>Q</mi><mo>(</mo><mi>a</mi><mo>,</mo><mi>x</mi><mo>)</mo></math>).
/// - `dy`: The first derivative of the function with respect to $a$.
/// - `d2y`: The second derivative of the function with respect to $a$.
///
/// # Arguments
/// * `x` - The evaluation point.
/// * `a` - The shape parameter.
/// * `upper` - If `true`, computes the upper tail $Q(a, x)$; if `false`, the lower tail $P(a, x)$.
///
/// # Domain
/// - `a >= 0`: Returns `NaN` if `a < 0`. At `a = 0`, the function is well-defined but derivatives are infinite.
/// - `x`: Valid for all real numbers, though accuracy is reduced for $x < 0$ when $|x| > a + 1$.
#[allow(dead_code)]
pub(crate) fn dgammainc(x: f64, a: f64, upper: bool) -> [f64; 3] {
    let lower = !upper;
    let mut y = f64::NAN;
    let mut dy = y;
    let mut d2y = y;

    let mut x = x;
    let mut a = a;

    if a < 0.0 {
        return [dy, y, d2y];
    }
    
    // Upper limit for series and continued fraction.
    let amax = 2.0_f64.powf(20.0);
    
    // Approximation for a > amax. Accurate to about 5.e-5.
    if a > amax {
        x = f64::max(amax - 1.0 / 3.0 + f64::sqrt(amax / a) * (x - (a - 1.0 / 3.0)), 0.0);
        a = amax;
    }
    
    // Series expansion for lower incomplete gamma when x < a+1)
    if x < a + 1.0 && x != 0.0 {
        let xk = x;
        let ak = a;
        let mut aplusn = ak;
        let [mut del, mut ddel, mut d2del] = [1.0_f64, 0.0_f64, 0.0_f64];
        let [mut sum, mut dsum, mut d2sum] = [del, ddel, d2del];

        while del.abs() >= 100.0 * sum.abs().next_up() {
            aplusn += 1.0;
            del *= xk / aplusn;
            ddel = (ddel * xk - del) / aplusn;
            d2del = (d2del * xk - 2.0 * ddel) / aplusn;
            sum += del;
            dsum += ddel;
            d2sum += d2del;
        }
        let fac = f64::exp(-xk + ak * f64::ln(xk) - gammaln(ak + 1.0));
        let mut yk = fac * sum;
        // For very small a, the series may overshoot very slightly.
        if xk > 0.0 && yk > 1.0 {
            yk = 1.0;
        }
        y = if lower { yk } else { 1.0 - yk };
        
        let dlogfac = f64::ln(xk) - psi(0, ak + 1.0);
        let dfac = fac * dlogfac;
        let dyk = dfac * sum + fac * dsum;
        dy = if lower { dyk } else { -dyk };
        
        let d2fac = dfac * dlogfac - fac * psi(1, ak + 1.0);
        let d2yk = d2fac * sum + 2.0 * dfac * dsum + fac * d2sum;
        d2y = if lower { d2yk } else { -d2yk };
    }
    if x >= a + 1.0 {
        // implies x != 0.0
        let xk = x;
        let ak = a;

        let [g, dg, d2g] = contfrac(xk, ak);
        let frac = f64::exp(-xk + ak * f64::ln(xk) - gammaln(ak + 1.0));
        let yk = frac * g;

        let dlogfac = f64::ln(xk) - psi(0, ak + 1.0);
        let dfac = frac * dlogfac;
        let dyk = dfac * g + frac * dg;

        y = if lower { 1.0 - yk } else { yk };
        dy = if lower { -dyk } else { dyk };

        let d2fac = dfac * dlogfac - frac * psi(1, ak + 1.0);
        let d2yk = d2fac * g + 2.0 * dfac * dg + frac * d2g;
        d2y = if lower { -d2yk } else { d2yk };
    }
    
    // Handle x == 0 separately to get it exactly correct.
    if x == 0.0 {
        y = if lower { 0.0 } else { 1.0 };
        dy = 0.0;
        d2y = 0.0;
    }

    // a == 0, x != 0 is already handled by the power series or continued
    // fraction, now fill in dgammainc(0, 0). While we're at it, make
    // gammainc(x, 0) or 1-gammainc(x, 0) exact for any x, not just x == 0.
    if a == 0.0 {
        y = if lower { 1.0 } else { 0.0 };
        dy = if lower { -f64::INFINITY } else { f64::INFINITY };
        d2y = if lower { f64::INFINITY } else { -f64::INFINITY };
    }

    [dy, y, d2y]
}

// Continued fraction for upper incomplete gamma when x >= a+1
fn contfrac(x: f64, a: f64) -> [f64; 3] {
    let mut n = 0.0;
    let [mut a0, mut a1] = [0.0, a];
    let [mut b0, mut b1] = [1.0, x];
    let [mut da0, mut da1] = [0.0, 1.0];
    let [mut db0, mut db1] = [0.0, 0.0];
    let [mut d2a0, mut d2a1] = [0.0, 0.0];
    let [mut d2b0, mut d2b1] = [0.0, 0.0];
    //let [mut g, mut dg, mut d2g] = [a / x, 1.0 / x, 0.0];
    let mut g: f64;
    let mut dg: f64;
    let mut d2g: f64 = 0.0;

    loop {
        let rescale = 1.0 / b1; // keep terms from overflowing
        n += 1.0;
        let nminusa = n - a;
        d2a0 = (d2a1 + d2a0 * nminusa - 2.0 * da0) * rescale;
        d2b0 = (d2b1 + d2b0 * nminusa - 2.0 * db0) * rescale;
        da0 = (da1 + da0 * nminusa - a0) * rescale;
        db0 = (db1 + db0 * nminusa - b0) * rescale;
        a0 = (a1 + a0 * nminusa) * rescale;
        b0 = 1.0 + (b0 * nminusa) * rescale; // (b1 + b0 * nminusa) * rescale
        let nrescale = n * rescale;
        d2a1 = d2a0 * x + d2a1 * nrescale;
        d2b1 = d2b0 * x + d2b1 * nrescale;
        da1 = da0 * x + da1 * nrescale;
        db1 = db0 * x + db1 * nrescale;
        a1 = a0 * x + a1 * nrescale;
        b1 = b0 * x + n; // b0 * x + b1 * nrescale
        let d2gold = d2g;
        let gnew = a1 / b1;
        let dgnew = (da1 - gnew * db1) / b1;
        let d2gnew = (d2a1 - dgnew * db1 - gnew * d2b1 - dgnew * db1) / b1;

        // Testing d2g is more stringent than testing g or dg. d2g may
        // be zero, so use a strict inequality. Continuing to iterate
        // on converged values may add noise, so avoid that.
        g = gnew;
        dg = dgnew;
        d2g = d2gnew;

        if f64::abs(d2g - d2gold) > 100.0 * d2g.abs().next_up() {
            continue;
        }
        break;
    }

    [g, dg, d2g]
}
