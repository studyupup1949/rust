use itertools::{Itertools, repeat_n};
use num::{Rational32, traits::{Inv, Pow}};
use crate::{
    iadic_neg, iadic_pos, radic, uadic, zadic_approx, zadic_exact_pos,
    AdicError, ZAdicValuation,
};
use super::{AdicInteger, IAdic, RAdic, UAdic, ZAdic};

use crate::num_adic::test_util::{i, r, u, z};


// UAdic

#[test]
fn add_u_adic() {
    assert_eq!(u::two(), u::one() + u::one());
    assert_eq!(u::three(), u::two() + u::one());
    assert_eq!(u::five(), u::two() + u::three());
    let neg_one_plus_neg_one = u::app_neg_one() + u::app_neg_one();
    assert_eq!(uadic!(5, [3, 4, 4, 4, 1]), neg_one_plus_neg_one);
    assert_eq!(u::app_neg_two(), neg_one_plus_neg_one.into_truncation(4));
    let neg_two_plus_neg_three = u::app_neg_two() + u::app_neg_three();
    assert_eq!(uadic!(5, [0, 4, 4, 4, 1]), neg_two_plus_neg_three);
    assert_eq!(u::app_neg_five(), neg_two_plus_neg_three.into_truncation(4));
    let neg_five_plus_neg_five = u::app_neg_five() + u::app_neg_five();
    assert_eq!(uadic!(5, [0, 3, 4, 4, 1]), neg_five_plus_neg_five);
    assert_eq!(u::app_neg_ten(), neg_five_plus_neg_five.into_truncation(4));
    let two_plus_neg_two = u::two() + u::app_neg_two();
    assert_eq!(uadic!(5, [0, 0, 0, 0, 1]), two_plus_neg_two);
    assert_eq!(u::zero(), two_plus_neg_two.into_truncation(4));
    let four_plus_one_grows = uadic!(5, [4]) + uadic!(5, [1]);
    assert_eq!(uadic!(5, [0, 1]), four_plus_one_grows);
}

#[test]
fn mul_u_adic() {

    assert_eq!(u::one(), u::one() * u::one());
    assert_eq!(u::two(), u::two() * u::one());
    assert_eq!(u::six(), u::two() * u::three());
    let neg_one_mul_neg_one = u::app_neg_one() * u::app_neg_one();
    assert_eq!(uadic!(5, [1, 0, 0, 0, 3, 4, 4, 4]), neg_one_mul_neg_one);
    assert_eq!(u::one(), neg_one_mul_neg_one.into_truncation(4));
    let neg_two_mul_neg_three = u::app_neg_two() * u::app_neg_three();
    assert_eq!(uadic!(5, [1, 1, 0, 0, 0, 4, 4, 4]), neg_two_mul_neg_three);
    assert_eq!(u::six(), neg_two_mul_neg_three.into_truncation(4));
    assert_eq!(u::zero(), u::zero() * u::two());
    assert_eq!(u::zero(), u::zero() * u::app_neg_two());
    let truncates_zeros = uadic!(5, [2, 0, 0, 0, 0]) * uadic!(5, [3, 0, 0, 0, 0]);
    assert_eq!(uadic!(5, [1, 1]), truncates_zeros);
    assert_eq!(u::ten(), u::five() * u::two());
    assert_eq!(u::twenty_five(), u::five() * u::five());

    assert_eq!(u::two(), 2 * u::one());
    assert_eq!(u::three(), 3 * u::one());
    assert_eq!(u::five(), 5 * u::one());
    assert_eq!(u::twenty_five(), 5 * u::five());

}

