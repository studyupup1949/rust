use num::traits::{Inv, Pow};
use crate::{
    uadic, qadic, zadic_approx,
    AdicApproximate, AdicError, AdicInteger, AdicNumber, AdicValuation,
};
use super::ZAdic;

use crate::num_adic::test_util::{i, r, u, z};


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
    assert_eq!(z::twenty_five_e().certainty(), AdicValuation::PosInf);
    assert_eq!(z::twenty_five_4().certainty(), AdicValuation::Finite(4));
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

    // Many adic integers have the same inverse
    let z_1_3 = zadic_approx!(5, 4, [2, 3, 1, 3]);
    assert_eq!(Ok(z_1_3.clone()), u::three().inv().zapprox(4));
    assert_eq!(Ok(z_1_3.clone()), i::three().inv().zapprox(4));
    assert_eq!(Ok(z_1_3.clone()), r::three().inv().zapprox(4));
    assert_eq!(Ok(z_1_3.clone()), z::three_e().inv().zapprox(4));
    assert_eq!(Ok(z_1_3.clone()), z::three_4().inv().zapprox(4));

    // Valuation-zero inverses retain certainty
    assert_eq!(Ok(zadic_approx!(5, 4, [1, 0, 0, 0])), zadic_approx!(5, 4, [1, 0, 0, 0]).inv().zapprox(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [3, 2, 2, 2])), zadic_approx!(5, 4, [2, 0, 0, 0]).inv().zapprox(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [2, 0, 3, 4])), zadic_approx!(5, 4, [3, 2, 0, 0]).inv().zapprox(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [3, 2, 0, 0])), zadic_approx!(5, 4, [2, 0, 3, 4]).inv().zapprox(4));

    // Higher valuation lowers certainty
    assert_eq!(Ok(zadic_approx!(5, 2, [0, 0])), zadic_approx!(5, 4, [0, 1, 0, 0]).inv().zapprox(2));
    assert_eq!(Ok(zadic_approx!(5, 2, [2, 2])), zadic_approx!(5, 4, [0, 2, 0, 0]).inv().zapprox(2));
    assert_eq!(Ok(zadic_approx!(5, 2, [0, 3])), zadic_approx!(5, 4, [0, 3, 2, 0]).inv().zapprox(2));
    assert_eq!(Ok(zadic_approx!(5, 2, [2, 0])), zadic_approx!(5, 4, [0, 2, 0, 3]).inv().zapprox(2));

    // Valuation over half the certainty means an empty inverse
    assert_eq!(Ok(ZAdic::empty(5)), zadic_approx!(5, 4, [0, 0, 1, 0]).inv().zapprox(0));
    assert_eq!(Ok(ZAdic::empty(5)), zadic_approx!(5, 4, [0, 0, 2, 0]).inv().zapprox(0));
    assert_eq!(Ok(ZAdic::empty(5)), zadic_approx!(5, 4, [0, 0, 3, 2]).inv().zapprox(0));
    assert!(matches!(
        zadic_approx!(5, 4, [0, 0, 0, 1]).inv().zapprox(0),
        Err(AdicError::InappropriatePrecision(_))
    ));
    assert!(matches!(
        zadic_approx!(5, 4, [0, 0, 0, 0]).inv().zapprox(0),
        Err(AdicError::InappropriatePrecision(_))
    ));

    // Non-unit inverses (positive valuation) truncate the inverse
    assert_eq!(Ok(zadic_approx!(5, 4, [4, 3, 2, 1])), zadic_approx!(5, 4, [4, 0, 1, 0]).inv().zapprox(4));
    assert_eq!(Ok(zadic_approx!(5, 2, [3, 2])), zadic_approx!(5, 4, [0, 4, 0, 1]).inv().zapprox(2));

}

