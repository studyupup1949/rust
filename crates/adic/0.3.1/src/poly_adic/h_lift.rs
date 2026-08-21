//! Functions for doing Hensel lift

use std::iter::repeat_n;
use itertools::Itertools;
use num::{traits::Pow, Zero};
use crate::{
    uadic, zadic_approx,
    AdicError, AdicInteger, UAdic, ZAdic,
    ZAdicValuation, ZAdicVariety,
};


const GENERAL_POLYNOMIAL_ERROR_MSG: &str = "We do not currently support solving general polynomial, just n-th roots";


/// Newton/Hensel method for deriving a variety for an nth root ([`AdicInteger::nth_root`])
///
/// # Errors
/// Errors if:
/// 1. n == 0
/// 2. `precision` is not high enough (roughly, `adic_int.certainty() >= n * precision`)
pub fn nth_root<AdicInt>(
    adic_int: &AdicInt, n: u32, precision: usize,
) -> Result<ZAdicVariety, AdicError>
where AdicInt: AdicInteger {

    let p = adic_int.p();
    let ns = usize::try_from(n)?;

    // 0-th root is ill defined
    if n == 0 { return Err(AdicError::IllDefined("Cannot take the 0-th root".to_string())); }

    let ZAdicValuation::Finite(m) = adic_int.valuation() else {
        // Infinite valuation means 0. The root of 0 is always 0; return 1 solution.
        return Ok(ZAdicVariety::new(p, vec![ZAdic::zero(p)]))
    };

    // If m is not proportional to n then there are no n-th roots
    if m % ns != 0 { return Ok(ZAdicVariety::empty(p)); }

    // nth_root(adic_int) will have valuation of (valuation / n) and max certainty of ((certainty + valuation) / n)
    let output_valuation = m / ns;
    if let ZAdicValuation::Finite(c) = adic_int.certainty() {
        let max_output_certainty = (m + c) / ns;
        if max_output_certainty < precision {
            let msg = format!(
                "Could not perform nth_root on Adic with n {n}, valuation {m}, certainty {c} to precision {precision}"
            );
            println!("Attempted nth_root with bad precision");
            println!("{}", msg.clone());
            return Err(AdicError::InappropriatePrecision(msg));
        }
    };

    let min_input_certainty = precision * ns;
    let a = adic_int.truncation(min_input_certainty);

    // Divide out the powers of p and multiply them back at the end
    // Find b and m such that a = b * p^m
    // I.e. if a = b * p^m then nth_root(a) = p^(m/n) * nth_root(b)
    // a might not be a unit, but b is (meaning the first digit is nonzero and it's an invertible integer)
    let b = a.quotient(m);

    let full_varieties = if n % p != 0 {
        simple_nth_root(&b, n, precision)?
    } else {
        general_nth_root(&b, n, precision)?
    };

    let k = output_valuation;
    let adic_roots = full_varieties.into_iter()
        // I don't think we can correctly count degenerate roots, so remove duplicates
        .unique()
        // Multiply the k = a / b back in
        .map(|variety| {
            ZAdic::new_approx(
                p, precision,
                repeat_n(0, k).chain(variety.into_digits()).take(precision).collect()
            )
        })
        .collect();

    Ok(ZAdicVariety::new(p, adic_roots))

}