#[test]
fn pow_u_adic() {
    assert_eq!(u::zero(), u::zero().pow(2));
    assert_eq!(u::zero(), u::zero().pow(3));
    assert_eq!(u::one(), u::one().pow(2));
    assert_eq!(u::one(), u::one().pow(3));
    assert_eq!(u::four(), u::two().pow(2));
    assert_eq!(u::eight(), u::two().pow(3));
    assert_eq!(u::twenty_five(), u::five().pow(2));
    assert_eq!(u::one(), u::app_neg_two().pow(0));
    assert_eq!(u::app_neg_one(), u::app_neg_one().pow(1));
    assert_eq!(uadic!(5, [1, 0, 0, 0, 3, 4, 4, 4]), u::app_neg_one().pow(2));
    assert_eq!(uadic!(5, [4, 0, 0, 0, 1, 4, 4, 4]), u::app_neg_two().pow(2));
}

#[test]
fn div_u_adic() {
    assert_eq!(Ok(zadic_approx!(5, 4, [1, 0, 0, 0])), (u::one() / u::one()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [2, 0, 0, 0])), (u::two() / u::one()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [1, 0, 0, 0])), (u::two() / u::two()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [3, 2, 2, 2])), (u::one() / u::two()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [2, 2, 2, 2])), (u::app_neg_one() / u::two()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [2, 2, 2, 2])), (u::one() / u::app_neg_two()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [3, 2, 2, 2])), (u::app_neg_one() / u::app_neg_two()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [2, 3, 1, 3])), (u::one() / u::three()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [4, 4, 3, 4])), (u::one() / u::twenty_four()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [0, 0, 0])), (u::one() / u::five()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [2, 2, 2, 2])), (u::one() / u::ten()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [3, 1, 3, 1])), (u::one() / u::fifteen()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [0, 0, 0, 0])), (u::one() / u::twenty_five()).approx(4));
    assert_eq!(Err(AdicError::DivideByZero), (u::one() / u::zero()).approx(4));
}

#[test]
fn u_adic_ops_many() {
    // Test addition and multiplication over many integers using u32_value
    let p = 5;
    let n1 = 3;
    let n2 = 2;
    let firsts = repeat_n(0..p, n1).multi_cartesian_product().map(
        |digits| UAdic::new(p, digits[0..n1].to_vec())
    );
    let seconds = repeat_n(0..p, n2).multi_cartesian_product().map(
        |digits| UAdic::new(p, digits[0..n2].to_vec())
    );
    for (first, second) in firsts.cartesian_product(seconds) {
        let first_val = first.u32_value();
        let second_val = second.u32_value();
        let sum_val = (&first + &second).u32_value();
        let prod_val = (&first * &second).u32_value();
        assert_eq!(first_val + second_val, sum_val);
        assert_eq!(first_val * second_val, prod_val);
    }
}



// IAdic

#[test]
fn add_i_adic() {
    assert_eq!(i::two(), i::one() + i::one());
    assert_eq!(i::three(), i::two() + i::one());
    assert_eq!(i::five(), i::two() + i::three());
    let neg_one_plus_neg_one = i::neg_one() + i::neg_one();
    assert_eq!(i::neg_two(), neg_one_plus_neg_one);
    let neg_two_plus_neg_three = i::neg_two() + i::neg_three();
    assert_eq!(i::neg_five(), neg_two_plus_neg_three);
    let neg_five_plus_neg_five = i::neg_five() + i::neg_five();
    assert_eq!(i::neg_ten(), neg_five_plus_neg_five);
    let two_plus_neg_two = i::two() + i::neg_two();
    assert_eq!(i::zero(), two_plus_neg_two);
}

#[test]
fn neg_i_adic() {
    assert_eq!(i::neg_one(), -i::one());
    assert_eq!(i::zero(), -i::zero());
    assert_eq!(i::neg_five(), -i::five());
    let neg_p_to_third = -iadic_pos!(5, [0, 0, 0, 1]);
    assert_eq!(iadic_neg!(5, [0, 0, 0]), neg_p_to_third);
}

#[test]
fn sub_i_adic() {
    assert_eq!(i::one(), i::two() - i::one());
    assert_eq!(i::zero(), i::one() - i::one());
    assert_eq!(i::neg_one(), i::one() - i::two());
    assert_eq!(i::neg_five(), i::one() - i::six());
}