#[test]
fn div_z_adic() {

    let prec_err = |d: u32| {
        let err_msg = format!("a and b not precise enough to give {d} digits");
        Err(AdicError::InappropriatePrecision(err_msg))
    };

    assert_eq!(Ok(zadic_approx!(5, 4, [1, 0, 0, 0])), (z::one_4() / z::one_4()).zapprox(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [2, 0, 0, 0])), (z::two_4() / z::one_4()).zapprox(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [1, 0, 0, 0])), (z::two_4() / z::two_4()).zapprox(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [3, 2, 2, 2])), (z::one_4() / z::two_4()).zapprox(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [2, 2, 2, 2])), (z::neg_one_4() / z::two_4()).zapprox(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [2, 2, 2, 2])), (z::one_4() / z::neg_two_4()).zapprox(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [3, 2, 2, 2])), (z::neg_one_4() / z::neg_two_4()).zapprox(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [2, 3, 1, 3])), (z::one_4() / z::three_4()).zapprox(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [4, 4, 3, 4])), (z::one_4() / z::twenty_four_4()).zapprox(4));
    assert_eq!(prec_err(4), (z::one_4() / z::five_4()).zapprox(4));
    assert_eq!(prec_err(3), (z::one_4() / z::five_4()).zapprox(3));
    assert_eq!(Ok(zadic_approx!(5, 2, [0, 0])), (z::one_4() / z::five_4()).zapprox(2));
    assert_eq!(Ok(zadic_approx!(5, 3, [0, 0, 0])), (z::one_4() / z::five_e()).zapprox(3));
    assert_eq!(prec_err(4), (z::one_4() / z::five_4()).zapprox(4));
    assert_eq!(prec_err(4), (z::one_e() / z::five_4()).zapprox(4));
    assert_eq!(prec_err(4), (z::one_4() / z::five_e()).zapprox(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [0, 0, 0, 0])), (z::one_e() / z::five_e()).zapprox(4));
    assert_eq!(prec_err(4), (z::one_4() / z::ten_4()).zapprox(4));
    assert_eq!(prec_err(4), (z::one_4() / z::fifteen_4()).zapprox(4));
    assert_eq!(Ok(z::five_4()), (z::five_4() / z::one_4()).zapprox(4));
    assert_eq!(Ok(zadic_approx!(5, 4, [0, 3, 2, 2])), (z::five_4() / z::two_4()).zapprox(4));
    assert_eq!(Ok(zadic_approx!(5, 3, [0, 0, 0])), (z::one_twenty_five_4() / z::one_4()).zapprox(3));
    assert_eq!(Ok(zadic_approx!(5, 2, [0, 0])), (z::one_twenty_five_4() / z::one_4()).zapprox(2));
    assert_eq!(Ok(zadic_approx!(5, 1, [0])), (z::one_twenty_five_4() / z::one_4()).zapprox(1));
    assert_eq!(Ok(zadic_approx!(5, 2, [0, 0])), (z::one_twenty_five_4() / z::five_4()).zapprox(2));
    assert_eq!(Ok(zadic_approx!(5, 1, [0])), (z::one_twenty_five_4() / z::five_4()).zapprox(1));
    assert_eq!(Ok(zadic_approx!(5, 1, [0])), (z::one_twenty_five_4() / z::twenty_five_4()).zapprox(1));
    assert_eq!(Ok(zadic_approx!(5, 3, [0, 0, 1])), (z::one_twenty_five_4() / z::five_4()).zapprox(3));
    assert_eq!(Ok(zadic_approx!(5, 3, [0, 1, 1])), (z::neg_five_e().approximation(3) / z::four_4()).zapprox(3));

    let four = z::four_e();
    let twenty = z::twenty_e();
    let neg_five = z::neg_five_e();
    let neg_one = z::neg_one_e();

    assert_eq!(Ok(zadic_approx!(5, 2, [1, 1])), (&neg_one / four.approximation(2)).zapprox_max());
    assert_eq!(Ok(zadic_approx!(5, 2, [1, 1])), (neg_one.approximation(3) / four.approximation(2)).zapprox_max());
    assert_eq!(Ok(zadic_approx!(5, 4, [1, 1, 1, 1])), (&neg_one / four.approximation(4)).zapprox_max());
    assert_eq!(Ok(zadic_approx!(5, 3, [1, 1, 1])), (neg_one.approximation(3) / four.approximation(4)).zapprox_max());
    assert_eq!(Ok(zadic_approx!(5, 3, [1, 1, 1])), (neg_one.approximation(4) / four.approximation(3)).zapprox_max());
    assert_eq!(Ok(zadic_approx!(5, 2, [1, 1])), (neg_one.approximation(3) / twenty.approximation(4)).zapprox_max());
    assert_eq!(Ok(zadic_approx!(5, 1, [1])), (neg_one.approximation(4) / twenty.approximation(3)).zapprox_max());
    assert_eq!(Ok(zadic_approx!(5, 2, [1, 1])), (neg_five.approximation(3) / twenty.approximation(4)).zapprox_max());
    assert_eq!(Ok(zadic_approx!(5, 2, [1, 1])), (neg_five.approximation(4) / twenty.approximation(3)).zapprox_max());
    assert_eq!(Ok(zadic_approx!(5, 3, [0, 1, 1])), (neg_five.approximation(3) / four.approximation(4)).zapprox_max());
    assert_eq!(Ok(zadic_approx!(5, 4, [0, 1, 1, 1])), (neg_five.approximation(4) / four.approximation(3)).zapprox_max());
    assert!(matches!((&neg_one / &four).zapprox_max(), Err(AdicError::InappropriatePrecision(_))));
    assert_eq!(Err(AdicError::DivideByZero), (&neg_one / ZAdic::zero(5)).zapprox_max());

    assert_eq!(Ok(qadic!(zadic_approx!(5, 2, [1, 1]), 0)), (&neg_one / four.approximation(2)).qapprox_max());
    assert_eq!(Ok(qadic!(zadic_approx!(5, 2, [1, 1]), 0)), (neg_one.approximation(3) / four.approximation(2)).qapprox_max());
    assert_eq!(Ok(qadic!(zadic_approx!(5, 4, [1, 1, 1, 1]), 0)), (&neg_one / four.approximation(4)).qapprox_max());
    assert_eq!(Ok(qadic!(zadic_approx!(5, 3, [1, 1, 1]), 0)), (neg_one.approximation(3) / four.approximation(4)).qapprox_max());
    assert_eq!(Ok(qadic!(zadic_approx!(5, 3, [1, 1, 1]), 0)), (neg_one.approximation(4) / four.approximation(3)).qapprox_max());
    assert_eq!(Ok(qadic!(zadic_approx!(5, 3, [1, 1, 1]), -1)), (neg_one.approximation(3) / twenty.approximation(4)).qapprox_max());
    assert_eq!(Ok(qadic!(zadic_approx!(5, 2, [1, 1]), -1)), (neg_one.approximation(4) / twenty.approximation(3)).qapprox_max());
    assert_eq!(Ok(qadic!(zadic_approx!(5, 2, [1, 1]), 0)), (neg_five.approximation(3) / twenty.approximation(4)).qapprox_max());
    assert_eq!(Ok(qadic!(zadic_approx!(5, 2, [1, 1]), 0)), (neg_five.approximation(4) / twenty.approximation(3)).qapprox_max());
    assert_eq!(Ok(qadic!(zadic_approx!(5, 2, [1, 1]), 1)), (neg_five.approximation(3) / four.approximation(4)).qapprox_max());
    assert_eq!(Ok(qadic!(zadic_approx!(5, 4, [0, 1, 1, 1]), 0)), (neg_five.approximation(4) / four.approximation(3)).qapprox_max());
    assert!(matches!((&neg_one / &four).qapprox_max(), Err(AdicError::InappropriatePrecision(_))));
    assert_eq!(Err(AdicError::DivideByZero), (&neg_one / ZAdic::zero(5)).qapprox_max());

}
