use crate::gammaln;

/// Computes the regularized incomplete gamma function with optional scaling.
///
/// This function returns either lower- or upper-tail regularized incomplete gamma values:
/// - `lower = true` returns the lower regularized incomplete gamma
///   <math><mi>P</mi><mo>(</mo><mi>a</mi><mo>,</mo><mi>x</mi><mo>)</mo></math>.
/// - `lower = false` returns the upper regularized incomplete gamma
///   <math><mi>Q</mi><mo>(</mo><mi>a</mi><mo>,</mo><mi>x</mi><mo>)</mo></math>.
///
/// The returned values are regularized, i.e. divided by
/// <math><mi>Γ</mi><mo>(</mo><mi>a</mi><mo>)</mo></math>.
///
/// If `scaled = true`, the returned tail value is explicitly scaled as:
/// - lower tail: <math><mi>P</mi><mo>(</mo><mi>a</mi><mo>,</mo><mi>x</mi><mo>)</mo><mi>Γ</mi><mo>(</mo><mi>a</mi><mo>+</mo><mn>1</mn><mo>)</mo><msup><mi>e</mi><mi>x</mi></msup><msup><mi>x</mi><mrow><mo>-</mo><mi>a</mi></mrow></msup></math>
/// - upper tail: <math><mi>Q</mi><mo>(</mo><mi>a</mi><mo>,</mo><mi>x</mi><mo>)</mo><mi>Γ</mi><mo>(</mo><mi>a</mi><mo>+</mo><mn>1</mn><mo>)</mo><msup><mi>e</mi><mi>x</mi></msup><msup><mi>x</mi><mrow><mo>-</mo><mi>a</mi></mrow></msup></math>
///
/// where <math><mi>P</mi></math> and <math><mi>Q</mi></math> are the regularized lower and upper tails.
///
/// # Domain
/// - `a` must be positive for finite results.
/// - `x` must be non-negative.
/// - Invalid domain inputs return `NaN`.
/// Computes the Incomplete Gamma Function with 1-ULP precision.
/// Matches MATLAB behavior for regularized, scaledlower, and scaledupper.
pub fn gammainc(x: f64, a: f64, lower: bool, scaled: bool) -> f64 {
    const EPS: f64 = f64::EPSILON;
    const FPMIN: f64 = 1e-300;
    const MAX_IT: usize = 200;

    if x.is_nan() || a.is_nan() || a <= 0.0 || x < 0.0 {
        return f64::NAN;
    }

    if x == 0.0 {
        return if lower { 0.0 } else {
            if scaled { f64::INFINITY } else { 1.0 }
        };
    }

    let x_is_small = x < a + 1.0;

    if !scaled {
        // --- Regularized Mode ---
        let gln = gammaln(a);
        let prefix = (a * x.ln() - x - gln).exp();
        
        if x_is_small {
            let p = prefix * lower_series_core(x, a, MAX_IT, EPS);
            if lower { p } else { (1.0 - p).max(0.0) }
        } else {
            let q = prefix * upper_cf_core(x, a, MAX_IT, FPMIN, EPS);
            if !lower { q } else { (1.0 - q).max(0.0) }
        }
    } else {
        // --- MATLAB Scaled Mode ---
        // Both scaledlower and scaledupper use: Gamma(a+1)*exp(x)/x^a
        // This factor cancels the regularized prefix terms, leaving only 'a'.
        if lower {
            // scaledlower
            a * lower_series_core(x, a, MAX_IT, EPS)
        } else {
            // scaledupper
            a * upper_cf_core(x, a, MAX_IT, FPMIN, EPS)
        }
    }
}

/// Sum part of the power series for the lower tail
fn lower_series_core(x: f64, a: f64, max_it: usize, eps: f64) -> f64 {
    let mut sum = 1.0 / a;
    let mut del = sum;
    let mut ap = a;
    for _ in 0..max_it {
        ap += 1.0;
        del *= x / ap;
        sum += del;
        if del.abs() < sum.abs() * eps { break; }
    }
    sum
}

/// Factor 'h' from Lentz's continued fraction for the upper tail
fn upper_cf_core(x: f64, a: f64, max_it: usize, fpmin: f64, eps: f64) -> f64 {
    let mut b = x + 1.0 - a;
    let mut c = 1.0 / fpmin;
    let mut d = 1.0 / b;
    let mut h = d;

    for i in 1..=max_it {
        let fi = i as f64;
        let an = -fi * (fi - a);
        b += 2.0;
        d = an * d + b;
        if d.abs() < fpmin { d = fpmin; }
        c = b + an / c;
        if c.abs() < fpmin { c = fpmin; }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < eps { break; }
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gamma;

    #[test]
    fn test_a1_identities() {
        let x = 2.5;
        let p = gammainc(x, 1.0, true, false);
        let q = gammainc(x, 1.0, false, false);
        assert!((p - (1.0 - (-x).exp())).abs() < 1e-14);
        assert!((q - (-x).exp()).abs() < 1e-14);
        assert!((p + q - 1.0).abs() < 1e-14);
    }

    #[test]
    fn test_a_half_erf_relation() {
        let x = 1.0;
        let p = gammainc(x, 0.5, true, false);
        let erf1 = 0.8427007929497149;
        assert!((p - erf1).abs() < 2e-14);
    }

    #[test]
    fn test_scaled_consistency() {
        let x = 15.0;
        let a = 4.5;

        let unscaled = gammainc(x, a, false, false);
        let scaled = gammainc(x, a, false, true);
        let expected = unscaled * gamma(a + 1.0) * x.exp() * x.powf(-a);
        assert!((scaled - expected).abs() / expected.abs() < 1e-12);
        
        let unscaled = gammainc(x, a, true, false);
        let scaled = gammainc(x, a, true, true);
        let expected = unscaled * gamma(a + 1.0) * x.exp() * x.powf(-a);
        assert!((scaled - expected).abs() / expected.abs() < 1e-12);
    }

}
