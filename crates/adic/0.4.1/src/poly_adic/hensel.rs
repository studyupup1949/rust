//! Functions for doing [Hensel lifting](https://en.wikipedia.org/wiki/Hensel%27s_lemma#Hensel_lifting).

use std::iter::repeat_n;
use itertools::Itertools;
use num::Zero;
use crate::{
    AdicApproximate, AdicError, AdicInteger, AdicNumber, AdicPolynomial, AdicResult, AdicSized, AdicValuation,
    Divisible, HasDigits, ZAdic, ZAdicVariety,
};


/// Find the variety of roots that solve a polynomial
///
/// See [`AdicPolynomial::variety`] for information on the algorithm.
///
/// # Errors
/// 1. `AdicPolynomial`'s `certainty` is not high enough for desired `precision`
/// 2. A degenerate root is suspected (multiplicity not yet supported but on its way)
///
/// # Panics
/// Panics if certainty does not behave as expected
pub (crate) fn polynomial_variety<T>(f: AdicPolynomial<T>, precision: usize) -> AdicResult<ZAdicVariety>
where T: AdicNumber, AdicPolynomial<T>: Into<AdicPolynomial<ZAdic>> {

    let p = f.p();
    let Some(lowest_degree) = f.lowest_degree()
    else {
        return Ok(ZAdicVariety::empty(p));
    };
    let f: AdicPolynomial<ZAdic> = f.into();

    // If f has nonzero lowest degree, skip past it and add back in at the end
    let f = f.into_coefficients().skip(lowest_degree).collect();
    let f = AdicPolynomial::<ZAdic>::new(p, f);

    if f.degree().is_none_or(|d| d < 1) {
        return Ok(ZAdicVariety::new(p, vec![ZAdic::zero(p).into_approximation(precision); lowest_degree]));
    }

    // Two parts:
    // - Find the solutions in F_{p^v}, where v is the max valuation of the derivative for any solution
    // - Lift those solutions up to the desired precision
    let varieties = initial_variety_estimate(&f)?;

    let mut varieties = increase_precision(&f, varieties, precision)?;

    if lowest_degree > 0 {
        varieties.extend(repeat_n(ZAdic::zero(p).into_approximation(precision), lowest_degree));
    }

    Ok(varieties)

}

/// Calculate the number of roots for a given [`AdicPolynomial`]
///
/// # Errors
/// Errors if rootfinding encounters problems, e.g. heavily degenerate roots
pub (crate) fn variety_size<T>(f: &AdicPolynomial<T>) -> AdicResult<usize>
where T: AdicNumber, AdicPolynomial<T>: Into<AdicPolynomial<ZAdic>> {

    let p = f.p();
    let Some(lowest_degree) = f.lowest_degree()
    else {
        return Ok(0);
    };

    let f: AdicPolynomial<ZAdic> = f.clone().into();

    // If f has nonzero lowest degree, skip past it and add back in at the end
    let f = f.into_coefficients().skip(lowest_degree).collect();
    let f = AdicPolynomial::<ZAdic>::new(p, f);

    if f.degree().is_none_or(|d| d < 1) {
        return Ok(lowest_degree);
    }

    Ok(initial_variety_estimate(&f)?.len() + lowest_degree)

}


/// Find the variety of n-th roots of an adic integer
///
/// See [`AdicInteger::nth_root`] for information on the algorithm.
///
/// # Errors
/// 1. `AdicInteger` `certainty` is not high enough for desired `precision`
/// 2. n == 0
///
/// # Panics
/// Panics if certainty does not behave as expected
pub (crate) fn nth_root<T>(
    adic_int: &T, n: u32, precision: usize,
) -> AdicResult<ZAdicVariety>
where T: Clone + Into<ZAdic> {
    let adic_int: ZAdic = adic_int.clone().into();
    AdicPolynomial::nth_root_polynomial(adic_int.clone(), n).variety(precision)
}

/// Calculate the number of nth roots for an [`AdicInteger`]
///
/// # Errors
/// Errors if rootfinding encounters problems, e.g. heavily degenerate roots
pub (crate) fn num_nth_roots<T>(adic_int: &T, n: u32) -> AdicResult<usize>
where T: Clone + Into<ZAdic> {
    variety_size(&AdicPolynomial::nth_root_polynomial(adic_int.clone().into(), n))
}


