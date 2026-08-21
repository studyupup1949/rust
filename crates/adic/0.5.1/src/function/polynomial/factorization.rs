use crate::{
    local_num::{LocalInteger, LocalOne},
    mapping::Differentiable,
    normed::Normed,
    traits::HasApproximateDigits,
    ZAdic,
};
use super::{euclid, zadic_wrapper, Polynomial};


/// Get the square-free decomposition of a `Polynomial`, unnormalized
pub fn raw_square_free_decomposition<T>(poly: &Polynomial<T>) -> Vec<Polynomial<T>>
where T: Clone + LocalInteger {

    square_free_decomposition(poly)

}


/// Get the square-free decomposition of a `Polynomial`, reduced to primitive monic form at the last step
pub fn primitive_square_free_decomposition<T>(poly: &Polynomial<T>) -> Vec<Polynomial<T>>
where T: Clone + LocalInteger + Normed, T::Unit: LocalOne {

    // Note: we could re-implement the below fn but with `euclid::primitive_pseudo_rem_gcd`.
    // This could avoid some coefficient blowup during computation.
    // However, in practice dividing out the leading unit from the coefficients is often taxing.
    // As a judgement call, we rely on `factor_coefficients` in the intermediate calculation
    //  and then call `primitive_monic` at the end to do this division once.

    square_free_decomposition(poly).into_iter().map(Polynomial::primitive_monic).collect()

}


/// Get the square-free decomposition of a `Polynomial`
fn square_free_decomposition<T>(poly: &Polynomial<T>) -> Vec<Polynomial<T>>
where T: Clone + LocalInteger {

    // `f(x) = a_1 a_2^2 a_3^3 ... a_d^d = (a_2^1 a_3^2 ... a_d^(d-1)) a_1 a_2 a_3 ... a_d = a_0 a_1 a_2 a_3 ... a_d`
    // `f'(x) = (a_2^1 a_3^2 ... a_d^(d-1)) * sum_i a_1 a_2 ... a_{i-1} i a_i' a_{i+1} ... a_d = a_0 sum_i a_1 ... i a_i ... a_d`
    // `f.square_free_decomposition() -> vec![a_1, a_2, a_3, ... a_d]` (truncated at first `1`)

    // a_0 = gcd(f, f'), b_0 = f, d_0 = f'
    // b_{i+1} = b_i / a_i, c_{i+1} = d_i / a_i, d_{i+1} = c_{i+1} - b_{i+1}', a_{i+1} = gcd(b_{i+1}, d_{i+1})
    // repeat until b_i = 1
    // output vec![a1, a2, ... a_k]

    // Let's instead use pseudo-division, specifically the Euclidean algorithm with pseudo-division.

    let mut factors = vec![];

    let mut a = poly.clone();
    let mut b = a.derivative();
    let mut gcd = euclid::factored_pseudo_rem_gcd(a.clone(), b.clone());

    while a.degree().is_some_and(|db| db != 0) {

        // m0 a = q0 gcd
        let (quotient0, _, mult0) = euclid::euclidean_pseudo_div(a.clone(), &gcd);
        // m1 b = q1 gcd
        let (quotient1, _, mult1) = euclid::euclidean_pseudo_div(b.clone(), &gcd);

        a = quotient0.mul_coefficients(mult1);
        b = quotient1.mul_coefficients(mult0) - a.derivative();
        gcd = euclid::factored_pseudo_rem_gcd(a.clone(), b.clone());

        factors.push(gcd.clone());

    }

    // The last factor is a constant; pop it off
    factors.pop();

    // Finally, factor the polynomial coefficients using their GCD
    let factors = factors.into_iter().map(Polynomial::factor_coefficients).collect();

    factors

}


// === ZADIC WORKAROUND ===

/// We need to special-case `ZAdic` because of its "`is_approx_zero`" behavior.
///
/// E.g. `Polynomial` will remove the leading coefficient if it is zero, but in an uncertain case, it will only **approximately** be zero.
/// In future releases, we might aim at this behavior with a trait like `ApproxZero`.
/// For now, we wrap the `ZAdic` in `zadic_wrapper::ZAdicZeroWrapper` to use `is_approx_zero` as `is_local_zero`.
///
/// Note: this may lower the certainty!
/// In particular, if we find there ARE degeneracies, we need to check things at higher precision.
///
/// E.g. 5-adic at precision 4: `(x-1)^2 (x-26) = x^3 + ...4342 x^2 + ...0203 x + ...4344`.
/// This is the same as `(x-199)^3 = (x + ...1244)^3 = x^3 + ...4342 x^2 + ...0203 x + ...4344`.
pub fn zadic_square_free_decomposition(poly: &Polynomial<ZAdic>) -> Vec<Polynomial<ZAdic>> {

    if poly.coefficients().all(HasApproximateDigits::is_certain) {
        return primitive_square_free_decomposition(poly);
    }

    let wrapped_poly = zadic_wrapper::wrap_poly(poly.clone());

    let mut factors = vec![];

    let mut a = wrapped_poly.clone();
    let mut b = a.derivative();
    let mut gcd = euclid::primitive_pseudo_rem_gcd(a.clone(), b.clone());

    while a.degree().is_some_and(|db| db != 0) {

        // m0 a = q0 gcd
        let (quotient0, _, mult0) = euclid::euclidean_pseudo_div(a.clone(), &gcd);
        // m1 b = q1 gcd
        let (quotient1, _, mult1) = euclid::euclidean_pseudo_div(b.clone(), &gcd);

        a = quotient0.mul_coefficients(mult1);
        b = quotient1.mul_coefficients(mult0) - a.derivative();
        gcd = euclid::primitive_pseudo_rem_gcd(a.clone(), b.clone());

        factors.push(gcd.clone());

    }

    // The last factor is a constant; pop it off
    factors.pop();

    factors.into_iter().map(|poly| {
        let unwrapped_poly = Polynomial::new(poly.into_coefficients().map(|z| z.0).collect());
        unwrapped_poly.primitive_monic()
    }).collect()

}
