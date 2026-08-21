use itertools::{Itertools, repeat_n};
use num::{
    traits::{Inv, Pow},
    CheckedDiv, Rational32,
};
use crate::traits::{AdicPrimitive, CanApproximate, CanTruncate, PrimedFrom, TryPrimedFrom};
use super::EAdic;

use crate::num_adic::test_util::e;



// IAdic

#[test]
fn add_i_adic() {
    assert_eq!(e::two(), e::one() + e::one());
    assert_eq!(e::three(), e::two() + e::one());
    assert_eq!(e::five(), e::two() + e::three());
    let neg_one_plus_neg_one = e::neg_one() + e::neg_one();
    assert_eq!(e::neg_two(), neg_one_plus_neg_one);
    let neg_two_plus_neg_three = e::neg_two() + e::neg_three();
    assert_eq!(e::neg_five(), neg_two_plus_neg_three);
    let neg_five_plus_neg_five = e::neg_five() + e::neg_five();
    assert_eq!(e::neg_ten(), neg_five_plus_neg_five);
    let two_plus_neg_two = e::two() + e::neg_two();
    assert_eq!(e::zero(), two_plus_neg_two);
}

#[test]
fn neg_i_adic() {
    assert_eq!(e::neg_one(), -e::one());
    assert_eq!(e::zero(), -e::zero());
    assert_eq!(e::neg_five(), -e::five());
    let neg_p_to_third = -eadic!(5, [0, 0, 0, 1]);
    assert_eq!(eadic_neg!(5, [0, 0, 0]), neg_p_to_third);
}

#[test]
fn sub_i_adic() {
    assert_eq!(e::one(), e::two() - e::one());
    assert_eq!(e::zero(), e::one() - e::one());
    assert_eq!(e::neg_one(), e::one() - e::two());
    assert_eq!(e::neg_five(), e::one() - e::six());
}

#[test]
fn mul_i_adic() {
    assert_eq!(e::zero(), e::zero() * e::one());
    assert_eq!(e::zero(), e::zero() * e::neg_one());
    assert_eq!(e::one(), e::one() * e::one());
    assert_eq!(e::two(), e::two() * e::one());
    assert_eq!(e::six(), e::two() * e::three());
    let neg_one_mul_neg_one = e::neg_one() * e::neg_one();
    assert_eq!(e::one(), neg_one_mul_neg_one);
    let neg_two_mul_neg_three = e::neg_two() * e::neg_three();
    assert_eq!(e::six(), neg_two_mul_neg_three);
    assert_eq!(e::zero(), e::zero() * e::two());
    assert_eq!(e::zero(), e::zero() * e::neg_two());
    assert_eq!(e::ten(), e::five() * e::two());
    assert_eq!(e::twenty_five(), e::five() * e::five());
    assert_eq!(e::neg_one(), e::one() * e::neg_one());
    assert_eq!(e::neg_one(), e::neg_one() * e::one());
    assert_eq!(e::one(), e::neg_one() * e::neg_one());
    assert_eq!(e::neg_one(), e::neg_one() * e::neg_one() * e::neg_one());
    assert_eq!(e::neg_two(), e::neg_one() * e::two());
    assert_eq!(e::neg_two(), e::neg_two() * e::one());
    assert_eq!(e::neg_ten(), e::neg_two() * e::five());
    assert_eq!(e::neg_ten(), e::neg_five() * e::two());
}

#[test]
fn pow_i_adic() {
    assert_eq!(e::zero(), e::zero().pow(2));
    assert_eq!(e::zero(), e::zero().pow(3));
    assert_eq!(e::one(), e::one().pow(2));
    assert_eq!(e::one(), e::one().pow(3));
    assert_eq!(e::four(), e::two().pow(2));
    assert_eq!(e::eight(), e::two().pow(3));
    assert_eq!(e::twenty_five(), e::five().pow(2));
    assert_eq!(e::one(), e::neg_two().pow(0));
    assert_eq!(e::neg_one(), e::neg_one().pow(1));
    assert_eq!(e::one(), e::neg_one().pow(2));
    assert_eq!(e::neg_one(), e::neg_one().pow(3));
    assert_eq!(e::four(), e::neg_two().pow(2));
}

