//! Functions for doing [Hensel lifting](https://en.wikipedia.org/wiki/Hensel%27s_lemma#Hensel_lifting).

use itertools::Itertools;
use num::Zero;
use crate::{
    divisible::{Divisible, Modulus, Prime, PrimePower},
    error::{AdicError, AdicResult},
    local_num::{LocalOne, LocalZero},
    mapping::{Differentiable, Mapping},
    normed::{Normed, UltraNormed, Valuation},
    traits::{AdicPrimitive, CanApproximate, HasApproximateDigits, HasDigits},
    Polynomial, QAdic, Variety, ZAdic,
};


/// Find the variety of roots that solve a polynomial
///
/// See [`Polynomial::variety`] for information on the algorithm.
///
/// # Errors
/// `Polynomial`'s `certainty` is not high enough for desired `precision`
///
/// # Panics
/// Panics if certainty does not behave as expected
pub (crate) fn qadic_polynomial_variety(
    f: Polynomial<QAdic<ZAdic>>,
    precision: isize,
) -> AdicResult<Variety<QAdic<ZAdic>>> {

    let Some(lowest_degree) = f.lowest_degree()
    else {
        return Ok(Variety::empty());
    };
    let p = f.p()?;

    // Gather the trivial roots into a Vec
    let trivial_roots = vec![QAdic::<ZAdic>::zero(p).into_approximation(precision); lowest_degree];

    // If f has nonzero lowest degree, skip past it and add back in at the end
    let coeffs = f.into_coefficients().skip(lowest_degree).collect::<Vec<_>>();

    if coeffs.is_empty() {
        return Ok(Variety::new(trivial_roots));
    }

    // Now we normalize to a polynomial with integer coefficients and the leading coefficient has valuation 0.
    // All roots will then be integers.
    let (integer_coeffs, leading_valuation) = valuation_monic_integer_coefficients(p, coeffs)?;
    let zadic_precision = usize::try_from(precision + leading_valuation).unwrap_or(0);

    let sff = square_free_factorization_loop(&integer_coeffs, zadic_precision)?;

    let mut qadic_roots = trivial_roots;
    for (idx, sf_poly) in sff.into_iter().enumerate() {
        if sf_poly.degree().as_ref().is_some_and(|d| *d > 0) {

            // ===
            // Main ZAdic algorithm
            // ===
            // Two parts:
            // - Find the solutions in F_{p^v}, where v is the max valuation of the derivative for any solution
            // - Lift those solutions up to the desired precision
            let mut nontrivial_roots = initial_variety_estimate(&sf_poly)?;
            increase_precision(&sf_poly, &mut nontrivial_roots, zadic_precision)?;

            // Finally, put the nontrivial roots back in and multiply roots by `p^(-leading_degree)`
            qadic_roots.extend(nontrivial_roots.into_iter().flat_map(
                // Put in a duplicate root for each power in this square-free factor
                |root| std::iter::repeat_n(QAdic::<ZAdic>::new(root, -leading_valuation), idx + 1)
            ));

        }
    }

    Ok(Variety::new(qadic_roots))

}


