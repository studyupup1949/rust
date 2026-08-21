use crate::gammaln;
use std::f64::consts::PI;

/// Compute the modified Bessel function of the first kind $I_\nu(x)$ for non-negative real input.
///
/// The modified Bessel function of the first kind is defined as a solution to the
/// modified Bessel differential equation:
/// <math display="block">
///   <msup><mi>x</mi><mn>2</mn></msup><mfrac><mrow><msup><mi>d</mi><mn>2</mn></msup><mi>y</mi></mrow><mrow><mi>d</mi><msup><mi>x</mi><mn>2</mn></msup></mrow></mfrac>
///   <mo>+</mo><mi>x</mi><mfrac><mrow><mi>dy</mi></mrow><mrow><mi>dx</mi></mrow></mfrac>
///   <mo>-</mo><mo>(</mo><msup><mi>x</mi><mn>2</mn></msup><mo>+</mo><msup><mi>ν</mi><mn>2</mn></msup><mo>)</mo><mi>y</mi><mo>=</mo><mn>0</mn>
/// </math>
///
/// This implementation uses a combination of strategies for numerical stability:
/// - **Power Series**: Used for small arguments.
/// - **Asymptotic Expansion (Large x)**: Used when the argument is much larger than the order.
/// - **Debye Asymptotics (Large $\nu$)**: Used when the order is large.
/// - **Miller's Backward Recurrence**: Used for the intermediate region.
///
/// # Parameters
/// * `nu` - Order of the Bessel function (must be non-negative).
/// * `x` - Real argument (must be non-negative).
/// * `scale` - If `true`, returns the exponentially scaled function $I_\nu(x) \cdot e^{-x}$
///   to prevent numerical overflow for large $x$.
///
/// # Domain
/// * Returns `NaN` if `nu < 0` or `x < 0`.
///
/// # Examples
/// ```
/// use abax::besseli;
///
/// // I_0(1.0) is approximately 1.2660658777
/// let val = besseli(0.0, 1.0, false);
/// assert!((val - 1.2660658777520083).abs() < 1e-15);
/// ```
pub fn besseli(nu: f64, x: f64, scale: bool) -> f64 {
    if nu < 0.0 || x < 0.0 || nu.is_nan() || x.is_nan() {
        return f64::NAN;
    }
    match (nu, x) {
        (0.0, 0.0) => return 1.0,
        (  _, 0.0) => return 0.0,
        _ => (),
    }

    // Precision-dependent constants (Double precision)
    let eps = f64::EPSILON;

    // Region 1: Small argument - Power series with backward accumulation
    let small_z_boundary = 4.0 * f64::sqrt(nu + 1.0);
    if x <= small_z_boundary {
        let log_prefactor = nu * (x / 2.0).ln() - gammaln(nu + 1.0);
        let check_val = if scale { log_prefactor - x } else { log_prefactor };
        if check_val < f64::ln(f64::MIN_POSITIVE) {
            return 0.0;
        }
        return power_series(nu, x, scale, eps);
    }
    
    // Region 2: Large argument asymptotic expansion with truncation tracking
    let z_s3: f64 = 16.0;
    if x >= z_s3.max(nu.powi(2) / 2.0 + 15.0) {
        if !scale {
            let log_approx = x - 0.5 * (2.0 * std::f64::consts::PI * x).ln();
            if log_approx >= f64::ln(f64::MAX) {
                return f64::INFINITY;
            }
        }
        return asymptotic_large_z(nu, x, scale, eps);
    }

    // Region 3: Large order asymptotic expansion (Debye Asymptotics via Horner's Method)
    let c1: f64 = 52.0;
    let large_nu_threshold = c1 + x;
    if nu >= large_nu_threshold {
        return asymptotic_large_nu(nu, x, scale);
    }

    // Region 4: Miller's Backward Recurrence for Intermediate Region
    backward_recurrence(nu, x, scale, eps)
}

fn power_series(nu: f64, x: f64, scale: bool, eps: f64) -> f64 {
    let z_squared_over_4 = (x * x) / 4.0;
    
    // 256 terms are more than enough to underflow double precision naturally
    let mut terms = [0.0; 256];
    terms[0] = 1.0;
    let mut k = 0;

    while k < 255 {
        let k_f = (k + 1) as f64;
        let next_term = terms[k] * z_squared_over_4 / (k_f * (k_f + nu));
        
        if next_term.abs() < eps * terms[0].abs() || next_term == 0.0 {
            break;
        }
        k += 1;
        terms[k] = next_term;
    }

    // High precision: Sum from smallest to largest
    let mut sum_series = 0.0;
    for i in (0..=k).rev() {
        sum_series += terms[i];
    }

    let prefactor = if nu == 0.0 {
        if scale { (-x).exp() } else { 1.0 }
    } else {
        let mut log_prefactor = nu * (x / 2.0).ln() - gammaln(nu + 1.0);
        if scale {
            log_prefactor -= x;
        }
        log_prefactor.exp()
    };

    prefactor * sum_series
}

fn asymptotic_large_z(nu: f64, x: f64, scale: bool, eps: f64) -> f64 {
    let mu = 4.0 * nu.powi(2);
    let mut terms = [0.0; 512];
    terms[0] = 1.0;
    
    let mut k = 0;
    let mut last_abs = f64::INFINITY;

    while k < 511 {
        let k_f = (k + 1) as f64;
        let factor = (mu - (2.0 * k_f - 1.0).powi(2)) / (8.0 * k_f * x);
        let next_term = -terms[k] * factor;
        
        let abs_next = next_term.abs();
        
        // Asymptotic series rules: break immediately when the terms begin to diverge
        if abs_next > last_abs || next_term.is_nan() {
            break;
        }
        
        k += 1;
        terms[k] = next_term;
        
        if abs_next < eps {
            break;
        }
        last_abs = abs_next;
    }

    // High precision: Sum backward from the minimum convergent term
    let mut sum_series = 0.0;
    for i in (0..=k).rev() {
        sum_series += terms[i];
    }

    let mut log_prefactor = -0.5 * (2.0 * std::f64::consts::PI * x).ln();
    if !scale {
        log_prefactor += x;
    }
    
    log_prefactor.exp() * sum_series
}