#[test]
fn div_i_adic() {

    assert_eq!(e::one(), e::one() / e::one());
    assert_eq!(e::two(), e::two() / e::one());
    assert_eq!(e::pos_1_2(), e::one() / e::two());
    assert_eq!(e::neg_1_2(), e::neg_one() / e::two());
    assert_eq!(e::neg_1_2(), e::one() / e::neg_two());
    assert_eq!(e::pos_1_2(), e::neg_one() / e::neg_two());
    assert_eq!(e::pos_1_3(), e::one() / e::three());
    assert_eq!(e::pos_1_24(), e::one() / e::twenty_four());
    assert_eq!(e::zero(), e::one() / e::five());
    assert_eq!(e::neg_1_2(), e::one() / e::ten());
    assert_eq!(e::neg_1_3(), e::one() / e::fifteen());
    assert_eq!(e::zero(), e::one() / e::twenty_five());

    assert_eq!(e::neg_two(), e::two() / e::neg_one());
    assert_eq!(e::two(), e::neg_two() / e::neg_one());
    assert_eq!(e::neg_1_24(), e::neg_one() / e::twenty_four());
    assert_eq!(None, e::one().checked_div(&e::zero()));

}

#[test]
fn i_adic_ops_many() {
    // Test addition and multiplication over many integers using i32_value
    let p = 5;
    let n1 = 2;
    let n2 = 2;
    let firsts = repeat_n(0..p, n1).multi_cartesian_product().flat_map(
        |digits| [EAdic::new(p, digits[0..n1].to_vec()), EAdic::new_neg(p, digits[0..n1].to_vec())]
    );
    let seconds = repeat_n(0..p, n2).multi_cartesian_product().flat_map(
        |digits| [EAdic::new(p, digits[0..n2].to_vec()), EAdic::new_neg(p, digits[0..n2].to_vec())]
    );
    for (first, second) in firsts.cartesian_product(seconds) {
        let first_val = first.i32_value().unwrap();
        let second_val = second.i32_value().unwrap();
        let sum_val = (first.clone() + second.clone()).i32_value().unwrap();
        let prod_val = (first * second).i32_value().unwrap();
        assert_eq!(first_val + second_val, sum_val);
        assert_eq!(first_val * second_val, prod_val);
    }
}



// RAdic

#[test]
fn add_r_integers() {
    assert_eq!(e::three(), e::two() + e::one());
    assert_eq!(e::two(), e::one() + e::one());
    assert_eq!(e::two() + e::one(), e::one() + e::two());
    assert_eq!(e::seven(), e::one() + e::six());
    assert_eq!(e::neg_ten(), e::neg_five() + e::neg_five());
    assert_eq!(e::neg_8_3_2(), e::neg_1_3_2() + e::neg_1_3_2() + e::neg_1_3_2() + e::neg_1_3_2() + e::neg_1_3_2() + e::neg_1_3_2() + e::neg_1_3_2() + e::neg_1_3_2());
}

#[test]
fn neg_r_integers() {
    assert_eq!(e::neg_one(), -e::one());
    assert_eq!(e::zero(), -e::zero());
    assert_eq!(e::neg_five(), -e::five());
    let neg_p_to_third = -eadic_rep!(5, [0, 0, 0, 1], []);
    assert_eq!(eadic_rep!(5, [0, 0, 0], [4]), neg_p_to_third);
    assert_eq!(e::neg_six(), -e::six());
}

#[test]
fn sub_r_integers() {
    assert_eq!(e::one(), e::two() - e::one());
    assert_eq!(e::zero(), e::one() - e::one());
    assert_eq!(e::neg_one(), e::one() - e::two());
    assert_eq!(e::neg_five(), e::one() - e::six());
}

