//! Functions for doing Hensel lift

use std::iter::{once, repeat, repeat_n};
use itertools::Itertools;
use num::{integer::binomial, traits::Pow};
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
/// 2. `precision` is not high enough (roughly, `self.certainty() >= n * precision`)
pub fn nth_root<AdicInt>(
    adic_int: &AdicInt, n: u32, precision: u32
) -> Result<ZAdicVariety, AdicError>
where AdicInt: AdicInteger {

    let p = adic_int.p();

    // 0-th root is ill defined
    if n == 0 { return Err(AdicError::IllDefined("Cannot take the 0-th root".to_string())); }

    let ZAdicValuation::Finite(m) = adic_int.valuation() else {
        // Infinite valuation means 0. The root of 0 is always 0; return 1 solution.
        return Ok(ZAdicVariety::new(p, vec![ZAdic::zero(p)]))
    };

    // If m is not proportional to n then there are no n-th roots
    if m % n != 0 { return Ok(ZAdicVariety::empty(p)); }

    // nth_root(adic_int) will have valuation of (valuation / n) and max certainty of ((certainty + valuation) / n)
    let output_valuation = m / n;
    if let ZAdicValuation::Finite(c) = adic_int.certainty() {
        let max_output_certainty = (m + c) / n;
        if max_output_certainty < precision {
            let msg = format!(
                "Could not perform nth_root on Adic with n {n}, valuation {m}, certainty {c} to precision {precision}"
            );
            println!("Attempted nth_root with bad precision");
            println!("{}", msg.clone());
            return Err(AdicError::InappropriatePrecision(msg));
        }
    };

    let min_input_certainty = precision * n;
    let a = adic_int.truncation(min_input_certainty as usize);

    // Divide out the powers of p and multiply them back at the end
    // Find b and m such that a = b * p^m
    // I.e. if a = b * p^m then nth_root(a) = p^(m/n) * nth_root(b)
    // a might not be a unit, but b is (meaning the first digit is nonzero and it's an invertible integer)
    let b = UAdic::new(p, a.into_digits().skip(m as usize).collect());

    let full_varieties = if n % p != 0 {
        simple_nth_root(&b, n, precision)?
    } else {
        general_nth_root(&b, n, precision)?
    };

    let k = output_valuation as usize;
    let adic_roots = full_varieties.into_iter()
        // I don't think we can correctly count degenerate roots, so remove duplicates
        .unique()
        // Multiply the k = a / b back in
        .map(|variety| {
            ZAdic::new_approx(
                p, precision,
                repeat_n(0, k).chain(variety.into_digits()).take(precision as usize).collect()
            )
        })
        .collect();

    Ok(ZAdicVariety::new(p, adic_roots))

}