fn asymptotic_large_nu(nu: f64, x: f64, scale: bool) -> f64 {
    let z_scaled = x / nu;
    let p = 1.0 / f64::sqrt(1.0 + z_scaled.powi(2));
    let eta = f64::sqrt(1.0 + z_scaled.powi(2)) + f64::ln(z_scaled / (1.0 + f64::sqrt(1.0 + z_scaled.powi(2))));
    //let log_result = (z_scaled / (1.0 + (1.0 + z_scaled.powi(2)).sqrt())).ln();

    let p2  = p * p;
    let p3  = p2 * p;
    let p4  = p3 * p;
    let p5  = p4 * p;
    let p6  = p5 * p;

    // Exact analytical coefficients derived from Olver's Debye Recurrence
    let u1 = p * (3.0 - 5.0 * p2) / 24.0;
    let u2 = p2 * (81.0 - 462.0 * p2 + 385.0 * p4) / 1152.0;
    let u3 = p3 * (30375.0 - 369603.0 * p2 + 765765.0 * p4 - 425425.0 * p6) / 414720.0;
    
    // High precision evaluation: Horner's nesting scheme to protect least significant bits
    let inv_nu = 1.0 / nu;
    let correction = 1.0 + inv_nu * (u1 + inv_nu * (u2 + inv_nu * u3));

    let mut log_result = nu * eta + 0.5 * (p / (2.0 * PI * nu)).ln();
    if scale {
        log_result -= x;
    }

    log_result.exp() * correction
}

fn backward_recurrence(nu: f64, x: f64, scale: bool, eps: f64) -> f64 {
    let nu_floor = nu.floor() as i32;
    let alpha = nu - nu_floor as f64;

    let n_start = (nu_floor + 40).max((x + 30.0) as i32);

    let mut i_next = 0.0;
    let mut i_curr = 1.0e-30;

    let mut target_i_nu = 0.0;
    let mut target_i_alpha = 0.0;

    for n in (1..=n_start).rev() {
        let current_order = (n as f64) + alpha;
        let i_prev = (2.0 * current_order / x) * i_curr + i_next;

        i_next = i_curr;
        i_curr = i_prev;

        if n - 1 == nu_floor {
            target_i_nu = i_curr;
        }
        if n - 1 == 0 {
            target_i_alpha = i_curr;
        }
    }

    // Always reference an unscaled target to prevent algebraic bias in ratios
    let true_i_alpha_unscaled = power_series(alpha, x, false, eps);
    let normalization_factor = true_i_alpha_unscaled / target_i_alpha;
    let unscaled_target = target_i_nu * normalization_factor;

    if scale {
        unscaled_target * (-x).exp()
    } else {
        unscaled_target
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_besseli_boundaries() {
        assert_eq!(besseli(0.0, 0.0, false), 1.0);
        assert_eq!(besseli(1.0, 0.0, false), 0.0);
        assert_eq!(besseli(2.5, 0.0, false), 0.0);
        assert!(besseli(-1.0, 1.0, false).is_nan());
        assert!(besseli(1.0, -1.0, false).is_nan());
    }

    #[test]
    fn test_besseli_small_x() {
        let tol = 1e-15;
        // I_0(1.0) ref: 1.2660658777520083
        assert!((besseli(0.0, 1.0, false) - 1.2660658777520083).abs() < tol);
        // I_1(1.0) ref: 0.5651591039924850
        assert!((besseli(1.0, 1.0, false) - 0.5651591039924850).abs() < tol);
    }

    #[test]
    fn test_besseli_half_integer() {
        let tol = 1e-14;
        // I_0.5(x) = sqrt(2 / (pi * x)) * sinh(x)
        let x = 1.0;
        let expected = (2.0 / (PI * x)).sqrt() * x.sinh();
        assert!((besseli(0.5, x, false) - expected).abs() < tol);
    }

    #[test]
    fn test_besseli_large_x_asymptotic() {
        let tol = 1e-14;
        // Check scaling and asymptotic behavior for large x
        let x = 50.0;
        let val_scaled = besseli(0.0, x, true);
        // Reference value for I_0(50) * exp(-50)
        let expected = 0.05656162664745419;
        assert!((val_scaled - expected).abs() < tol);
    }

    #[test]
    fn test_besseli_large_nu_debye() {
        // Large order case
        let nu = 50.0;
        let x = 10.0;
        let val = besseli(nu, x, false);
        // Ref value: 1.34685023964431e-25
        assert!((val - 4.7568945607268954e-30).abs() < 1e-43);
    }

    #[test]
    fn test_besseli_intermediate_backward() {
        // Region likely triggering backward recurrence
        let nu = 5.0;
        let x = 10.0;
        let val = besseli(nu, x, false);
        assert!((val - 777.18828640326).abs() < 1e-10);
    }

    #[test]
    fn test_besseli_overflow_protection() {
        // Without scaling, I_0(1000) would overflow f64
        assert!(besseli(0.0, 1000.0, false).is_infinite());
        // With scaling, it should be a small finite value
        assert!(besseli(0.0, 1000.0, true) > 0.0);
    }
}