#[test]
fn mul_r_integers() {
    let check = |c: &EAdic, a: &EAdic, b: &EAdic| {
        assert_eq!(*c, a.clone() * b.clone());
        assert_eq!(*c, b.clone() * a.clone());
    };
    check(&e::one(), &e::one(), &e::one());
    check(&e::two(), &e::two(), &e::one());
    check(&e::six(), &e::two(), &e::three());
    check(&e::six(), &e::three(), &e::two());
    for num in [&e::zero(), &e::one(), &e::two(), &e::three(), &e::four(), &e::five(), &e::six(), &e::neg_one()] {
        check(&e::zero(), &e::zero(), num);
        check(&e::zero(), num, &e::zero());
    }
    check(&e::neg_one(), &e::one(), &e::neg_one());
    check(&e::neg_two(), &e::two(), &e::neg_one());
    check(&e::neg_four(), &e::two(), &e::neg_two());
    check(&e::one(), &e::neg_one(), &e::neg_one());
    check(&e::six(), &e::neg_two(), &e::neg_three());
    check(&e::ten(), &e::five(), &e::two());
    check(&e::twenty_five(), &e::five(), &e::five());
    check(&e::twenty_five(), &e::neg_five(), &e::neg_five());
}

#[test]
fn add_sub_r() {

    assert_eq!(e::neg_1_4(), -e::pos_1_4());
    assert_eq!(e::pos_1_4(), -e::neg_1_4());
    assert_eq!(e::pos_43_4(), e::neg_1_4() + e::eleven());
    assert_eq!(-e::pos_43_4(), e::pos_1_4() - e::eleven());
    assert_eq!(e::neg_1_24() + e::neg_1_24() + e::neg_1_24() + e::neg_1_24() + e::neg_1_24(), e::neg_5_24());
    assert_eq!(e::neg_1_24() + e::neg_1_24() + e::neg_1_24() + e::neg_1_24() + e::neg_1_24() + e::neg_1_24(), e::neg_1_4());

    assert_eq!(eadic_rep!(5, [0, 1], [0, 4, 0]), e::pos_30_31());
    assert_eq!(eadic_rep!(5, [], [0, 4, 0]), e::neg_5_31());
    assert_eq!(eadic_rep!(5, [], []), e::pos_30_31() + e::neg_30_31());

    assert_eq!(eadic_rep!(5, [2, 1], [4, 0]), e::pos_17_6());
    assert_eq!(uadic!(5, [2, 1, 4, 0, 4, 0]), e::pos_17_6().into_truncation(6));

}

#[test]
fn mul_r() {

    assert_eq!(eadic_rep!(5, [1], [2, 3, 4, 0]), e::neg_1_4() * e::neg_1_4());
    assert_eq!(eadic_rep!(5, [], [4]), e::neg_1_4() * e::four());
    assert_eq!(e::neg_1_4(), e::neg_1_24() * e::six());
    assert_eq!(e::neg_5_24(), e::neg_1_24() * e::five());

    // 3-adic
    let neg_9_2 = eadic_rep!(3, [0, 0], [1]);
    assert_eq!(Ok(Rational32::new(-9, 2)), neg_9_2.rational_value());
    let pos_81_4 = eadic_rep!(3, [0, 0, 0, 0, 1], [2, 0]);
    assert_eq!(Ok(Rational32::new(81, 4)), pos_81_4.rational_value());
    let neg_729_8 = eadic_rep!(3, [0, 0, 0, 0, 0], [0, 1]);
    assert_eq!(Ok(Rational32::new(-729, 8)), neg_729_8.rational_value());
    assert_eq!(pos_81_4, neg_9_2.clone() * neg_9_2.clone());
    assert_eq!(neg_729_8, pos_81_4.clone() * neg_9_2.clone());
    assert_eq!(neg_729_8, pos_81_4 * neg_9_2);

    // 7-adic
    let neg_1_6_sq = eadic_rep!(7, [], [1]) * eadic_rep!(7, [], [1]);
    assert_eq!(eadic_rep!(7, [1], [2, 3, 4, 5, 6, 0]), neg_1_6_sq);

    // 2-adic
    assert_eq!(e::one_2(), e::neg_one_2() * e::neg_one_2());
    assert_eq!(e::pos_1_9_2(), e::neg_1_3_2() * e::neg_1_3_2());
    assert_eq!(e::neg_8_3_2(), e::eight_2() * e::neg_1_3_2());
    assert_eq!(e::pos_64_9_2(), e::neg_8_3_2() * e::neg_8_3_2());

}

