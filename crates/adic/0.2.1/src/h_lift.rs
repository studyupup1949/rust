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
    let b = a.int_div(m as usize);

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
fn general_nth_root(b: &UAdic, n: u32, precision: u32) -> Result<Vec<ZAdic>, AdicError> {
    let p = b.p();

    // Two parts:
    // - Find the solutions in F_{p^v}, where v is the max valuation of the derivative for any solution
    // - Lift those solutions up to the desired precision

    // We need to find roots in F_p and calculate the valuation of their derivative
    // If the valuation is larger than the digit we've calculated so far, search F_{p^2}, F_{p^3} etc
    let adic_b = ZAdic::from(b.clone());
    let adic_n = ZAdic::exact_from_integer(p, n as i32);
    let mut v: u32 = 0;
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
        for trial_digits in repeat_n(0..p, compare_v as usize).multi_cartesian_product() {
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
        .map(|var| var.into_approximation((v+1) as usize))
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
pub fn roots_of_unity(p: u32, precision: u32) -> Result<ZAdicVariety, AdicError> {
    let n = if p == 2 { 2 } else { p-1 };
    ZAdic::one(p).nth_root(n, precision)
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

    fn zero3() -> UAdic { uadic!(3, []) }
    fn one3() -> UAdic { uadic!(3, [1]) }
    fn two3() -> UAdic { uadic!(3, [0, 1]) }
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
    fn test_sqrt_2_digits() {

        // 2-adic
        let expected = ZAdicVariety::empty(2);
        let actual= nth_root(&two2(), 2, 3);
        assert_eq!(expected, actual.unwrap());
        let actual= variety_to_digits(2, &vec![two2(), zero2(), one2()], 3);
        assert_eq!(expected, actual.unwrap());

        // 3-adic
        let expected = ZAdicVariety::empty(3);
        let actual= nth_root(&two3(), 2, 3);
        assert_eq!(expected, actual.unwrap());
        let actual= variety_to_digits(3, &vec![two3(), zero3(), one3()], 3);
        assert_eq!(expected, actual.unwrap());

        // 5-adic
        let expected = ZAdicVariety::empty(5);
        let actual= nth_root(&two5(), 2, 3);
        assert_eq!(expected, actual.unwrap());
        let actual= variety_to_digits(5, &vec![two5(), zero5(), one5()], 3);
        assert_eq!(expected, actual.unwrap());

        // 7-adic
        let expected = zadic_variety!(7, 3, [
            [3, 1, 2],
            [4, 5, 4],
        ]);
        let actual = nth_root(&two7(), 2, 3);
        assert_eq!(expected, actual.unwrap());
        let actual = variety_to_digits(7, &vec![two7(), zero7(), one7()], 3);
        assert_eq!(expected, actual.unwrap());

        for zadic in expected.into_roots() {
            assert_eq!(zadic_approx!(7, 3, [2]), zadic.clone() * zadic.clone());
        }

    }

    #[test]
    fn test_nth_root_of_p() {

        // 5-adic

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

        // 7-adic

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
            seven.into_approximation(4)
        ]);
        let actual = nth_root(
            &seven_to_fifth.into_approximation(20),
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
        let zneg_nine_half = neg_nine_half.approximation(4);
        let zpos_nine_half = (-neg_nine_half).approximation(4);
        let var_one = ZAdicVariety::new(3, vec![zneg_nine_half.clone()]);
        let var_both = ZAdicVariety::new(3, vec![zneg_nine_half.clone(), zpos_nine_half.clone()]);
        assert_eq!(var_both, nth_root(&squared, 2, 4).unwrap());
        assert_eq!(var_one, nth_root(&cubed, 3, 4).unwrap());
        assert_eq!(var_both, nth_root(&fourthed, 4, 4).unwrap());
        assert_eq!(var_one, nth_root(&fifthed, 5, 4).unwrap());

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
        let expected = zadic_variety!(2, 8, [
            [1, 0, 0, 1, 0, 1, 1, 1],
            [1, 1, 1, 0, 1, 0, 0, 0],
        ]);
        let actual = nth_root(&seventeen2(), 2, 8);
        assert_eq!(expected, actual.unwrap());

        let expected = zadic_variety!(2, 8, [
            [1, 0, 0, 0, 0, 0, 0, 0],
            [1, 1, 1, 1, 1, 1, 1, 1],
        ]);
        let actual = nth_root(&one2(), 2, 8).unwrap();
        assert_eq!(expected, actual);
        let actual = nth_root(&one2(), 4, 8).unwrap();
        assert_eq!(expected, actual);
        let actual = nth_root(&one2(), 6, 8).unwrap();
        assert_eq!(expected, actual);
        let actual = nth_root(&one2(), 8, 8).unwrap();
        assert_eq!(expected, actual);
        let actual = nth_root(&one2(), 10, 8).unwrap();
        assert_eq!(expected, actual);

        let expected = zadic_variety!(2, 8, [
            [1, 0, 1, 1, 1, 1, 1, 1],
            [1, 1, 0, 0, 0, 0, 0, 0],
        ]);
        let eighty_one = uadic!(2, [1, 0, 0, 0, 1, 0, 1]);
        let actual = nth_root(&eighty_one, 4, 8).unwrap();
        assert_eq!(expected, actual);

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
    fn test_sqrt_fractions() {

        // 2-adic

        let pos_one_ninth = radic!(2, [1], [0, 0, 1, 1, 1, 0]);
        let neg_one_third = radic!(2, [], [1, 0]);
        let pos_one_third = radic!(2, [1], [1, 0]);
        let expected = ZAdicVariety::new(2, vec![
            neg_one_third.into_approximation(6),
            pos_one_third.into_approximation(6),
        ]);
        let actual = nth_root(&pos_one_ninth, 2, 6).unwrap();
        assert_eq!(expected, actual);

        // 5-adic

        let expected = ZAdicVariety::new(5, vec![
            neg_one_half().into_approximation(6),
            pos_one_half().into_approximation(6),
        ]);
        let actual = nth_root(&pos_one_fourth(), 2, 6).unwrap();
        assert_eq!(expected, actual);

        // 7-adic

        let pos_one_sixteenth = radic!(7, [4], [6, 3]);
        let neg_one_fourth = radic!(7, [], [5, 1]);
        let pos_one_fourth = radic!(7, [2], [5, 1]);
        let neg_one_half = radic!(7, [], [3]);
        let pos_one_half = radic!(7, [4], [3]);
        let expected = ZAdicVariety::new(7, vec![
            pos_one_fourth.approximation(6),
            neg_one_fourth.approximation(6),
        ]);
        let actual = nth_root(&pos_one_sixteenth, 2, 6).unwrap();
        assert_eq!(expected, actual);
        let expected = ZAdicVariety::new(7, vec![
            neg_one_half.into_approximation(6),
            pos_one_half.into_approximation(6),
        ]);
        let actual = nth_root(&pos_one_fourth, 2, 6).unwrap();
        assert_eq!(expected, actual);

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

        let expected = zadic_variety!(2, 6, [
            [1, 0, 0, 0, 0, 0],
            [1, 1, 1, 1, 1, 1],
        ]);
        let actual = roots_of_unity(2, 6).unwrap();
        assert_eq!(expected, actual);
        assert!(actual.roots().try_len() == Ok(2));
        assert!(actual.roots().contains(&zadic_approx![2, 6, [1, 0, 0, 0, 0, 0]]));
        assert!(actual.roots().contains(&zadic_approx![2, 6, [1, 1, 1, 1, 1, 1]]));

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


    // SLOW TESTS; enable when you want to fully test

    #[ignore = "slow"]
    #[test]
    fn test_pth_root_slow() {

        // twentyfifthrt(32042220212._5) has a single root, 2
        println!("twenty_fifth rt of (32042220212._5) is 2._5...");
        let var2 = zadic_variety!(5, 8, [[2]]);
        assert_eq!(
            var2,
            nth_root(&uadic!(5, [2, 1, 2, 0, 2, 2, 2, 4, 0, 2, 3]), 25, 8).unwrap()
        );

        // twentyfifthrt(231310332124302430341011314243033243440243222010001._5) has a single root, 101.5
        println!("twenty_fifth rt of (101.5^25) is 101._5...");
        let u26 = uadic!(5, [1, 0, 1]);
        let var26 = ZAdicVariety::new(5, vec![u26.approximation(6)]);
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
        println!("nth rt of (-9/2^n)...");
        let neg_nine_half = radic!(3, [0, 0], [1]);
        let sixthed = neg_nine_half.clone().pow(6);
        let zneg_nine_half = neg_nine_half.approximation(4);
        let zpos_nine_half = (-neg_nine_half.clone()).approximation(4);
        let var_both = ZAdicVariety::new(3, vec![zneg_nine_half.clone(), zpos_nine_half.clone()]);
        assert_eq!(var_both, nth_root(&sixthed, 6, 4).unwrap());

    }

    #[ignore = "very slow"]
    #[test]
    fn test_nth_root_many() {

        // Test nth_root over many integers and rationals

        // Test 5-adic positive integers
        let p = 5;
        let num_digits = 2;
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

    #[ignore = "slow"]
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

}