// Find the solutions in F_{p^v}, where v is the max valuation of the derivative for any solution
fn initial_variety_estimate(f: &AdicPolynomial<ZAdic>) -> AdicResult<Vec<ZAdic>> {

    let p = f.p();
    let derivative = f.derivative();

    // We need to find roots in F_p and calculate the valuation of their derivative
    // If the valuation is larger than the digit we've calculated so far, search F_{p^2}, F_{p^3} etc
    let mut varieties = p.digit_range().filter_map(|d| {
        let var = ZAdic::new_approx(p, 1, vec![d]);
        if f.evaluate(&var).into_digits().all(|a| a.is_zero()) {
            Some(var)
        } else {
            None
        }
    }).collect::<Vec<_>>();

    // Then loop, first checking the derivatives for each solution.
    // If they have low enough valuation, we have distinguished unique simple roots.
    // Otherwise, proceed by adding two digits at a time to the varieties and looking for solutions.
    // Strip off the variety members that satisfy the low derivative valuation as we go.
    let mut digit_val = 0;
    let mut final_varieties = vec![];
    loop {

        // Each loop is per-valuation, increasing the power of p we compare the numbers at
        // digit_val represents the digit we know we can add
        // We have to compare digits at least up to 2*v+1

        // Calculate the derivative for each variety
        // Peel off roots that satisfy the derivative inequality and continue with the rest.
        let (unsolved_vars, solved_vars) = varieties.into_iter().partition(|var| {

            let deriv = derivative.evaluate(var);
            let d_val = deriv.valuation();
            let AdicValuation::Finite(test_val) = d_val else {
                panic!("Derivative is exactly zero during Hensel lift; not sure what to do");
            };

            test_val > digit_val

        });
        varieties = unsolved_vars;
        final_varieties = [final_varieties, solved_vars].concat();

        // If no derivative had a higher valuation, we don't need to search in higher powers of p
        // We can proceed to the iterative portion of the algorithm
        if varieties.is_empty() {
            break;
        }

        // Otherwise, add two digits to each and look for solutions
        digit_val += 1;
        varieties = varieties.into_iter().flat_map(|var| {

            // Add digit_val test digits onto the end of variety and check for solution mod 2*digit_val-1
            // If the extended variety still satisfies the polynomial, add it to the list

            let mut updated_vars = vec![];
            for (d1, d2) in p.digit_range().cartesian_product(p.digit_range()) {
                let new_var = ZAdic::new_approx(
                    var.p(), 2*digit_val + 1, var.digits().chain([d1, d2]).collect()
                );
                // TODO: This is WAY too generous; we should be filtering out more solutions than this.
                // We should instead check the valuation is INCREASING
                if f.evaluate(&new_var).into_digits().all(|a| a.is_zero()) {
                    updated_vars.push(new_var);
                }
            }
            updated_vars.into_iter()

        }).collect();

        // If we are calculating too many digits this function's run time can easily blow up
        // Therefore we're setting a cutoff based on p
        let cutoff: u32 = 10000;
        if digit_val > cutoff.ilog(u32::from(p)).try_into()? {
            return Err(AdicError::NotImplemented("Degenerate root suspected; multiplicity not yet supported".to_string()))
        }

    }

    Ok(final_varieties.into_iter()
        // Cut off the digits past v+1
        // They are a distraction and need to be rebuilt either in the next loop or iteratively below
        .map(|var| {
            let AdicValuation::Finite(full_certainty) = var.certainty() else {
                panic!("Should not have infinite certainty during initial variety estimate");
            };
            let stripped_certainty = full_certainty.div_ceil(2);
            var.into_approximation(stripped_certainty)
        })
        // Remove duplicates; we will have multiple corresponding to the same root otherwise
        .unique()
        .collect())

}

// This function takes the initial variety estimates and iteratively increases their precision until it matches the given parameter `precision`
fn increase_precision(f: &AdicPolynomial<ZAdic>, mut varieties: Vec<ZAdic>, precision: usize) -> AdicResult<ZAdicVariety> {

    // Now, let's do the Newton approximation
    // `v` is the highest valuation of a derivative in our previous step
    // Luckily, this v does not change during Hensel lifting
    // Note, this only works if the derivative is not exactly zero (non-degenerate)

    // The estimate of the polynomial variety to the k-th iteration is given by:
    // r_k = a_0 + a_1 p + ... + a_k p^k
    //
    // This can be equivalently stated:
    // r_k = r_{k-1} + a_k p^k
    //
    // f(r_{k-1}) === 0 mod p^{k}
    // f(r_k) === 0 mod p^{k+1}
    //
    // Looking at the Taylor series expansion for f, up to the linear term, we get
    // f(r_k) = f(r_{k-1} + a_k p^k) === f(r_{k-1}) + a_k p^k f'(r_{k-1}) mod p^{k+1}
    //
    // Then dividing this by p^k we can express this as
    // c p^k + a_k p^k f'(r_{k-1}) === 0 mod p^{k+1}
    // where c = f(r_{k-1}) / p^k
    //
    // This can be expressed as
    // c + a_k f'(r_{k-1}) / p^v === 0 mod p

    // This final modulus can be used to find a_k

    let p = f.p();
    for variety in &mut varieties {

        let v = match variety.certainty() {
            AdicValuation::PosInf => panic!("Should not have infinite certainty"),
            AdicValuation::Finite(0) => panic!("Should not have zero certainty"),
            AdicValuation::Finite(cert) => cert,
        };

        if v > precision {
            variety.set_certainty(precision.into());
        }

        // For each variety, increase the precision on the result until it matches the desired precision
        for k in v..precision {

            let mut this_variety = variety.clone();
            this_variety.set_certainty(AdicValuation::Finite(k+v+1));

            // Calculate c = f(r_{k-1}) / p^k
            let c = f.evaluate(&this_variety);
            let c_d = c.digit(k+v-1)?;
            // Calculate f'(r_{k-1})
            let derivative = f.derivative().evaluate(&this_variety);
            let deriv_d = match derivative.into_unit() {
                Some(u) => u.digit0()?,
                None => 0,
            };

            // Check for values where the following is true:
            // c + a_k f'(r_{k-1}) === 0 mod p
            for a_k in p.digit_range() {
                let val = c_d + a_k * deriv_d;
                if val % p == 0 {
                    variety.push_digit(a_k)?;
                }
            }

        }

    }

    Ok(ZAdicVariety::new(p, varieties))

}
