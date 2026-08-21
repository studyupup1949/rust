use itertools::{Itertools, repeat_n};
use num::traits::Pow;
use crate::traits::CanTruncate;
use super::UAdic;

use crate::num_adic::test_util::u;


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
        let first_val = first.u32_value().unwrap();
        let second_val = second.u32_value().unwrap();
        let sum_val = (first.clone() + second.clone()).u32_value().unwrap();
        let prod_val = (first * second).u32_value().unwrap();
        assert_eq!(first_val + second_val, sum_val);
        assert_eq!(first_val * second_val, prod_val);
    }
}
