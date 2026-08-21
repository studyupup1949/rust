use num::{
    traits::{Inv, Pow},
    CheckedDiv,
};
use crate::{
    normed::Valuation,
    traits::{AdicPrimitive, CanApproximate, HasApproximateDigits},
};
use super::ZAdic;

use crate::num_adic::test_util::{e, z};


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
    let four_plus_one_does_not_grow = zadic_approx!(5, 1, [4]) + ZAdic::from(uadic!(5, [1]));
    assert_eq!(zadic_approx!(5, 1, [0]), four_plus_one_does_not_grow);
    assert_eq!(z::twenty_five_e().certainty(), Valuation::PosInf);
    assert_eq!(z::twenty_five_4().certainty(), Valuation::Finite(4));
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

    // Approximate inverses of exact adic units as expected
    assert_eq!(e::one().into_approximation(4), e::one().inv().approximation(4));
    assert_eq!(e::pos_1_2().into_approximation(4), e::two().inv().approximation(4));
    assert_eq!(e::two().into_approximation(4), e::pos_1_2().inv().approximation(4));
    assert_eq!(e::pos_1_16().into_approximation(4), e::sixteen().inv().approximation(4));
    assert_eq!(e::sixteen().into_approximation(4), e::pos_1_16().inv().approximation(4));
    assert_eq!(e::neg_two().into_approximation(4), e::neg_1_2().inv().approximation(4));
    assert_eq!(e::neg_1_2().into_approximation(4), e::neg_two().inv().approximation(4));
    assert_eq!(-e::four().into_approximation(4), e::neg_1_4().inv().approximation(4));
    assert_eq!(-e::pos_1_4().into_approximation(4), e::neg_four().inv().approximation(4));
    assert_eq!(-e::six().into_approximation(4), e::neg_1_6().inv().approximation(4));
    assert_eq!(-e::pos_1_6().into_approximation(4), e::neg_six().inv().approximation(4));

    // Many adic integers have the same inverse
    let z_1_3 = zadic_approx!(5, 4, [2, 3, 1, 3]);
    assert_eq!(z_1_3.clone(), e::three().inv().approximation(4));
    assert_eq!(z_1_3.clone(), e::three().inv().approximation(4));
    assert_eq!(z_1_3.clone(), z::three_e().inv().approximation(4));
    assert_eq!(z_1_3.clone(), z::three_4().inv().approximation(4));

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
    assert_eq!(ZAdic::empty(5), zadic_approx!(5, 4, [0, 0, 0, 1]).inv());
    assert_eq!(ZAdic::empty(5), zadic_approx!(5, 4, [0, 0, 0, 0]).inv());

    // Non-unit inverses (positive valuation) truncate the inverse
    assert_eq!(zadic_approx!(5, 4, [4, 3, 2, 1]), zadic_approx!(5, 4, [4, 0, 1, 0]).inv());
    assert_eq!(zadic_approx!(5, 2, [3, 2]), zadic_approx!(5, 4, [0, 4, 0, 1]).inv());

}