#[test]
fn pow_r_adic() {

    assert_eq!(e::zero(), e::zero().pow(2));
    assert_eq!(e::zero(), e::zero().pow(3));
    assert_eq!(e::one(), e::one().pow(2));
    assert_eq!(e::one(), e::one().pow(3));
    assert_eq!(e::four(), e::two().pow(2));
    assert_eq!(e::eight(), e::two().pow(3));
    assert_eq!(e::nine(), e::three().pow(2));
    assert_eq!(e::twenty_five(), e::five().pow(2));
    assert_eq!(e::one(), e::neg_two().pow(0));
    assert_eq!(e::neg_one(), e::neg_one().pow(1));
    assert_eq!(e::one(), e::neg_one().pow(2));
    assert_eq!(e::four(), e::neg_two().pow(2));
    assert_eq!(e::twenty_five(), e::neg_five().pow(2));
    assert_eq!(e::pos_1_16(), e::neg_1_4().pow(2));
    assert_eq!(e::neg_1_64(), e::neg_1_4().pow(3));
    assert_eq!(e::pos_25_16(), e::neg_5_4().pow(2));

    assert_eq!(e::zero_2(), e::zero_2().pow(2));
    assert_eq!(e::one_2(), e::one_2().pow(2));
    assert_eq!(e::one_2(), e::neg_one_2().pow(2));
    assert_eq!(e::neg_one_2(), e::neg_one_2().pow(3));
    assert_eq!(e::pos_1_9_2(), e::neg_1_3_2().pow(2));
    assert_eq!(e::pos_64_9_2(), e::neg_8_3_2().pow(2));

}

#[test]
fn inv_r_adic() {

    // Many adic integers have the same inverse
    assert_eq!(e::pos_1_3(), e::three().inv());

    // Unit inverses are still units
    assert_eq!(e::one(), e::one().inv());
    assert_eq!(e::pos_1_2(), e::two().inv());
    assert_eq!(e::two(), e::pos_1_2().inv());
    assert_eq!(e::pos_1_16(), e::sixteen().inv());
    assert_eq!(e::sixteen(), e::pos_1_16().inv());
    assert_eq!(e::neg_two(), e::neg_1_2().inv());
    assert_eq!(e::neg_1_2(), e::neg_two().inv());
    assert_eq!(-e::four(), e::neg_1_4().inv());
    assert_eq!(-e::pos_1_4(), e::neg_four().inv());
    assert_eq!(-e::six(), e::neg_1_6().inv());
    assert_eq!(-e::pos_1_6(), e::neg_six().inv());

    // Non-unit inverses (positive valuation) truncate the inverse
    assert_eq!(eadic_rep!(5, [1], [4, 0]), e::six().inv());
    assert_eq!(eadic_rep!(5, [], [4, 0]), e::thirty().inv());

    assert_eq!(e::neg_five_2() / e::three_2(), e::neg_five_2() * e::three_2().inv());

}

#[test]
#[should_panic]
fn inv_zero_panics() {
    EAdic::zero(5).inv();
}