/// Calculate the number of roots for a given [`Polynomial`]
///
/// # Errors
/// Errors if rootfinding encounters problems, e.g. heavily degenerate roots
pub (crate) fn qadic_polynomial_variety_size(
    f: Polynomial<QAdic<ZAdic>>
) -> AdicResult<usize> {

    let Some(lowest_degree) = f.lowest_degree()
    else {
        return Ok(0);
    };
    let p = f.p()?;

    // If f has nonzero lowest degree, skip past it and add back in at the end
    let coeffs = f.into_coefficients().skip(lowest_degree).collect::<Vec<_>>();

    if coeffs.is_empty() {
        return Ok(lowest_degree);
    }

    // Now we normalize to a polynomial with integer coefficients and the leading coefficient has valuation 0.
    // All roots will then be integers.
    let (integer_coeffs, _) = valuation_monic_integer_coefficients(p, coeffs)?;

    // Estimate the necessary precision to be degree * (max finite coefficient valuation).
    // Then multiply by 2 and add 1, since initial_variety_estimate may need as much again.
    let Some(max_val) = integer_coeffs.iter().filter_map(
        |c| c.valuation().finite()
    ).max() else {
        // Shouldn't get here, would mean the Polynomial is zero
        return Err(AdicError::Severe("Unexpected zero Polynomial during variety_size".to_string()));
    };
    let estimated_precision = 2 * integer_coeffs.len() * max_val + 1;

    let sff = square_free_factorization_loop(&integer_coeffs, estimated_precision)?;

    let mut num_roots = lowest_degree;
    for (idx, sf_poly) in sff.into_iter().enumerate() {
        if sf_poly.degree().as_ref().is_some_and(|d| *d > 0) {

            // Just need to find the solutions in F_{p^v}, where v is the max valuation of the derivative for any solution
            let roots = initial_variety_estimate(&sf_poly)?;
            num_roots += roots.len() * (idx + 1);

        }
    }

    Ok(num_roots)

}


fn valuation_monic_integer_coefficients(p: Prime, coeffs: Vec<QAdic<ZAdic>>) -> AdicResult<(Vec<ZAdic>, isize)> {

    // First we want to convert to a polynomial with `ZAdic` coefficients where the leading degree coefficient is valuation zero.
    // This will guarantee all roots are integers.
    //
    // E.g.
    // 0 == 5x^2 - 6x + 1 -> 0 == 5(y/5)^2 - 6(y/5) + 1 = 1/5 (y^2 - 6y + 5) -> 0 == y^2 - 6y + 5
    // Roots (y - 1)(y - 5) mean the roots for the original are (x - 1/5)(x - 1)
    // Higher valuation:
    // 0 == 25x^2 - 6x + 1 -> 0 == 25(y/25)^2 - 6(y/25) + 1 = 1/25(y^2 - 6y + 25) -> 0 == y^2 - 6y + 25
    //
    // - Divide out the trivial roots
    // - Return if the remainder is constant
    // - Multiply the coefficients by the `prime` until they all have non-negative valuation
    // - Calculate the valuation `v` of the largest degree `d` coefficient
    // - Divide `x` by `p^v`, then factor out `p^(-(d-1) v)`
    // - Multiply each coefficient with degree `n` by `p^((d-n-1) v)`
    // - This gives the leading coefficient a valuation of `0` while leaving the rest nonnegative

    // Adjust the coefficients to be integers with lowest possible valuation
    let Valuation::Finite(lowest_valuation) = coeffs.iter().map(QAdic::<ZAdic>::valuation).min().unwrap_or(0.into()) else {
        return Err(AdicError::Severe("Coefficients have infinite valuation, i.e. they are zero; should not be possible".to_string()));
    };
    let mut integer_coeffs = coeffs.into_iter().map(|coeff| match coeff.into_unit_and_valuation() {
        (Some(unit), Valuation::Finite(val)) => {
            let adjusted_val = usize::try_from(val - lowest_valuation)?;
            let pp = PrimePower::try_from((p, adjusted_val))?;
            Ok(unit * ZAdic::from_prime_power(pp))
        },
        _ => Ok(ZAdic::zero(p)),
    }).collect::<AdicResult<Vec<_>>>()?;

    // Now we need to divide out any valuation, `v`, in the coefficient with the highest degree, `d`.
    // This amounts to multiplying coefficients with lower degree, `n`, by `p^((d-n-1) v)`.
    let Some(Valuation::Finite(leading_val)) = integer_coeffs.last().map(ZAdic::valuation) else {
        return Err(AdicError::Severe("Leading coefficient does not exist or valuation is infinite".to_string()));
    };
    let ileading_val = isize::try_from(leading_val)?;

    // Adjust leading coefficient
    if let Some(leading_coeff) = integer_coeffs.last_mut() {
        if let Some(u) = leading_coeff.unit() {
            *leading_coeff = u;
        } else {
            return Err(AdicError::Severe("Leading coefficient should not be zero".to_string()));
        }
    } else {
        return Err(AdicError::Severe("Leading coefficient should exist".to_string()));
    }
    // Adjust other coefficients
    for (reversed_idx, coeff) in integer_coeffs.iter_mut().rev().skip(1).enumerate() {
        let pp = PrimePower::try_from((p, reversed_idx * leading_val))?;
        *coeff *= ZAdic::from_prime_power(pp);
    }

    Ok((integer_coeffs, ileading_val))

}