#[test]
fn mul_i_adic() {
    assert_eq!(i::zero(), i::zero() * i::one());
    assert_eq!(i::zero(), i::zero() * i::neg_one());
    assert_eq!(i::one(), i::one() * i::one());
    assert_eq!(i::two(), i::two() * i::one());
    assert_eq!(i::six(), i::two() * i::three());
    let neg_one_mul_neg_one = i::neg_one() * i::neg_one();
    assert_eq!(i::one(), neg_one_mul_neg_one);
    let neg_two_mul_neg_three = i::neg_two() * i::neg_three();
    assert_eq!(i::six(), neg_two_mul_neg_three);
    assert_eq!(i::zero(), i::zero() * i::two());
    assert_eq!(i::zero(), i::zero() * i::neg_two());
    assert_eq!(i::ten(), i::five() * i::two());
    assert_eq!(i::twenty_five(), i::five() * i::five());
    assert_eq!(i::neg_one(), i::one() * i::neg_one());
    assert_eq!(i::neg_one(), i::neg_one() * i::one());
    assert_eq!(i::one(), i::neg_one() * i::neg_one());
    assert_eq!(i::neg_one(), i::neg_one() * i::neg_one() * i::neg_one());
    assert_eq!(i::neg_two(), i::neg_one() * i::two());
    assert_eq!(i::neg_two(), i::neg_two() * i::one());
    assert_eq!(i::neg_ten(), i::neg_two() * i::five());
    assert_eq!(i::neg_ten(), i::neg_five() * i::two());
}

#[test]
fn pow_i_adic() {
    assert_eq!(i::zero(), i::zero().pow(2));
    assert_eq!(i::zero(), i::zero().pow(3));
    assert_eq!(i::one(), i::one().pow(2));
    assert_eq!(i::one(), i::one().pow(3));
    assert_eq!(i::four(), i::two().pow(2));
    assert_eq!(i::eight(), i::two().pow(3));
    assert_eq!(i::twenty_five(), i::five().pow(2));
    assert_eq!(i::one(), i::neg_two().pow(0));
    assert_eq!(i::neg_one(), i::neg_one().pow(1));
    assert_eq!(i::one(), i::neg_one().pow(2));
    assert_eq!(i::neg_one(), i::neg_one().pow(3));
    assert_eq!(i::four(), i::neg_two().pow(2));
}

#[test]
fn div_i_adic() {
    assert_eq!(Ok(zadic_approx!(5, 4, [1, 0, 0, 0])), (i::one() / i::one()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [2, 0, 0, 0])), (i::two() / i::one()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [1, 0, 0, 0])), (i::two() / i::two()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [3, 2, 2, 2])), (i::one() / i::two()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [2, 2, 2, 2])), (i::neg_one() / i::two()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [2, 2, 2, 2])), (i::one() / i::neg_two()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [3, 2, 2, 2])), (i::neg_one() / i::neg_two()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [2, 3, 1, 3])), (i::one() / i::three()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [4, 4, 3, 4])), (i::one() / i::twenty_four()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [0, 0, 0, 0])), (i::one() / i::five()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [2, 2, 2, 2])), (i::one() / i::ten()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [3, 1, 3, 1])), (i::one() / i::fifteen()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [0, 0, 0, 0])), (i::one() / i::twenty_five()).approx(4));
}

#[test]
fn i_adic_ops_many() {
    // Test addition and multiplication over many integers using i32_value
    let p = 5;
    let n1 = 2;
    let n2 = 2;
    let firsts = repeat_n(0..p, n1).multi_cartesian_product().flat_map(
        |digits| [IAdic::new_pos(p, digits[0..n1].to_vec()), IAdic::new_neg(p, digits[0..n1].to_vec())]
    );
    let seconds = repeat_n(0..p, n2).multi_cartesian_product().flat_map(
        |digits| [IAdic::new_pos(p, digits[0..n2].to_vec()), IAdic::new_neg(p, digits[0..n2].to_vec())]
    );
    for (first, second) in firsts.cartesian_product(seconds) {
        let first_val = first.i32_value();
        let second_val = second.i32_value();
        let sum_val = (&first + &second).i32_value();
        let prod_val = (&first * &second).i32_value();
        assert_eq!(first_val + second_val, sum_val);
        assert_eq!(first_val * second_val, prod_val);
    }
}



