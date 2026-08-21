use crate::betaln;

/// Regularized incomplete beta function <math><msub><mi>I</mi><mi>x</mi></msub><mo>(</mo><mi>z</mi><mo>,</mo><mi>w</mi><mo>)</mo></math>.
///
/// Solves for:
/// - <math><msub><mi>I</mi><mi>x</mi></msub><mo>(</mo><mi>z</mi><mo>,</mo><mi>w</mi><mo>)</mo></math> when `lower = true` (regularized lower incomplete beta)
/// - <math><mn>1</mn><mo>-</mo><msub><mi>I</mi><mi>x</mi></msub><mo>(</mo><mi>z</mi><mo>,</mo><mi>w</mi><mo>)</mo></math> when `lower = false` (regularized upper incomplete beta)
///
/// # Domain
/// - `0 <= x <= 1`
/// - `z > 0`, `w > 0`
/// - Invalid inputs return `NaN`.
pub fn betainc(x: f64, z: f64, w: f64, lower: bool) -> f64 {
    if x.is_nan() || z.is_nan() || w.is_nan() || x < 0.0 || x > 1.0 || z <= 0.0 || w <= 0.0 {
        return f64::NAN;
    }

    if x == 0.0 {
        return if lower { 0.0 } else { 1.0 };
    }
    if x == 1.0 { // Corrected from `x == 1.0` to `x == 1.0` (no change, just re-evaluating)
        return if lower { 1.0 } else { 0.0 };
    }
    if z == w {
        if z == 1.0 {
            return x;
        }
        if x == 0.5 {
            return 0.5;
        }
    }


    // Use symmetry: I_x(a, b) = 1 - I_{1-x}(b, a)
    // To ensure the continued fraction converges efficiently, we want x < (z+1)/(z+w+2)
    if x > (z + 1.0) / (z + w + 2.0) {
        return if lower {
            1.0 - betainc_cf(1.0 - x, w, z)
        } else {
            betainc_cf(1.0 - x, w, z)
        };
    }

    let val = betainc_cf(x, z, w);
    if lower { val } else { 1.0 - val }
}

/// Evaluates the continued fraction for the regularized incomplete beta function.
/// Uses Lentz's method for stability.
fn betainc_cf(x: f64, z: f64, w: f64) -> f64 {
    let ln_beta = betaln(z, w);
    let front = (z * x.ln() + w * (1.0 - x).ln() - ln_beta).exp() / z;

    let mut c = 1.0;
    let mut d = 1.0 - (z + w) * x / (z + 1.0);
    let tiny = 1e-30;

    if d.abs() < tiny { d = tiny; }
    d = 1.0 / d;
    let mut h = d;

    for m in 1..200 {
        let m_f = m as f64;
        let m2 = 2.0 * m_f;
        
        // Even step (2m)
        let aa = m_f * (w - m_f) * x / ((z + m2 - 1.0) * (z + m2));
        d = 1.0 + aa * d;
        if d.abs() < tiny { d = tiny; }
        c = 1.0 + aa / c;
        if c.abs() < tiny { c = tiny; }
        d = 1.0 / d;
        h *= d * c;

        // Odd step (2m+1)
        let aa = -(z + m_f) * (z + w + m_f) * x / ((z + m2) * (z + m2 + 1.0));
        d = 1.0 + aa * d;
        if d.abs() < tiny { d = tiny; }
        c = 1.0 + aa / c;
        if c.abs() < tiny { c = tiny; }
        d = 1.0 / d;
        let delta = d * c;
        h *= delta;

        if (delta - 1.0).abs() < 1e-16 {
            break;
        }
    }

    front * h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_betainc_basic() {
        // I_0.5(1, 1) = 0.5
        assert!((betainc(0.5, 1.0, 1.0, true) - 0.5).abs() < 1e-15);
        // I_0.5(2, 2) = 0.5
        assert!((betainc(0.5, 2.0, 2.0, true) - 0.5).abs() < 1e-15);
        // I_0.2(1, 3) = 1 - (1-0.2)^3 = 0.488
        assert!((betainc(0.2, 1.0, 3.0, true) - 0.488).abs() < 1e-15);
    }

    #[test]
    fn test_betainc_symmetry() {
        let x = 0.3;
        let z = 2.5;
        let w = 1.5;
        let lower = betainc(x, z, w, true);
        let upper = betainc(x, z, w, false);
        assert!((lower + upper - 1.0).abs() < 1e-15);
    }

    #[test]
    fn test_betainc_boundaries() {
        assert_eq!(betainc(0.0, 1.0, 1.0, true), 0.0);
        assert_eq!(betainc(1.0, 1.0, 1.0, true), 1.0);
    }
}