fn square_free_factorization_loop(
    valuation_monic_integer_coeffs: &[ZAdic],
    zadic_precision: usize,
) -> AdicResult<Vec<Polynomial<ZAdic>>> {
    const LOOP_CUTOFF: usize = 10;

    // The precise certainty needed is not known until after completing `initial_variety_estimate`.
    // Once that fn completes, we will know the precision needed is the output `certainty` plus `zadic_precision`.
    // But if that fn returns output `certainty` greater than `zadic_precision`, we don't need to increase precision.
    // Therefore, the starting max precision needed is `2 * zadic_precision + 1`.
    // This may be increased over the course of the square-free factorization loop.
    let mut total_precision_needed = 2 * zadic_precision + 1;

    // We want to split into square-free parts and then perform hensel lifting on each separate polynomial.
    // Note: Currently calling a separate ZAdic-specific square free factorization b.c. of problems with is_zero.
    // We should fix that with a more formal solution.
    for _ in (0..LOOP_CUTOFF) {

        // Approximate coefficients so we can use the fast square_free_decomposition.
        // Just approximate the leading coefficient, which we know is a unit

        let mut approx_coeffs = Vec::from(valuation_monic_integer_coeffs);
        if let Some(leading_coeff) = approx_coeffs.last_mut() {
            *leading_coeff = leading_coeff.approximation(total_precision_needed);
        }
        let zadic_poly = Polynomial::<ZAdic>::new(approx_coeffs);

        let sff = zadic_poly.zadic_square_free_decomposition();

        if sff.iter().array_combinations().any(|[factor0, factor1]| !factor0.zadic_gcd(factor1).is_constant()) {
            // If any gcd between square-free factors is non-constant, repeat the loop with double precision
            total_precision_needed = 2 * total_precision_needed;
        } else if sff.len() * zadic_precision >= total_precision_needed {
            // If precision has been lost from the factorization, repeat the loop with more precision to make up for it
            total_precision_needed = 2 * sff.len() * zadic_precision + 1;
        } else {
            return Ok(sff);
        }

    }

    let err_poly = Polynomial::new(Vec::from(valuation_monic_integer_coeffs));
    let err_msg = format!("Square-free factorization of approximate Polynomial failed: {err_poly}");
    Err(AdicError::Severe(err_msg))

}