// RAdic

#[test]
fn add_r_integers() {
    assert_eq!(r::three(), r::two() + r::one());
    assert_eq!(r::two(), r::one() + r::one());
    assert_eq!(r::two() + r::one(), r::one() + r::two());
    assert_eq!(r::seven(), r::one() + r::six());
    assert_eq!(r::neg_ten(), r::neg_five() + r::neg_five());
    assert_eq!(r::neg_8_3_2(), r::neg_1_3_2() + r::neg_1_3_2() + r::neg_1_3_2() + r::neg_1_3_2() + r::neg_1_3_2() + r::neg_1_3_2() + r::neg_1_3_2() + r::neg_1_3_2());
}

#[test]
fn neg_r_integers() {
    assert_eq!(r::neg_one(), -r::one());
    assert_eq!(r::zero(), -r::zero());
    assert_eq!(r::neg_five(), -r::five());
    let neg_p_to_third = -radic!(5, [0, 0, 0, 1], []);
    assert_eq!(radic!(5, [0, 0, 0], [4]), neg_p_to_third);
}

#[test]
fn sub_r_integers() {
    assert_eq!(r::one(), r::two() - r::one());
    assert_eq!(r::zero(), r::one() - r::one());
    assert_eq!(r::neg_one(), r::one() - r::two());
    assert_eq!(r::neg_five(), r::one() - r::six());
}

#[test]
fn mul_r_integers() {
    let check = |c: &RAdic, a: &RAdic, b: &RAdic| {
        assert_eq!(*c, a * b);
        assert_eq!(*c, b * a);
    };
    check(&r::one(), &r::one(), &r::one());
    check(&r::two(), &r::two(), &r::one());
    check(&r::six(), &r::two(), &r::three());
    check(&r::six(), &r::three(), &r::two());
    for num in [&r::zero(), &r::one(), &r::two(), &r::three(), &r::four(), &r::five(), &r::six(), &r::neg_one()] {
        check(&r::zero(), &r::zero(), num);
        check(&r::zero(), num, &r::zero());
    }
    check(&r::neg_one(), &r::one(), &r::neg_one());
    check(&r::neg_two(), &r::two(), &r::neg_one());
    check(&r::neg_four(), &r::two(), &r::neg_two());
    check(&r::one(), &r::neg_one(), &r::neg_one());
    check(&r::six(), &r::neg_two(), &r::neg_three());
    check(&r::ten(), &r::five(), &r::two());
    check(&r::twenty_five(), &r::five(), &r::five());
    check(&r::twenty_five(), &r::neg_five(), &r::neg_five());
}

#[test]
fn add_sub_r() {

    assert_eq!(r::neg_1_4(), -r::pos_1_4());
    assert_eq!(r::pos_1_4(), -r::neg_1_4());
    assert_eq!(r::pos_43_4(), r::neg_1_4() + r::eleven());
    assert_eq!(-r::pos_43_4(), r::pos_1_4() - r::eleven());
    assert_eq!(r::neg_1_24() + r::neg_1_24() + r::neg_1_24() + r::neg_1_24() + r::neg_1_24(), r::neg_5_24());
    assert_eq!(r::neg_1_24() + r::neg_1_24() + r::neg_1_24() + r::neg_1_24() + r::neg_1_24() + r::neg_1_24(), r::neg_1_4());

    assert_eq!(radic!(5, [0, 1], [0, 4, 0]), r::pos_30_31());
    assert_eq!(radic!(5, [], [0, 4, 0]), r::neg_5_31());
    assert_eq!(radic!(5, [], []), r::pos_30_31() + r::neg_30_31());

    assert_eq!(radic!(5, [2, 1], [4, 0]), r::pos_17_6());
    assert_eq!(uadic!(5, [2, 1, 4, 0, 4, 0]), r::pos_17_6().into_truncation(6));

}