/// Perform the n-th root with the simple Hensel lift, only valid if n % p == 0
fn simple_nth_root(b: &UAdic, n: u32, precision: u32) -> Result<Vec<ZAdic>, AdicError> {
    let p = b.p();

    // Find the number of varieties in Z_p by looking for the number in F_p
    // TODO: This is only really correct for small n; we need to use the generalized
    //  Hensel lemma to more generally find all varieties
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

/// Perform the n-th root with the generalized Hensel lift (all derivatives instead of just first)
fn general_nth_root(b: &UAdic, n: u32, precision: u32) -> Result<Vec<ZAdic>, AdicError> {
    let p = b.p();

    // Two parts:
    // - Find the solutions in F_{p^v}, where v is the max valuation of the derivative for any solution
    // - Lift those solutions up to the desired precision

    // We need to find roots in F_p and calculate the valuation of their derivative
    // If the valuation is larger than the digit we've calculated so far, search F_{p^2}, F_{p^3} etc
    let mut max_val = 0;
    let mut v: u32 = 0;
    let mut varieties = vec![zadic_approx!(p, 0, [])];
    loop {

        // Each loop is per-valuation, increasing the power of p we compare the numbers at
        // v represents the digit we know we can add
        // We have to compare digits at least up to 2*v+1

        // For each "partial variety" we've found so far, fill out digits to 2*v
        // If the extended variety still satisfies the polynomial, add it to the list
        let compare_v = 2*v+1;
        varieties = varieties.clone().into_iter().flat_map(|zvar| {
            let mut new_vars = Vec::new();
            for trial_digits in repeat_n(0..p, v as usize + 1).multi_cartesian_product() {
                let mut extended_var = zvar.clone();
                for digit in trial_digits {
                    extended_var.push_digit(digit).unwrap();
                }
                extended_var.set_certainty(ZAdicValuation::Finite(compare_v));
                let powed_var = extended_var.pow(n);
                let b_digits = b.digits().chain(repeat(&0));
                if powed_var.digits().zip(b_digits).take(compare_v as usize).all(|(pvd, bd)| pvd == bd) {
                    new_vars.push(extended_var);
                }
            }
            new_vars
        }).collect();

        v += 1;

        // Cut off the digits past v
        // They are a distraction and need to be rebuilt either in the next loop or iteratively below
        varieties.iter_mut().for_each(|var| var.set_certainty(ZAdicValuation::Finite(v)));

        // Now for each variety, calculate the derivative and see if the valuation is larger
        for var in &varieties {
            let deriv = ZAdic::exact_from_integer(p, n) * var.pow(n-1);
            let d_val = deriv.valuation();
            if let ZAdicValuation::Finite(test_val) = d_val {
                if test_val > max_val {
                    max_val = test_val;
                }
            }
        }

        // If the valuation is already at precision, just return
        if v >= precision {
            return Ok(varieties);
        }

        // If the valuation is past max_val, we don't need to search in higher powers of p
        // We can proceed to the iterative portion of the algorithm
        if v > max_val { break; }

    }

    // Most resources at this point suggest a different algorithm
    // Essentially, you do the Newton approximation, but you're allowed to change previous digits
    // However, it seems like below is better, using more derivatives instead
    // The "look ahead" is k+v

    // f(r_k) = f(r_{k-1} + a_k p^k
    //  = sum_j (a_k p^k)^j (deriv_j f(r_{k-1})) / j!
    //  = sum_j a_k^j p^{j*k} f^(j)(r_{k-1}) / j!
    //
    // f(r_{k-1}) === 0 mod p^{k+v}
    // f(r_k) === 0 mod p^{k+v+1}
    //  => sum_j a_k^j p^{j*k} f^(j)(r_{k-1}) / j! === 0 mod p^{k+v+1}
    //  => f(r_{k-1}) + p^k * sum_j a_k^j p^{(j-1)*k} f^(j)(r_{k-1}) / j! === 0 mod p^{k+v+1}
    //  =>
    // f(r_{k-1})/p^k + sum_j a_k^j p^{(j-1)*k} f^(j)(r_{k-1}) / j! === 0 mod p^{v+1}
    //
    // This final modulus can be used to find a_k
    // Just plug in all a_k in F_p = [0, 1, ... p-1] until you see 0
    // We need to calculate all derivatives, but at least as a polynomial that's finite
    // And we can store the f and derivatives and calculate the next slowly with:
    //
    // f^(m)(r_{k}) = sum_j a_k^j p^{j*k} f^(m+j)(r_{k-1}) / j!

    for variety in &mut varieties {

        // For n-th root, f(r_k) = x^n - b and f^(j>0)(r_k) = n! / (n-j)! x^(n-j)
        let var_int = variety.truncation_to_uadic()?;
        let mut pows = Vec::with_capacity(n as usize + 1);
        let mut cur_pow = UAdic::one(p);
        pows.push(cur_pow.clone());
        for _ in (1..=n) {
            cur_pow = cur_pow.clone() * var_int.clone();
            pows.push(cur_pow.clone());
        }
        let mut pows = pows.into_iter().rev().collect::<Vec<_>>();

        for k in v..precision {

            // f(r_{k-1})/p^k + sum_j a_k^j p^{(j-1)*k} f^(j)(r_{k-1}) / j! === 0 mod p^{v+1}
            // See which digit fits the modular equation

            let poly = pows.first().unwrap();
            let mut new_digit = 0;
            if poly.digit(k+v-1)? % p != b.digit(k+v-1)? {
                for d in 0..p {
                    let mut left_full = poly.clone();
                    for (j, poly_deriv) in pows.iter().enumerate().skip(1) {

                        // We don't have to do this much calculation!
                        // We shouldn't need to do the full adic sums and powers, just some digits.
                        // TODO: simplify

                        // "Derivative factor" for x^n is d^(j) x^n / j! = n choose j (without x)
                        let dfactor = adic_nchoosek(p, n, j as u32);
                        let dpow = uadic!(p, [d]).pow(j as u32);
                        let kj = (k as usize) * j;
                        let pfactor = UAdic::new(p, repeat_n(0, kj).chain(once(1)).collect());
                        let added = dpow * dfactor * pfactor * poly_deriv.clone();
                        left_full = left_full.clone() + added;

                    }
                    let left_side = left_full.digit(k+v-1)?;
                    let right_side = b.digit(k+v-1)?;
                    if left_side % p == right_side {
                        new_digit = d;
                        break;
                    }
                }
            }
            variety.push_digit(new_digit)?;

            // f^(m)(r_{k}) = sum_j a_k^j p^{j*k} f^(m+j)(r_{k-1}) / j!
            // Update pows

            pows = pows.iter().enumerate().map(|(m, _)| {
                let new_pow = pows.iter().skip(m).enumerate().map(
                    |(j, power)| {
                        let digit_pow = uadic!(p, [new_digit]).pow(j as u32);
                        let dfactor = adic_nchoosek(p, (n-m as u32), j as u32);
                        let kj = (k as usize) * j;
                        let pfactor = UAdic::new(p, repeat_n(0, kj).chain(once(1)).collect());
                        digit_pow * dfactor * pfactor * power.clone()
                    }
                ).fold(UAdic::zero(p), |acc, el| acc + el);
                new_pow
            }).collect();

        }
    }

    Ok(varieties)

}

/// Utility method to calculate roots of unity for given p
pub fn roots_of_unity(p: u32, precision: u32) -> Result<ZAdicVariety, AdicError> {
    ZAdic::one(p).nth_root(p-1, precision)
}

fn adic_nchoosek(p: u32, n: u32, k: u32) -> UAdic {
    UAdic::from_integer(p, binomial(n, k))
}


/// Newton/Hensel method for deriving digits.
///
/// See ([`AdicInteger::nth_root`]) for information on the algorithm.
/// Just abstract the f(x) to be a polynomial (or even Taylor series).
///
/// <div class="warning">
///
/// Currently, this returns an error if input is not in the form [a, 0, 0, ..., 1].
/// We do not handle general polynomials at this point.
/// For now, this just serves as a passthrough to [`nth_root`]
///
/// </div>
///
/// # Errors
/// Errors if:
/// 1. Polynomial is not of the form `x^n - a`
/// 2. p == 2
/// 3. n == 0
/// 4. `precision` is not high enough (roughly, `self.certainty() >= n * precision`)
///
/// # Panics
/// Panics if input coefficients have length zero
pub fn variety_to_digits<AdicInt>(p: u32, a: &[AdicInt], precision: u32) -> Result<ZAdicVariety, AdicError>
where AdicInt: AdicInteger {
    let nonzero_coefficients = a.iter().filter(|&a_n| *a_n != AdicInt::zero(p)).count();
    if nonzero_coefficients > 2
    || *a.iter().last().unwrap() != AdicInt::one(p)
    || (nonzero_coefficients > 1 && a[0] == AdicInt::zero(p)) {
        Err(AdicError::NotImplemented(GENERAL_POLYNOMIAL_ERROR_MSG.to_string()))
    } else {
        nth_root(&a[0], u32::try_from(a.len()).unwrap() - 1, precision)
    }
}


#[cfg(test)]
mod tests {
    use std::iter::repeat_n;
    use itertools::Itertools;
    use num::{traits::Pow, Rational32};
    use crate::{
        radic, uadic, zadic_approx, zadic_variety,
        AdicError, AdicInteger, RAdic, UAdic, ZAdic, ZAdicVariety,
    };

    use super::{variety_to_digits, nth_root, roots_of_unity};

    fn zero2() -> UAdic { uadic!(2, []) }
    fn one2() -> UAdic { uadic!(2, [1]) }
    fn two2() -> UAdic { uadic!(2, [0, 1]) }
    fn three2() -> UAdic { uadic!(2, [1, 1]) }
    fn four2() -> UAdic { uadic!(2, [0, 0, 1]) }
    fn eight2() -> UAdic { uadic!(2, [0, 0, 0, 1]) }
    fn sixteen2() -> UAdic { uadic!(2, [0, 0, 0, 0, 1]) }
    fn seventeen2() -> UAdic { uadic!(2, [1, 0, 0, 0, 1]) }

    fn ten3() -> UAdic { uadic!(3, [1, 0, 1]) }

    fn zero5() -> UAdic { uadic!(5, []) }
    fn one5() -> UAdic { uadic!(5, [1]) }
    fn two5() -> UAdic { uadic!(5, [2]) }
    fn five5() -> UAdic { uadic!(5, [0, 1]) }
    fn twenty_five5() -> UAdic { uadic!(5, [0, 0, 1]) }
    fn one_twenty_five5() -> UAdic { uadic!(5, [0, 0, 0, 1]) }
    fn six_twenty_five5() -> UAdic { uadic!(5, [0, 0, 0, 0, 1]) }

    fn zero7() -> UAdic { UAdic::zero(7) }
    fn one7() -> UAdic { UAdic::one(7) }
    fn two7() -> UAdic { uadic!(7, [2]) }
    fn ninety_eight7() -> UAdic { uadic!(7, [0, 0, 2]) }

    fn pos_one_fourth() -> RAdic { radic!(5, [4], [3]) }
    fn neg_one_half() -> RAdic { radic!(5, [], [2]) }
    fn pos_one_half() -> RAdic { radic!(5, [3], [2]) }

    #[test]
    fn test_7_adic_sqrt_2_digits() {

        let expected = zadic_variety!(7, 3, [
            [3, 1, 2],
            [4, 5, 4],
        ]);
        let actual = nth_root(&two7(), 2, 3);
        assert_eq!(expected, actual.unwrap());
        let actual = variety_to_digits(7, &vec![two7(), zero7(), one7()], 3);
        assert_eq!(expected, actual.unwrap());

        let two_3 = zadic_approx!(7, 3, [2]);
        for zadic in expected.into_roots() {
            assert_eq!(two_3, zadic.clone() * zadic.clone());
        }

    }

    #[test]
    fn test_5_adic_sqrt_2_digits() {
        let expected = ZAdicVariety::empty(5);
        let actual= nth_root(&two5(), 2, 3);
        assert_eq!(expected, actual.unwrap());
        let actual= variety_to_digits(5, &vec![two5(), zero5(), one5()], 3);
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn test_2_adic_sqrt_2_digits() {
        let expected = ZAdicVariety::empty(2);
        let actual= nth_root(&two2(), 2, 3);
        assert_eq!(expected, actual.unwrap());
        let actual= variety_to_digits(2, &vec![two2(), zero2(), one2()], 3);
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn test_nth_root_of_p() {

        // sqrt(5^m)
        let expected = ZAdicVariety::empty(5);
        let actual = nth_root(&five5(), 2, 4);
        assert_eq!(expected, actual.unwrap());
        let expected = zadic_variety!(5, 4, [
            [0, 1, 0, 0],
            [0, 4, 4, 4],
        ]);
        let actual = nth_root(&twenty_five5(), 2, 4);
        assert_eq!(expected, actual.unwrap());
        let expected = ZAdicVariety::empty(5);
        let actual = nth_root(&one_twenty_five5(), 2, 4);
        assert_eq!(expected, actual.unwrap());
        let expected = zadic_variety!(5, 4, [
            [0, 0, 1, 0],
            [0, 0, 4, 4],
        ]);
        let actual = nth_root(&six_twenty_five5(), 2, 4);
        assert_eq!(expected, actual.unwrap());

        // cubert(5^m)
        let expected = ZAdicVariety::empty(5);
        let actual = nth_root(&five5(), 3, 4);
        assert_eq!(expected, actual.unwrap());
        let expected = ZAdicVariety::empty(5);
        let actual = nth_root(&twenty_five5(), 3, 4);
        assert_eq!(expected, actual.unwrap());
        let expected = zadic_variety!(5, 4, [
            [0, 1, 0],
        ]);
        let actual = nth_root(&one_twenty_five5(), 3, 4);
        assert_eq!(expected, actual.unwrap());
        let expected = ZAdicVariety::empty(5);
        let actual = nth_root(&six_twenty_five5(), 3, 4);
        assert_eq!(expected, actual.unwrap());

        // fourthrt(5^m)
        let expected = ZAdicVariety::empty(5);
        let actual = nth_root(&five5(), 4, 4);
        assert_eq!(expected, actual.unwrap());
        let expected = ZAdicVariety::empty(5);
        let actual = nth_root(&twenty_five5(), 4, 4);
        assert_eq!(expected, actual.unwrap());
        let expected = ZAdicVariety::empty(5);
        let actual = nth_root(&one_twenty_five5(), 4, 4);
        assert_eq!(expected, actual.unwrap());
        let expected = zadic_variety!(5, 4, [
            [0, 1, 0, 0],
            [0, 2, 1, 2],
            [0, 3, 3, 2],
            [0, 4, 4, 4],
        ]);
        let actual = nth_root(&six_twenty_five5(), 4, 4);
        assert_eq!(expected, actual.unwrap());

        let expected = zadic_variety!(7, 6, [
            [0, 3, 1, 2, 6, 1],
            [0, 4, 5, 4, 0, 5],
        ]);
        let actual = nth_root(&ninety_eight7(), 2, 6);
        assert_eq!(expected, actual.unwrap());

    }

    #[test]
    fn test_pth_root() {

        // cubert(10) in 3-adics
        let solution = zadic_approx!(3, 8, [1, 1, 1, 0, 0, 0, 2, 1]);
        let expected = ZAdicVariety::new(3, vec![solution.clone()]);
        let actual = nth_root(&ten3(), 3, 8);
        assert_eq!(expected, actual.unwrap());
        assert_eq!(ten3(), solution.pow(3).into_truncation_to_uadic().unwrap());

        // fifthrt(1._5) has a single root, 1
        assert_eq!(
            zadic_variety!(5, 8, [[1]]),
            nth_root(&UAdic::one(5), 5, 8).unwrap()
        );

        // fifthrt(112._5) has a single root, 2
        let var2 = zadic_variety!(5, 8, [[2]]);
        assert_eq!(var2, nth_root(&uadic!(5, [2, 1, 1]), 5, 8).unwrap());

        // fifthrt((12._5)^5) has one root: 12._5
        let seven = uadic!(5, [2, 1]);
        let seven_to_fifth = uadic!(5, [2, 1, 2, 4, 1, 0, 1]);
        assert_eq!(seven.clone().pow(5), seven_to_fifth);
        let expected = ZAdicVariety::new(5, vec![
            ZAdic::new_approx(5, 4, seven.into_digits_vec())
        ]);
        let actual = nth_root(
            &ZAdic::new_approx(5, 20, seven_to_fifth.into_digits_vec()),
            5, 4
        );
        assert_eq!(expected, actual.unwrap());

        // nth-rt((-9/2)^n)
        let neg_nine_half = radic!(3, [0, 0], [1]);
        assert_eq!(Rational32::new(-9, 2), neg_nine_half.rational_value());
        let squared = neg_nine_half.clone().pow(2);
        assert_eq!(Rational32::new(81, 4), squared.rational_value());
        let cubed = neg_nine_half.clone().pow(3);
        assert_eq!(Rational32::new(-729, 8), cubed.rational_value());
        let fourthed = neg_nine_half.clone().pow(4);
        let fifthed = neg_nine_half.clone().pow(5);
        let zneg_nine_half = ZAdic::new_approx(3, 4, neg_nine_half.clone().into_digits().take(4).collect());
        let zpos_nine_half = ZAdic::new_approx(3, 4, (-neg_nine_half.clone()).into_digits().take(4).collect());
        let var_one = ZAdicVariety::new(3, vec![zneg_nine_half.clone()]);
        let var_both = ZAdicVariety::new(3, vec![zneg_nine_half.clone(), zpos_nine_half.clone()]);
        assert_eq!(var_both, nth_root(&squared, 2, 4).unwrap());
        assert_eq!(var_one, nth_root(&cubed, 3, 4).unwrap());
        assert_eq!(var_both, nth_root(&fourthed, 4, 4).unwrap());
        assert_eq!(var_one, nth_root(&fifthed, 5, 4).unwrap());

    }

    #[ignore = "slow"]
    #[test]
    fn test_pth_root_slow() {

        // twentyfifthrt(32042220212._5) has a single root, 2
        let var2 = zadic_variety!(5, 8, [[2]]);
        assert_eq!(
            var2,
            nth_root(&uadic!(5, [2, 1, 2, 0, 2, 2, 2, 4, 0, 2, 3]), 25, 8).unwrap()
        );

        // twentyfifthrt(231310332124302430341011314243033243440243222010001._5) has a single root, 101.5
        let u26 = uadic!(5, [1, 0, 1]);
        let var26 = ZAdicVariety::new(5, vec![ZAdic::new_approx(5, 6, u26.clone().into_digits_vec())]);
        let u26_pow25 = uadic!(5, [
            1, 0, 0, 0, 1, 0, 2, 2, 2, 3,
            4, 2, 0, 4, 4, 3, 4, 2, 3, 3,
            0, 3, 4, 2, 4, 1, 3, 1, 1, 0,
            1, 4, 3, 0, 3, 4, 2, 0, 3, 4,
            2, 1, 2, 3, 3, 0, 1, 3, 1, 3,
            2
        ]);
        assert_eq!(u26_pow25.big_integer_value(), u26.big_integer_value().pow(25 as u32));
        assert_eq!(var26, nth_root(&u26_pow25, 25, 6).unwrap());

        // nth-rt((-9/2)^n)
        let neg_nine_half = radic!(3, [0, 0], [1]);
        let sixthed = neg_nine_half.clone().pow(6);
        let ninthed = neg_nine_half.clone().pow(9);
        let zneg_nine_half = ZAdic::new_approx(3, 4, neg_nine_half.clone().into_digits().take(4).collect());
        let zpos_nine_half = ZAdic::new_approx(3, 4, (-neg_nine_half.clone()).into_digits().take(4).collect());
        let var_one = ZAdicVariety::new(3, vec![zneg_nine_half.clone()]);
        let var_both = ZAdicVariety::new(3, vec![zneg_nine_half.clone(), zpos_nine_half.clone()]);
        assert_eq!(var_both, nth_root(&sixthed, 6, 4).unwrap());
        assert_eq!(var_one, nth_root(&ninthed, 9, 4).unwrap());

    }

    #[ignore = "Takes five minutes"]
    #[test]
    fn test_nth_root_many() {

        // Test nth_root over many integers and rationals

        // Test 5-adic positive integers
        let p = 5;
        let num_digits = 3;
        let pows = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 15, 20, 25, 26];
        let roots = repeat_n(0..p, num_digits).multi_cartesian_product().map(
            |digits| UAdic::new(p, digits.into_iter().rev().collect())
        );
        for root in roots {
            let root_val = root.big_integer_value();
            for power in pows {
                let root_powed = root.pow(power);
                let root_powed_val = root_powed.big_integer_value();
                assert_eq!(root_val.clone().pow(power), root_powed_val);
                println!("{root}({root_val})^{power} = {root_powed}({root_powed_val})");
                let variety = nth_root(&root_powed, power, 6).unwrap();
                println!("[{}]", variety.roots().map(|r| r.to_string()).join(", "));
                assert!(variety.roots().any(|var_root| var_root.truncation_to_uadic().unwrap() == root));
            }
        }

        // Test 3-adic rationals
        let p = 3;
        let fix_num = 2;
        let rep_num = 1;
        let pows = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let roots = repeat_n(0..p, fix_num).multi_cartesian_product().cartesian_product(
            repeat_n(0..p, rep_num).multi_cartesian_product()
        ).map(
            |(fixed_digits, repeat_digits)| RAdic::new(p, fixed_digits, repeat_digits)
        );
        for root in roots {
            let root_val = root.big_rational_value();
            for power in pows {
                let root_powed = root.pow(power);
                let root_powed_val = root_powed.big_rational_value();
                assert_eq!(root_val.clone().pow(power), root_powed_val);
                println!("{root}({root_val})^{power} = {root_powed}({root_powed_val})");
                let variety = nth_root(&root_powed, power, 6).unwrap();
                println!("[{}]", variety.roots().map(|r| r.to_string()).join(", "));
                assert!(variety.roots().any(|var_root| var_root.truncation_to_uadic().unwrap() == root.truncation(6)));
            }
        }

    }

    #[test]
    fn test_2_adic() {

        // sqrt(2^m)
        let expected = ZAdicVariety::empty(2);
        let actual = nth_root(&two2(), 2, 4);
        assert_eq!(expected, actual.unwrap());
        let expected = zadic_variety!(2, 4, [
            [0, 1, 0, 0],
            [0, 1, 1, 1],
        ]);
        let actual = nth_root(&four2(), 2, 4);
        assert_eq!(expected, actual.unwrap());
        let expected = ZAdicVariety::empty(2);
        let actual = nth_root(&eight2(), 2, 4);
        assert_eq!(expected, actual.unwrap());
        let expected = zadic_variety!(2, 4, [
            [0, 0, 1, 0],
            [0, 0, 1, 1],
        ]);
        let actual = nth_root(&sixteen2(), 2, 4);
        assert_eq!(expected, actual.unwrap());

        // sqrt(3)
        let expected = ZAdicVariety::empty(2);
        let actual = nth_root(&three2(), 2, 4);
        assert_eq!(expected, actual.unwrap());

        // sqrt(17)
        let expected = zadic_variety!(2, 4, [
            [1, 0, 0, 1],
            [1, 1, 1, 0],
        ]);
        let actual = nth_root(&seventeen2(), 2, 4);
        assert_eq!(expected, actual.unwrap());

    }

    #[ignore = "Takes five minutes"]
    #[test]
    fn test_2_adic_many() {

        // Test nth_root over many 2-adic integers and rationals
        let p = 2;

        // Test 2-adic positive integers
        let num_digits = 3;
        let pows = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let roots = repeat_n(0..p, num_digits).multi_cartesian_product().map(
            |digits| UAdic::new(p, digits.into_iter().rev().collect())
        );
        for root in roots {
            let root_val = root.big_integer_value();
            for power in pows {
                let root_powed = root.pow(power);
                let root_powed_val = root_powed.big_integer_value();
                assert_eq!(root_val.clone().pow(power), root_powed_val);
                println!("{root}({root_val})^{power} = {root_powed}({root_powed_val})");
                let variety = nth_root(&root_powed, power, 6).unwrap();
                println!("[{}]", variety.roots().map(|r| r.to_string()).join(", "));
                assert!(variety.roots().any(|var_root| var_root.truncation_to_uadic().unwrap() == root));
            }
        }

        // Test 2-adic rationals
        let fix_num = 2;
        let rep_num = 1;
        let pows = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let roots = repeat_n(0..p, fix_num).multi_cartesian_product().cartesian_product(
            repeat_n(0..p, rep_num).multi_cartesian_product()
        ).map(
            |(fixed_digits, repeat_digits)| RAdic::new(p, fixed_digits, repeat_digits)
        );
        for root in roots {
            let root_val = root.big_rational_value();
            for power in pows {
                let root_powed = root.pow(power);
                let root_powed_val = root_powed.big_rational_value();
                assert_eq!(root_val.clone().pow(power), root_powed_val);
                println!("{root}({root_val})^{power} = {root_powed}({root_powed_val})");
                let variety = nth_root(&root_powed, power, 6).unwrap();
                println!("[{}]", variety.roots().map(|r| r.to_string()).join(", "));
                assert!(variety.roots().any(|var_root| var_root.truncation_to_uadic().unwrap() == root.truncation(6)));
            }
        }

        // Test 2-adic rationals (more digits, fewer powers)
        let fix_num = 2;
        let rep_num = 2;
        let pows = [1, 2, 3];
        let roots = repeat_n(0..p, fix_num).multi_cartesian_product().cartesian_product(
            repeat_n(0..p, rep_num).multi_cartesian_product()
        ).map(
            |(fixed_digits, repeat_digits)| RAdic::new(p, fixed_digits, repeat_digits)
        );
        for root in roots {
            let root_val = root.big_rational_value();
            for power in pows {
                let root_powed = root.pow(power);
                let root_powed_val = root_powed.big_rational_value();
                assert_eq!(root_val.clone().pow(power), root_powed_val);
                println!("{root}({root_val})^{power} = {root_powed}({root_powed_val})");
                let variety = nth_root(&root_powed, power, 6).unwrap();
                println!("[{}]", variety.roots().map(|r| r.to_string()).join(", "));
                assert!(variety.roots().any(|var_root| var_root.truncation_to_uadic().unwrap() == root.truncation(6)));
            }
        }

    }

    #[test]
    fn test_a_equals_zero() {
        // Returns a single solution
        let expected = ZAdicVariety::new(5, vec![ZAdic::zero(5)]);
        let actual = nth_root(&zero5(), 2, 3);
        assert_eq!(actual.unwrap(), expected);
        let actual = variety_to_digits(5, &vec![zero5(), zero5(), one5()], 3);
        assert_eq!(actual.unwrap(), expected);
    }

    #[test]
    fn test_5_adic_sqrt_dne_digits() {
        let expected = ZAdicVariety::empty(5);

        let actual = nth_root(&five5(), 2, 3);
        assert_eq!(actual.unwrap(), expected);

        let actual = nth_root(&one_twenty_five5(), 2, 3);
        assert_eq!(actual.unwrap(), expected);
    }

    #[test]
    fn test_5_adic_sqrt_exists_digits() {

        let expected = zadic_variety!(5, 4, [
            [0, 1, 0, 0],
            [0, 4, 4, 4],
        ]);
        let actual = nth_root(&twenty_five5(), 2, 4);
        assert_eq!(actual.unwrap(), expected);

        let twenty_five_4 = zadic_approx!(5, 5, [0, 0, 1]);
        for zadic in expected.into_roots() {
            assert_eq!(twenty_five_4, zadic.clone() * zadic.clone());
        }

        let expected = zadic_variety!(5, 4, [
            [0, 0, 1, 0],
            [0, 0, 4, 4],
        ]);
        let actual = nth_root(&six_twenty_five5(), 2, 4);
        assert_eq!(actual.unwrap(), expected);

        let six_twenty_five_4 = zadic_approx!(5, 6, [0, 0, 0, 0, 1]);
        for zadic in expected.into_roots() {
            assert_eq!(six_twenty_five_4, zadic.clone() * zadic.clone());
        }

    }

    #[test]
    fn test_7_adic_sqrt_exist_digits() {

        let expected = zadic_variety!(7, 5, [
            [0, 3, 1, 2, 6],
            [0, 4, 5, 4, 0],
        ]);
        let actual = nth_root(&ninety_eight7(), 2, 5);
        assert_eq!(actual.unwrap(), expected);

        let ninety_eight_5 = zadic_approx!(7, 6, [0, 0, 2]);
        for zadic in expected.into_roots() {
            assert_eq!(ninety_eight_5, zadic.clone() * zadic.clone());
        }

    }

    #[test]
    fn test_7_adic_sqrt_fractions() {

        let expected = ZAdicVariety::new(5, vec![
            ZAdic::new_approx(5, 6, neg_one_half().into_truncation(6).into_digits().collect()),
            ZAdic::new_approx(5, 6, pos_one_half().into_truncation(6).into_digits().collect()),
        ]);
        let actual = nth_root(&pos_one_fourth(), 2, 6);
        assert_eq!(expected, actual.unwrap());

    }

    #[test]
    fn test_variety_to_digits_refactor() {
        let expected = zadic_variety!(7, 6, [
            [0, 3, 1, 2, 6, 1],
            [0, 4, 5, 4, 0, 5],
        ]);
        let ninety_eight = uadic!(7, [0, 0, 2]);
        let actual = variety_to_digits(7, &vec![ninety_eight.clone(), zero7(), one7()], 6);
        assert_eq!(expected, actual.unwrap());

        let actual = variety_to_digits(5, &vec![zero5(), ninety_eight.clone(), zero5(), one5()], 3);
        assert!(matches!(actual, Err(AdicError::NotImplemented(_))));

    }

    #[test]
    fn test_roots_of_unity() {
        let expected = zadic_variety!(3, 6, [
            [1, 0, 0, 0, 0, 0],
            [2, 2, 2, 2, 2, 2],
        ]);
        let actual = roots_of_unity(3, 6).unwrap();
        assert_eq!(expected, actual);
        assert!(actual.roots().try_len() == Ok(2));
        assert!(actual.roots().contains(&zadic_approx![3, 6, [1, 0, 0, 0, 0, 0]]));
        assert!(actual.roots().contains(&zadic_approx![3, 6, [2, 2, 2, 2, 2, 2]]));

        let expected = zadic_variety!(5, 6, [
            [1, 0, 0, 0, 0, 0],
            [2, 1, 2, 1, 3, 4],
            [3, 3, 2, 3, 1, 0],
            [4, 4, 4, 4, 4, 4],
        ]);
        let actual = roots_of_unity(5, 6).unwrap();
        assert_eq!(expected, actual);
        assert!(actual.roots().try_len() == Ok(4));
        assert!(actual.roots().contains(&zadic_approx![5, 6, [1, 0, 0, 0, 0, 0]]));
        assert!(actual.roots().contains(&zadic_approx![5, 6, [4, 4, 4, 4, 4, 4]]));

        let expected = zadic_variety!(7, 6, [
            [1, 0, 0, 0, 0, 0],
            [2, 4, 6, 3, 0, 2],
            [3, 4, 6, 3, 0, 2],
            [4, 2, 0, 3, 6, 4],
            [5, 2, 0, 3, 6, 4],
            [6, 6, 6, 6, 6, 6],
        ]);
        let actual = roots_of_unity(7, 6).unwrap();
        assert_eq!(expected, actual);
        assert!(actual.roots().try_len() == Ok(6));
        assert!(actual.roots().contains(&zadic_approx![7, 6, [1, 0, 0, 0, 0, 0]]));
        assert!(actual.roots().contains(&zadic_approx![7, 6, [6, 6, 6, 6, 6, 6]]));
    }

}