/// Perform the n-th root with the simple Hensel lift, only valid if n % p == 0
fn simple_nth_root(b: &UAdic, n: u32, precision: usize) -> Result<Vec<ZAdic>, AdicError> {
    let p = b.p();

    // Find the number of varieties in Z_p by looking for the number in F_p
    let b0 = b.zeroth_digit().unwrap();
    let mut varieties = Vec::new();
    for num in 0..p {
        let n0 = uadic!(p, [num]).pow(n).zeroth_digit().unwrap();
        if n0 == b0 {
            varieties.push(zadic_approx!(p, 1, [num]));
        }
    }

    for variety in &mut varieties {
        for k in 1..precision {

            // r_k+1 = r_k - f(r_k)/f'(r_k) = r_k - (r_k^n - b) / (n * r_k^{n-1})
            // => f(r_k) / p^k + a_k+1 * f'(r_k) === 0 mod p
            // => (r_k^n - b) / p^k + a_k+1 * (n * r_k^{n-1}) === 0 mod p
            //
            // r_k^n / p^k + a_k+1 * (n * r_k^{n-1}) === b / p_k mod p

            // Turn variety to UAdic
            let u_var = variety.truncation_to_uadic()?;
            // 0-th digit of (n-1)-th power
            let var_n_1 = u_var.clone().pow(n-1);
            let deriv_0th_digit = n * var_n_1.zeroth_digit().unwrap_or(0);
            // k-th digit of n-th power
            let var_n = var_n_1 * u_var;
            let var_n_kth_digit = var_n.digit(k)?;
            // k-th digit of b
            let b_kth_digit = b.digit(k)?;

            // See which digit fits the modular equation
            for d in 0..p {
                let left_side = var_n_kth_digit + d * deriv_0th_digit;
                let right_side = b_kth_digit;
                if left_side % p == right_side {
                    variety.push_digit(d)?;
                }
            }

        }
    }

    Ok(varieties)

}

/// Perform the n-th root with the generalized Hensel lift (look ahead based on derivative valuation)
fn general_nth_root(b: &UAdic, n: u32, precision: usize) -> Result<Vec<ZAdic>, AdicError> {
    let p = b.p();

    // Two parts:
    // - Find the solutions in F_{p^v}, where v is the max valuation of the derivative for any solution
    // - Lift those solutions up to the desired precision

    // We need to find roots in F_p and calculate the valuation of their derivative
    // If the valuation is larger than the digit we've calculated so far, search F_{p^2}, F_{p^3} etc
    let adic_b = ZAdic::from(b.clone());
    let adic_n = ZAdic::from_u32(p, n);
    let mut v = 0;
    let mut varieties: Vec<ZAdic>;
    loop {

        // Each loop is per-valuation, increasing the power of p we compare the numbers at
        // v represents the digit we know we can add
        // We have to compare digits at least up to 2*v+1

        // Restart the variety calculation to make sure we don't miss any varieties
        varieties = vec![];

        // For each "partial variety" we've found so far, fill out digits to 2*v
        // If the extended variety still satisfies the polynomial, add it to the list
        let compare_v = 2*v+1;
        for trial_digits in repeat_n(0..p, compare_v).multi_cartesian_product() {
            let mut extended_var = zadic_approx!(p, 0, []);
            for digit in trial_digits {
                extended_var.push_digit(digit).unwrap();
            }
            let poly = extended_var.pow(n) - adic_b.clone();
            if poly.into_digits().all(|d| d.is_zero()) {
                varieties.push(extended_var);
            }
        }

        // If the valuation is already at precision, just return
        if v > precision {
            return Ok(varieties);
        }

        // Now for each variety, calculate the derivative and see if the valuation is larger
        let mut flag_dirty_v = false;
        for var in &varieties {
            let deriv = adic_n.clone() * var.pow(n-1);
            let d_val = deriv.valuation();
            if let ZAdicValuation::Finite(test_val) = d_val {
                if test_val > v {
                    flag_dirty_v = true;
                    v = test_val;
                }
            } else {
                return Err(AdicError::Severe("Derivative is exactly zero during Hensel lift; not sure what to do".to_string()));
            }
        }

        // If no derivative had a higher valuation, we don't need to search in higher powers of p
        // We can proceed to the iterative portion of the algorithm
        if !flag_dirty_v { break; }

    }

    varieties = varieties.into_iter()
        // Cut off the digits past v+1
        // They are a distraction and need to be rebuilt either in the next loop or iteratively below
        .map(|var| var.into_approximation(v+1))
        // Eliminate duplicates
        .unique()
        .collect();

    // Now, let's do the Newton approximation
    // This is a lot like the simple_nth_root version, but we have to look ahead several digits
    // The "look ahead" is v, the highest valuation of a derivative in our previous step
    // Luckily, this v does not change during Hensel lifting

    // r_k = a_0 + a_1 p + ... + a_k p^k
    // f(r_{k-1}) === 0 mod p^{k+v}
    // f(r_k) === 0 mod p^{k+v+1}
    // f(r_k) = f(r_{k-1} + a_k p^k) === f(r_{k-1}) + a_k p^k f'(r_{k-1}) mod p^{k+v+1}
    // Find c = f(r_{k-1}) / p^{k+v}
    // c p^{k+v} + a_k p^k f'(r_{k-1}) === 0 mod p^{k+v+1}
    // c + a_k f'(r_{k-1}) / p^v === 0 mod p

    // This final modulus can be used to find a_k

    for variety in &mut varieties {

        for k in (v+1)..precision {

            // f(s_{k-1})/p^k + f'(s_{k-1}) D_k === 0 mod p^{v+1}
            // D_k = (a_k - b_k) + (c_{k+1} - b_{k+1}) p + ... + (c_{k+v-1} - b_{k+v-1}) p^{v-1} + c_{k+v} p^v

            // For n-th root, f(x) = x^n - b and f'(x) = n x^{n-1}
            let mut rk = variety.clone();
            rk.set_certainty(ZAdicValuation::Finite(k+v+1));
            let pm1 = rk.pow(n-1);
            let poly = pm1.clone() * rk - adic_b.clone();
            let poly_d = poly.digit(k+v)?;
            let deriv = adic_n.clone() * pm1;
            let deriv_d = deriv.digit(v)?;
            for new_d in 0..p {
                if (poly_d + deriv_d * new_d) % p == 0 {
                    variety.push_digit(new_d)?;
                    break;
                }
            }

        }

    }

    Ok(varieties)

}