#[test]
fn mul_r() {

    assert_eq!(radic!(5, [1], [2, 3, 4, 0]), r::neg_1_4() * r::neg_1_4());
    assert_eq!(radic!(5, [], [4]), r::neg_1_4() * r::four());
    assert_eq!(r::neg_1_4(), r::neg_1_24() * r::six());
    assert_eq!(r::neg_5_24(), r::neg_1_24() * r::five());

    // 3-adic
    let neg_9_2 = radic!(3, [0, 0], [1]);
    assert_eq!(Rational32::new(-9, 2), neg_9_2.rational_value());
    let pos_81_4 = radic!(3, [0, 0, 0, 0, 1], [2, 0]);
    assert_eq!(Rational32::new(81, 4), pos_81_4.rational_value());
    let neg_729_8 = radic!(3, [0, 0, 0, 0, 0], [0, 1]);
    assert_eq!(Rational32::new(-729, 8), neg_729_8.rational_value());
    assert_eq!(pos_81_4, &neg_9_2 * &neg_9_2);
    assert_eq!(neg_729_8, &pos_81_4 * &neg_9_2);
    assert_eq!(neg_729_8, &pos_81_4 * &neg_9_2);

    // 7-adic
    let neg_1_6_sq = radic!(7, [], [1]) * radic!(7, [], [1]);
    assert_eq!(radic!(7, [1], [2, 3, 4, 5, 6, 0]), neg_1_6_sq);

    // 2-adic
    assert_eq!(r::one_2(), r::neg_one_2() * r::neg_one_2());
    assert_eq!(r::pos_1_9_2(), r::neg_1_3_2() * r::neg_1_3_2());
    assert_eq!(r::neg_8_3_2(), r::eight_2() * r::neg_1_3_2());
    assert_eq!(r::pos_64_9_2(), r::neg_8_3_2() * r::neg_8_3_2());

}

#[test]
fn pow_r_adic() {

    assert_eq!(r::zero(), r::zero().pow(2));
    assert_eq!(r::zero(), r::zero().pow(3));
    assert_eq!(r::one(), r::one().pow(2));
    assert_eq!(r::one(), r::one().pow(3));
    assert_eq!(r::four(), r::two().pow(2));
    assert_eq!(r::eight(), r::two().pow(3));
    assert_eq!(r::nine(), r::three().pow(2));
    assert_eq!(r::twenty_five(), r::five().pow(2));
    assert_eq!(r::one(), r::neg_two().pow(0));
    assert_eq!(r::neg_one(), r::neg_one().pow(1));
    assert_eq!(r::one(), r::neg_one().pow(2));
    assert_eq!(r::four(), r::neg_two().pow(2));
    assert_eq!(r::twenty_five(), r::neg_five().pow(2));
    assert_eq!(r::pos_1_16(), r::neg_1_4().pow(2));
    assert_eq!(r::neg_1_64(), r::neg_1_4().pow(3));
    assert_eq!(r::pos_25_16(), r::neg_5_4().pow(2));

    assert_eq!(r::zero_2(), r::zero_2().pow(2));
    assert_eq!(r::one_2(), r::one_2().pow(2));
    assert_eq!(r::one_2(), r::neg_one_2().pow(2));
    assert_eq!(r::neg_one_2(), r::neg_one_2().pow(3));
    assert_eq!(r::pos_1_9_2(), r::neg_1_3_2().pow(2));
    assert_eq!(r::pos_64_9_2(), r::neg_8_3_2().pow(2));

}