// Find the solutions in F_{p^v}, where v is the max valuation of the derivative for any solution
fn initial_variety_estimate(f: &Polynomial<ZAdic>) -> AdicResult<Vec<ZAdic>> {

    let p = f.p()?;
    let derivative = f.derivative();

    // We need to find roots in F_p and calculate the valuation of their derivative
    // If the valuation is larger than the digit we've calculated so far, search F_{p^2}, F_{p^3} etc
    let mut varieties = vec![];
    for d in p.digit_range() {
        let var = ZAdic::new_approx(p, 1, vec![d]);
        if f.eval(var.clone())?.digits().all(|a| a.is_zero()) {
            varieties.push(var);
        }
    }

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
        let (mut unsolved_vars, mut solved_vars) = (vec![], vec![]);
        for var in varieties {

            let deriv = derivative.eval(var.clone())?;
            let d_val = deriv.valuation();
            let Valuation::Finite(test_val) = d_val else {
                panic!("Derivative is exactly zero during Hensel lift; not sure what to do");
            };

            if test_val > digit_val {
                unsolved_vars.push(var);
            } else {
                solved_vars.push(var);
            }

        }
        varieties = unsolved_vars;
        final_varieties = [final_varieties, solved_vars].concat();

        // If no derivative had a higher valuation, we don't need to search in higher powers of p
        // We can proceed to the iterative portion of the algorithm
        if varieties.is_empty() {
            break;
        }

        // Otherwise, add two digits to each and look for solutions
        digit_val += 1;
        let mut updated_vars = vec![];
        for var in varieties {

            // Add digit_val test digits onto the end of variety and check for solution mod 2*digit_val-1
            // If the extended variety still satisfies the polynomial, add it to the list

            for (d1, d2) in p.digit_range().cartesian_product(p.digit_range()) {
                let new_var = ZAdic::new_approx(
                    var.p(), 2*digit_val + 1, var.digits().chain([d1, d2]).collect()
                );
                // TODO: This is WAY too generous; we should be filtering out more solutions than this.
                // We should instead check the valuation is INCREASING
                if f.eval(new_var.clone())?.digits().all(|a| a.is_zero()) {
                    updated_vars.push(new_var);
                }
            }

        }
        varieties = updated_vars;

        // If we are calculating too many digits this function's run time can easily blow up
        // Therefore we're setting a cutoff based on p
        let cutoff: u32 = 100_000_000;
        if digit_val > cutoff.ilog(u32::from(p)).try_into()? {
            return Err(AdicError::Severe("Heavily degenerate root suspected".to_string()))
        }

    }

    Ok(final_varieties.into_iter()
        // Cut off the digits past v+1
        // They are a distraction and need to be rebuilt either in the next loop or iteratively below
        .map(|var| {
            let Valuation::Finite(full_certainty) = var.certainty() else {
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
fn increase_precision(f: &Polynomial<ZAdic>, varieties: &mut Vec<ZAdic>, precision: usize) -> AdicResult<()> {

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
    // Then
    // c p^k + a_k p^k f'(r_{k-1}) === 0 mod p^{k+1}
    // where c = f(r_{k-1}) / p^k
    //
    // This can be expressed as
    // c + a_k f'(r_{k-1}) / p^k === 0 mod p

    // This final modulus can be used to find a_k

    let p = f.p()?;
    for variety in varieties {

        let v = match variety.certainty() {
            Valuation::PosInf => panic!("Should not have infinite certainty"),
            Valuation::Finite(0) => panic!("Should not have zero certainty"),
            Valuation::Finite(cert) => cert,
        };

        if v > precision {
            variety.set_certainty(precision.into());
        }

        // For each variety, increase the precision on the result until it matches the desired precision
        for k in v..precision {

            let mut this_variety = variety.clone();
            this_variety.set_certainty(Valuation::Finite(v+k));

            let mut output = this_variety.local_zero();
            let mut input_to_the_n = this_variety.local_one();
            for coefficient in f.coefficients() {
                output = output + input_to_the_n.clone() * coefficient.clone();
                input_to_the_n = input_to_the_n * this_variety.clone();
            }

            // Calculate c = f(r_{k-1}) / p^k
            let c = f.eval(this_variety.clone())?;
            let c_d = c.digit(k+v-1)?;
            // Calculate f'(r_{k-1})
            let derivative = f.derivative().eval(this_variety.clone())?;
            let deriv_d = match derivative.into_unit() {
                Some(u) => u.digit0()?,
                None => 0,
            };

            // Check for values where the following is true:
            // c + a_k f'(r_{k-1}) === 0 mod p
            let a_k = p.mod_neg((p.mod_inv(deriv_d) * c_d) % p);
            variety.push_digit(a_k)?;

        }

    }

    Ok(())

}