#[test]
fn div_z_adic() {

    assert_eq!(zadic_approx!(5, 4, [1, 0, 0, 0]), z::one_4() / z::one_4());
    assert_eq!(zadic_approx!(5, 4, [2, 0, 0, 0]), z::two_4() / z::one_4());
    assert_eq!(zadic_approx!(5, 4, [1, 0, 0, 0]), z::two_4() / z::two_4());
    assert_eq!(zadic_approx!(5, 4, [3, 2, 2, 2]), z::one_4() / z::two_4());
    assert_eq!(zadic_approx!(5, 4, [2, 2, 2, 2]), z::neg_one_4() / z::two_4());
    assert_eq!(zadic_approx!(5, 4, [2, 2, 2, 2]), z::one_4() / z::neg_two_4());
    assert_eq!(zadic_approx!(5, 4, [3, 2, 2, 2]), z::neg_one_4() / z::neg_two_4());
    assert_eq!(zadic_approx!(5, 4, [2, 3, 1, 3]), z::one_4() / z::three_4());
    assert_eq!(zadic_approx!(5, 4, [4, 4, 3, 4]), z::one_4() / z::twenty_four_4());
    assert_eq!(zadic_approx!(5, 2, [0, 0]), z::one_4() / z::five_4());
    assert_eq!(zadic_approx!(5, 3, [0, 0, 0]), z::one_4() / z::five_e());
    assert_eq!(e::zero().approximation(2), z::one_4() / z::five_4());
    assert_eq!(e::zero().approximation(2), z::one_e() / z::five_4());
    assert_eq!(e::zero().approximation(3), z::one_4() / z::five_e());
    assert_eq!(z::zero_e(), z::one_e() / z::five_e());
    assert_eq!(zadic_approx!(5, 2, [2, 2]), z::one_4() / z::ten_4());
    assert_eq!(zadic_approx!(5, 2, [3, 1]), z::one_4() / z::fifteen_4());
    assert_eq!(z::five_4(), z::five_4() / z::one_4());
    assert_eq!(zadic_approx!(5, 4, [0, 3, 2, 2]), z::five_4() / z::two_4());
    assert_eq!(zadic_approx!(5, 3, [0, 0, 0]), (z::one_twenty_five_4() / z::one_4()).approximation(3));
    assert_eq!(zadic_approx!(5, 2, [0, 0]), (z::one_twenty_five_4() / z::one_4()).approximation(2));
    assert_eq!(zadic_approx!(5, 1, [0]), (z::one_twenty_five_4() / z::one_4()).approximation(1));
    assert_eq!(zadic_approx!(5, 2, [0, 0]), (z::one_twenty_five_4() / z::five_4()).approximation(2));
    assert_eq!(zadic_approx!(5, 1, [0]), (z::one_twenty_five_4() / z::five_4()).approximation(1));
    assert_eq!(zadic_approx!(5, 1, [0]), (z::one_twenty_five_4() / z::twenty_five_4()).approximation(1));
    assert_eq!(zadic_approx!(5, 3, [0, 0, 1]), (z::one_twenty_five_4() / z::five_4()).approximation(3));
    assert_eq!(zadic_approx!(5, 3, [0, 1, 1]), (z::neg_five_e().approximation(3) / z::four_4()).approximation(3));

    let four = z::four_e();
    let twenty = z::twenty_e();
    let neg_five = z::neg_five_e();
    let neg_one = z::neg_one_e();

    assert_eq!(zadic_approx!(5, 2, [1, 1]), (neg_one.clone() / four.approximation(2)));
    assert_eq!(zadic_approx!(5, 2, [1, 1]), (neg_one.approximation(3) / four.approximation(2)));
    assert_eq!(zadic_approx!(5, 4, [1, 1, 1, 1]), (neg_one.clone() / four.approximation(4)));
    assert_eq!(zadic_approx!(5, 3, [1, 1, 1]), (neg_one.approximation(3) / four.approximation(4)));
    assert_eq!(zadic_approx!(5, 3, [1, 1, 1]), (neg_one.approximation(4) / four.approximation(3)));
    assert_eq!(zadic_approx!(5, 2, [1, 1]), (neg_one.approximation(3) / twenty.approximation(4)));
    assert_eq!(zadic_approx!(5, 1, [1]), (neg_one.approximation(4) / twenty.approximation(3)));
    assert_eq!(zadic_approx!(5, 2, [1, 1]), (neg_five.approximation(3) / twenty.approximation(4)));
    assert_eq!(zadic_approx!(5, 2, [1, 1]), (neg_five.approximation(4) / twenty.approximation(3)));
    assert_eq!(zadic_approx!(5, 3, [0, 1, 1]), (neg_five.approximation(3) / four.approximation(4)));
    assert_eq!(zadic_approx!(5, 4, [0, 1, 1, 1]), (neg_five.approximation(4) / four.approximation(3)));
    assert_eq!(ZAdic::from(eadic_rep!(5, [], [1])), neg_one.clone() / four.clone());
    assert_eq!(None, neg_one.clone().checked_div(&ZAdic::zero(5)));

}

#[test]
fn rem_z_adic() {

    assert_eq!(z::zero_e(), z::one_4() % z::one_4());
    assert_eq!(z::zero_e(), z::two_4() % z::one_4());
    assert_eq!(z::zero_e(), z::two_4() % z::two_4());
    assert_eq!(z::zero_e(), z::one_4() % z::two_4());
    assert_eq!(z::zero_e(), z::one_4() % z::twenty_four_4());
    assert_eq!(z::one_e(), z::one_4() % z::five_4());
    assert_eq!(z::one_e(), z::one_4() % z::five_e());
    assert_eq!(z::three_e(), z::one_4() % z::ten_4());
    assert_eq!(z::two_e(), z::one_4() % z::fifteen_4());
    assert_eq!(z::zero_e(), z::five_4() % z::one_4());
    assert_eq!(z::zero_e(), z::one_twenty_five_4() % z::five_4());

    // As denominator certainty lowers, the quotient loses precision quickly
    assert_eq!(z::zero_e().approximation(2), z::one_e() / zadic_approx!(5, 6, [0, 0, 1]));
    assert_eq!(z::zero_e().approximation(1), z::one_e() / zadic_approx!(5, 5, [0, 0, 1]));
    assert_eq!(z::empty(5), z::one_e() / zadic_approx!(5, 4, [0, 0, 1]));
    assert_eq!(z::empty(5), z::one_e() / zadic_approx!(5, 3, [0, 0, 1]));
    assert_eq!(z::empty(5), z::one_e() / zadic_approx!(5, 2, [0, 0, 1]));
    // ... and the remainder goes from exact to approximate
    assert_eq!(z::one_e(), z::one_e() % zadic_approx!(5, 6, [0, 0, 1]));
    assert_eq!(z::one_e(), z::one_e() % zadic_approx!(5, 5, [0, 0, 1]));
    assert_eq!(z::one_e(), z::one_e() % zadic_approx!(5, 4, [0, 0, 1]));
    assert_eq!(z::one_e().into_approximation(1), z::one_e() % zadic_approx!(5, 3, [0, 0, 1]));
    assert_eq!(z::empty(5), z::one_e() % zadic_approx!(5, 2, [0, 0, 1]));

}