#[test]
fn div_r_adic() {

    assert_eq!(e::one().into_approximation(4), (e::one() / e::one()).approximation(4));
    assert_eq!(e::two().into_approximation(4), (e::two() / e::one()).approximation(4));
    assert_eq!(zadic_approx!(5, 4, [2, 3, 1, 3]), (e::one() / e::three()).approximation(4));
    assert_eq!(e::neg_1_24().into_approximation(4), (e::neg_one() / e::twenty_four()).approximation(4));
    assert_eq!(e::zero().into_approximation(4), (e::one() / e::five()).approximation(4));
    assert_eq!(e::pos_1_2().quotient(1).into_approximation(4), (e::one() / e::ten()).approximation(4));
    assert_eq!(e::pos_1_3().quotient(1).into_approximation(4), (e::one() / e::fifteen()).approximation(4));
    assert_eq!(e::zero().into_approximation(4), (e::one() / e::twenty_five()).approximation(4));
    assert_eq!(e::four().into_approximation(4), (e::one() / e::pos_1_4()).approximation(4));
    assert_eq!(e::neg_four().into_approximation(4), (e::one() / e::neg_1_4()).approximation(4));
    assert_eq!(e::sixteen().into_approximation(4), (e::one() / e::pos_1_16()).approximation(4));
    assert_eq!(e::sixteen().into_approximation(4), (e::four() / e::pos_1_4()).approximation(4));
    assert_eq!(e::pos_3_2().into_approximation(4), (e::neg_1_4() / e::neg_1_6()).approximation(4));
    assert_eq!(-e::neg_1_6().into_approximation(4), (e::neg_5_24() / e::neg_5_4()).approximation(4));

    assert_eq!(e::two(), (e::two() / e::one()));
    assert_eq!(e::eight(), (e::two() / e::pos_1_4()));
    assert_eq!(e::pos_1_8(), (e::pos_1_4() / e::two()));
    assert_eq!(e::zero(), (e::one() / e::five()));
    assert_eq!(None, e::one().checked_div(&e::zero()));

    assert_eq!(e::neg_1_3_2(), (e::neg_one_2() / e::three_2()));
    assert_eq!(e::neg_1_3_2(), (-e::one_2() / e::three_2()));
    assert_eq!(e::neg_5_3_2(), (e::neg_five_2() / e::three_2()));
    assert_eq!(e::neg_5_3_2(), (-e::five_2() / e::three_2()));

    assert_eq!(
        EAdic::try_primed_from(3, Rational32::new(-5, 2)).unwrap(),
        EAdic::primed_from(3, -5) / EAdic::primed_from(3, 2)
    );
    assert_eq!(
        EAdic::try_primed_from(3, Rational32::new(-5, 4)).unwrap(),
        EAdic::primed_from(3, -5) / EAdic::primed_from(3, 4)
    );

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
        |(fixed_digits, repeat_digits)| EAdic::new_repeating(p, fixed_digits, repeat_digits)
    );
    let seconds = firsts.clone();
    for (first, second) in firsts.cartesian_product(seconds) {
        let first_val = first.big_rational_value();
        let second_val = second.big_rational_value();
        let sum_val = (first.clone() + second.clone()).big_rational_value();
        let prod_val = (first * second).big_rational_value();
        assert_eq!(&first_val + &second_val, sum_val);
        assert_eq!(&first_val * &second_val, prod_val);
    }
}

#[test]
fn rem_e_adic() {

    assert_eq!(e::zero(), e::one() % e::one());
    assert_eq!(e::zero(), e::two() % e::one());
    assert_eq!(e::zero(), e::one() % e::two());
    assert_eq!(e::zero(), e::neg_one() % e::two());
    assert_eq!(e::zero(), e::one() % e::neg_two());
    assert_eq!(e::zero(), e::neg_one() % e::neg_two());
    assert_eq!(e::zero(), e::one() % e::three());
    assert_eq!(e::zero(), e::one() % e::twenty_four());
    assert_eq!(e::one(), e::one() % e::five());
    assert_eq!(e::three(), e::one() % e::ten());
    assert_eq!(e::two(), e::one() % e::fifteen());
    assert_eq!(e::one(), e::one() % e::twenty_five());
    assert_eq!(e::twenty_four(), e::twenty_four() % e::twenty_five());
    assert_eq!(e::zero(), e::twenty_five() % e::twenty_five());
    assert_eq!(e::one(), e::twenty_six() % e::twenty_five());

}