#[test]
fn div_r_adic() {
    assert_eq!(Ok(r::one().into_approximation(4)), (r::one() / r::one()).approx(4));
    assert_eq!(Ok(r::two().into_approximation(4)), (r::two() / r::one()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [2, 3, 1, 3])), (r::one() / r::three()).approx(4));
    assert_eq!(Ok(r::neg_1_24().into_approximation(4)), (r::neg_one() / r::twenty_four()).approx(4));
    assert_eq!(Ok(r::zero().into_approximation(4)), (r::one() / r::five()).approx(4));
    assert_eq!(Ok(r::pos_1_2().quotient(1).into_approximation(4)), (r::one() / r::ten()).approx(4));
    assert_eq!(Ok(r::pos_1_3().quotient(1).into_approximation(4)), (r::one() / r::fifteen()).approx(4));
    assert_eq!(Ok(r::zero().into_approximation(4)), (r::one() / r::twenty_five()).approx(4));
    assert_eq!(Ok(r::four().into_approximation(4)), (r::one() / r::pos_1_4()).approx(4));
    assert_eq!(Ok(r::neg_four().into_approximation(4)), (r::one() / r::neg_1_4()).approx(4));
    assert_eq!(Ok(r::sixteen().into_approximation(4)), (r::one() / r::pos_1_16()).approx(4));
    assert_eq!(Ok(r::sixteen().into_approximation(4)), (r::four() / r::pos_1_4()).approx(4));
    assert_eq!(Ok(r::pos_3_2().into_approximation(4)), (r::neg_1_4() / r::neg_1_6()).approx(4));
}

#[ignore = "Takes five minutes"]
#[test]
fn r_adic_ops_many() {
    // Test addition and multiplication over many rationals using rational_value
    let p = 5;
    let fix_n = 2;
    let rep_n = 2;
    let firsts = repeat_n(0..p, fix_n).multi_cartesian_product().cartesian_product(
        repeat_n(0..p, rep_n).multi_cartesian_product()
    ).map(
        |(fixed_digits, repeat_digits)| RAdic::new(p, fixed_digits, repeat_digits)
    );
    let seconds = firsts.clone();
    for (first, second) in firsts.cartesian_product(seconds) {
        let first_val = first.big_rational_value();
        let second_val = second.big_rational_value();
        let sum_val = (&first + &second).big_rational_value();
        let prod_val = (&first * &second).big_rational_value();
        assert_eq!(&first_val + &second_val, sum_val);
        assert_eq!(&first_val * &second_val, prod_val);
    }
}



// ZAdic

#[test]
fn add_z_adic() {
    assert_eq!(z::two_e(), z::one_e() + z::one_e());
    assert_eq!(z::three_e(), z::two_e() + z::one_e());
    assert_eq!(z::five_e(), z::two_e() + z::three_e());
    let neg_one_plus_neg_one = z::neg_one_4() + z::neg_one_4();
    assert_eq!(z::neg_two_4(), neg_one_plus_neg_one);
    let neg_two_plus_neg_three = z::neg_two_4() + z::neg_three_4();
    assert_eq!(z::neg_five_4(), neg_two_plus_neg_three);
    let neg_five_plus_neg_five = z::neg_five_4() + z::neg_five_4();
    assert_eq!(z::neg_ten_4(), neg_five_plus_neg_five);
    let two_plus_neg_two = z::two_e() + z::neg_two_4();
    assert_eq!(z::zero_4(), two_plus_neg_two);
    let four_plus_one_does_not_grow = zadic_approx!(5, 1, [4]) + zadic_exact_pos!(5, [1]);
    assert_eq!(zadic_approx!(5, 1, [0]), four_plus_one_does_not_grow);
    assert_eq!(z::twenty_five_e().certainty(), ZAdicValuation::PosInf);
    assert_eq!(z::twenty_five_4().certainty(), ZAdicValuation::Finite(4));
}

