//! A-priori height bounds (in bits) used to provision an adaptive [`crate::basis::Basis`].
//!
//! These turn "might overflow" into "provision exactly enough channels, then prove
//! it fit." Every bound here is an *upper* bound on the bit-height of the result, so
//! `Basis::with_bits(bound + slack)` is guaranteed large enough for reconstruction.

use num_bigint::BigInt;
use num_traits::Zero;

/// Bit-length of an integer (0 has length 0).
pub fn integer_bits(n: &BigInt) -> u64 {
    n.bits()
}

/// Exact bound for a product of integers: the sum of their bit-lengths (+1 slack
/// for the leading carry).
pub fn product_bits(values: &[BigInt]) -> u64 {
    values.iter().map(|v| v.bits()).sum::<u64>() + 1
}

/// `floor` infinity-norm in bits: the largest coefficient bit-length.
fn inf_norm_bits(coeffs: &[BigInt]) -> u64 {
    coeffs.iter().map(|c| c.bits()).max().unwrap_or(0)
}

/// Upper bound on `log2 ‖f‖₂`, using `‖f‖₂ ≤ sqrt(n+1)·‖f‖∞`.
fn two_norm_log2_upper(coeffs: &[BigInt]) -> f64 {
    let n = coeffs.iter().filter(|c| !c.is_zero()).count().max(1);
    let inf = inf_norm_bits(coeffs) as f64;
    0.5 * (n as f64 + 1.0).log2() + inf
}

/// Hadamard/Mahler bound on `Res(f, g)` in bits:
/// `‖f‖₂^deg g · ‖g‖₂^deg f` (with slack for the sign bit and rounding).
pub fn resultant_bits(f: &[BigInt], g: &[BigInt]) -> u64 {
    let deg_f = f.len().saturating_sub(1) as f64;
    let deg_g = g.len().saturating_sub(1) as f64;
    let log_f = two_norm_log2_upper(f);
    let log_g = two_norm_log2_upper(g);
    let bits = deg_g * log_f + deg_f * log_g;
    bits.ceil() as u64 + 4
}

/// Hadamard bound on an `n×n` integer determinant in bits: `∏ row-2-norms`.
pub fn determinant_bits(rows: &[Vec<BigInt>]) -> u64 {
    let mut total = 0.0f64;
    for row in rows {
        total += two_norm_log2_upper(row);
    }
    total.ceil() as u64 + 2
}

/// Mignotte bound on the coefficients of any integer factor `g | f` in bits:
/// `‖g‖∞ ≤ 2^deg g · ‖f‖₂` (we use `deg f` as the worst-case factor degree).
pub fn mignotte_bits(f: &[BigInt]) -> u64 {
    let deg = f.len().saturating_sub(1) as u64;
    let log_f2 = two_norm_log2_upper(f).ceil() as u64;
    deg + log_f2 + 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_is_sum_of_bits() {
        let vals = vec![BigInt::from(255), BigInt::from(255)];
        // 255*255 = 65025 needs 16 bits; bound is 8+8+1 = 17 ≥ 16
        assert!(product_bits(&vals) >= 16);
    }

    #[test]
    fn resultant_bound_is_positive() {
        let f = vec![BigInt::from(1), BigInt::from(0), BigInt::from(-2)]; // x^2 - 2
        let g = vec![BigInt::from(1), BigInt::from(0), BigInt::from(-3)]; // x^2 - 3
        assert!(resultant_bits(&f, &g) > 0);
    }
}