/// Utility method to calculate roots of unity for given p
///
/// # Errors
/// Returns any errors that [`nth_root`] returns
///
/// ```
/// # use adic::{roots_of_unity, zadic_variety};
/// assert_eq!(
///     Ok(zadic_variety!(2, 6, [[1, 0, 0, 0, 0, 0], [1, 1, 1, 1, 1, 1]])),
///     roots_of_unity(2, 6)
/// );
/// assert_eq!(
///     Ok(zadic_variety!(3, 6, [[1, 0, 0, 0, 0, 0], [2, 2, 2, 2, 2, 2]])),
///     roots_of_unity(3, 6)
/// );
/// assert_eq!(
///     Ok(zadic_variety!(5, 6, [
///         [1, 0, 0, 0, 0, 0],
///         [2, 1, 2, 1, 3, 4],
///         [3, 3, 2, 3, 1, 0],
///         [4, 4, 4, 4, 4 ,4],
///     ])),
///     roots_of_unity(5, 6)
/// );
/// ```
pub fn roots_of_unity(p: u32, precision: usize) -> Result<ZAdicVariety, AdicError> {
    let n = if p == 2 { 2 } else { p-1 };
    ZAdic::one(p).nth_root(n, precision)
}


/// Newton/Hensel method for deriving digits.
///
/// See ([`AdicInteger::nth_root`]) for information on the algorithm.
/// Just abstract the f(x) to be a polynomial (or even Taylor series).
///
/// # Errors
/// Errors if:
/// 1. Polynomial is not of the form `x^n - a`
/// 2. n == 0
/// 3. `precision` is not high enough (roughly, `a[0].certainty() >= n * precision`)
///
/// # Panics
/// Panics if input coefficients have length zero
///
/// <div class="warning">
///
/// Currently, this returns an error if input is not in the form [a, 0, 0, ..., 1], i.e. an n-th root.
/// We do not handle general polynomials at this point.
/// This just serves as a passthrough to [`nth_root`]
///
/// </div>
pub fn polynomial_variety<AdicInt>(p: u32, a: &[AdicInt], precision: usize) -> Result<ZAdicVariety, AdicError>
where AdicInt: AdicInteger + std::ops::Neg, <AdicInt as std::ops::Neg>::Output: AdicInteger {
    let nonzero_coefficients = a.iter().filter(|&a_n| *a_n != AdicInt::zero(p)).count();
    if nonzero_coefficients > 2
    || *a.iter().last().unwrap() != AdicInt::one(p)
    || (nonzero_coefficients > 1 && a[0] == AdicInt::zero(p)) {
        Err(AdicError::NotImplemented(GENERAL_POLYNOMIAL_ERROR_MSG.to_string()))
    } else {
        let adic_int = &(-a[0].clone());
        let n = u32::try_from(a.len())? - 1;
        nth_root(adic_int, n, precision)
    }
}