#[test]
fn neg_z_adic() {
    assert_eq!(z::neg_one_e(), -z::one_e());
    assert_eq!(z::one_e(), -z::neg_one_e());
    assert_eq!(z::zero_e(), -z::zero_e());
    assert_eq!(z::neg_five_e(), -z::five_e());
    assert_eq!(z::neg_three_4(), -z::three_4());
    assert_eq!(z::neg_five_4(), -z::five_4());
    assert_eq!(z::sqrt_2_7_adic2(), -z::sqrt_2_7_adic());
}

#[test]
fn sub_z_adic() {
    assert_eq!(z::one_e(), z::two_e() - z::one_e());
    assert_eq!(z::zero_e(), z::one_e() - z::one_e());
    assert_eq!(z::neg_one_e(), z::one_e() - z::two_e());
    assert_eq!(z::neg_five_e(), z::one_e() - z::six_e());
    assert_eq!(z::one_e(), z::neg_one_e() - z::neg_two_e());
    assert_eq!(z::one_4(), z::neg_one_4() - z::neg_two_4());
}

#[test]
fn mul_z_adic() {
    assert_eq!(z::one_e(), z::one_e() * z::one_e());
    assert_eq!(z::two_e(), z::two_e() * z::one_e());
    assert_eq!(z::six_e(), z::two_e() * z::three_e());
    let neg_one_mul_neg_one = z::neg_one_4() * z::neg_one_4();
    assert_eq!(z::one_4(), neg_one_mul_neg_one);
    let neg_two_mul_neg_three = z::neg_two_4() * z::neg_three_4();
    assert_eq!(z::six_4(), neg_two_mul_neg_three);
    assert_eq!(z::zero_e(), z::zero_e() * z::two_e());
    assert_eq!(z::zero_e(), z::zero_e() * z::neg_two_4());
    assert_eq!(z::ten_e(), z::five_e() * z::two_e());
    assert_eq!(z::twenty_five_e(), z::five_e() * z::five_e());
    assert_eq!(z::one_e(), z::neg_one_e() * z::neg_one_e());
}

#[test]
fn pow_z_adic() {
    assert_eq!(z::zero_e(), z::zero_e().pow(2));
    assert_eq!(z::zero_e(), z::zero_e().pow(3));
    assert_eq!(z::one_e(), z::one_e().pow(2));
    assert_eq!(z::one_e(), z::one_e().pow(3));
    assert_eq!(z::four_e(), z::two_e().pow(2));
    assert_eq!(z::eight_e(), z::two_e().pow(3));
    assert_eq!(z::twenty_five_e(), z::five_e().pow(2));
    assert_eq!(z::one_e(), z::neg_two_4().pow(0));
    assert_eq!(z::neg_one_4(), z::neg_one_4().pow(1));
    assert_eq!(z::one_4(), z::neg_one_4().pow(2));
    assert_eq!(z::four_4(), z::neg_two_4().pow(2));
    let twenty_five_5 = zadic_approx!(5, 5, [0, 0, 1]);
    assert_eq!(twenty_five_5, z::neg_five_4().pow(2));
}

#[test]
fn inv_z_adic() {

    // Valuation-zero inverses retain certainty
    assert_eq!(zadic_approx!(5, 4, [1, 0, 0, 0]), zadic_approx!(5, 4, [1, 0, 0, 0]).inv());
    assert_eq!(zadic_approx!(5, 4, [3, 2, 2, 2]), zadic_approx!(5, 4, [2, 0, 0, 0]).inv());
    assert_eq!(zadic_approx!(5, 4, [2, 0, 3, 4]), zadic_approx!(5, 4, [3, 2, 0, 0]).inv());
    assert_eq!(zadic_approx!(5, 4, [3, 2, 0, 0]), zadic_approx!(5, 4, [2, 0, 3, 4]).inv());

    // Higher valuation lowers certainty
    assert_eq!(zadic_approx!(5, 2, [0, 0]), zadic_approx!(5, 4, [0, 1, 0, 0]).inv());
    assert_eq!(zadic_approx!(5, 2, [2, 2]), zadic_approx!(5, 4, [0, 2, 0, 0]).inv());
    assert_eq!(zadic_approx!(5, 2, [0, 3]), zadic_approx!(5, 4, [0, 3, 2, 0]).inv());
    assert_eq!(zadic_approx!(5, 2, [2, 0]), zadic_approx!(5, 4, [0, 2, 0, 3]).inv());

    // Valuation over half the certainty means an empty inverse
    assert_eq!(ZAdic::empty(5), zadic_approx!(5, 4, [0, 0, 1, 0]).inv());
    assert_eq!(ZAdic::empty(5), zadic_approx!(5, 4, [0, 0, 2, 0]).inv());
    assert_eq!(ZAdic::empty(5), zadic_approx!(5, 4, [0, 0, 3, 2]).inv());
    assert_eq!(ZAdic::empty(5), zadic_approx!(5, 4, [0, 0, 0, 0]).inv());

}

#[test]
fn div_z_adic() {

    let err_msg = "a and b not precise enough to give 4 digits".to_string();
    let prec_err = Err(AdicError::InappropriatePrecision(err_msg));

    assert_eq!(Ok(zadic_approx!(5, 4, [1, 0, 0, 0])), (z::one_4() / z::one_4()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [2, 0, 0, 0])), (z::two_4() / z::one_4()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [1, 0, 0, 0])), (z::two_4() / z::two_4()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [3, 2, 2, 2])), (z::one_4() / z::two_4()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [2, 2, 2, 2])), (z::neg_one_4() / z::two_4()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [2, 2, 2, 2])), (z::one_4() / z::neg_two_4()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [3, 2, 2, 2])), (z::neg_one_4() / z::neg_two_4()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [2, 3, 1, 3])), (z::one_4() / z::three_4()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [4, 4, 3, 4])), (z::one_4() / z::twenty_four_4()).approx(4));
    assert_eq!(prec_err, (z::one_4() / z::five_4()).approx(4));
    assert_eq!(
        Err(AdicError::InappropriatePrecision("a and b not precise enough to give 3 digits".to_string())),
        (z::one_4() / z::five_4()).approx(3)
    );
    assert_eq!(Ok(zadic_approx!(5, 2, [0, 0])), (z::one_4() / z::five_4()).approx(2));
    assert_eq!(prec_err, (z::one_4() / z::five_4()).approx(4));
    assert_eq!(prec_err, (z::one_e() / z::five_4()).approx(4));
    assert_eq!(prec_err, (z::one_4() / z::five_e()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [0, 0, 0, 0])), (z::one_e() / z::five_e()).approx(4));
    assert_eq!(prec_err, (z::one_4() / z::ten_4()).approx(4));
    assert_eq!(prec_err, (z::one_4() / z::fifteen_4()).approx(4));
    assert_eq!(Ok(z::five_4()), (z::five_4() / z::one_4()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [0, 3, 2, 2])), (z::five_4() / z::two_4()).approx(4));
    assert_eq!(Ok(zadic_approx!(5, 3, [0, 0, 0])), (z::one_twenty_five_4() / z::one_4()).approx(3));
    assert_eq!(Ok(zadic_approx!(5, 2, [0, 0])), (z::one_twenty_five_4() / z::one_4()).approx(2));
    assert_eq!(Ok(zadic_approx!(5, 1, [0])), (z::one_twenty_five_4() / z::one_4()).approx(1));
    assert_eq!(Ok(zadic_approx!(5, 2, [0, 0])), (z::one_twenty_five_4() / z::five_4()).approx(2));
    assert_eq!(Ok(zadic_approx!(5, 1, [0])), (z::one_twenty_five_4() / z::five_4()).approx(1));
    assert_eq!(Ok(zadic_approx!(5, 1, [0])), (z::one_twenty_five_4() / z::twenty_five_4()).approx(1));

}
