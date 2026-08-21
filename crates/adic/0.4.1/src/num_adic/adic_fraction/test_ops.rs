use num::traits::{Inv, Pow};
use crate::{
    iadic_pos, iadic_neg, qadic, radic, zadic_approx,
    AdicError, AdicFraction, AdicNumber, QAdic, RAdic,
};

use crate::num_adic::test_util::{qi, qr, qu, qz};


#[test]
fn add_q_adic() {

    assert_eq!(qu::two(), qu::one() + qu::one());
    assert_eq!(qu::three(), qu::two() + qu::one());
    assert_eq!(qu::five(), qu::two() + qu::three());
    assert_eq!(qu::three_fifth(), qu::one_fifth() + qu::two_fifth());
    assert_eq!(qu::one(), qu::two_fifth() + qu::three_fifth());

    assert_eq!(qi::neg_two(), qi::neg_one() + qi::neg_one());
    assert_eq!(qi::neg_five(), qi::neg_two() + qi::neg_three());
    assert_eq!(qi::neg_ten(), qi::neg_five() + qi::neg_five());
    assert_eq!(qi::zero(), qi::two() + qi::neg_two());
    assert_eq!(qi::three_fifth(), qi::one_fifth() + qi::two_fifth());
    assert_eq!(qi::one(), qi::two_fifth() + qi::three_fifth());

    assert_eq!(qr::two(), qr::one() + qr::one());
    assert_eq!(qr::neg_two(), qr::neg_one() + qr::neg_one());
    assert_eq!(qr::neg_five(), qr::neg_two() + qr::neg_three());
    assert_eq!(qr::zero(), qr::two() + qr::neg_two());
    assert_eq!(qr::pos_3_5(), qr::pos_1_5() + qr::pos_2_5());
    assert_eq!(qr::two(), qr::neg_5_6() + qr::pos_17_6());

}

#[test]
fn neg_q_adic() {

    assert_eq!(qi::neg_one(), -qi::one());
    assert_eq!(qi::zero(), -qi::zero());
    assert_eq!(qi::neg_five(), -qi::five());
    assert_eq!(qi::neg_one_fifth(), -qi::one_fifth());
    let neg_p_to_neg_third = -qadic!(iadic_pos!(5, [1]), -3);
    assert_eq!(qadic!(iadic_neg!(5, []), -3), neg_p_to_neg_third);

    assert_eq!(qr::neg_five(), -qr::five());
    assert_eq!(qr::neg_1_6(), -qr::pos_1_6());
    assert_eq!(qr::neg_5_6(), -qr::pos_5_6());
    assert_eq!(qr::neg_1_120(), -qr::pos_1_120());

}

#[test]
fn sub_q_adic() {

    assert_eq!(qi::one(), qi::two() - qi::one());
    assert_eq!(qi::zero(), qi::one() - qi::one());
    assert_eq!(qi::neg_one(), qi::one() - qi::two());
    assert_eq!(qi::neg_five(), qi::one() - qi::six());
    assert_eq!(qi::four_fifth(), qi::one() - qi::one_fifth());
    assert_eq!(qi::neg_one_fifth(), qi::three_fifth() - qi::four_fifth());

    assert_eq!(qr::one(), qr::three() - qr::two());
    assert_eq!(qr::zero(), qr::four() - qr::four());
    assert_eq!(qr::six(), qr::one() - qr::neg_five());
    assert_eq!(qr::neg_1_31(), qr::pos_30_31() - qr::one());

}

#[test]
fn mul_q_adic() {

    assert_eq!(qu::two(), 2 * qu::one());
    assert_eq!(qi::neg_two(), 2 * qi::neg_one());
    assert_eq!(qu::six(), 3 * qu::two());
    assert_eq!(qi::neg_six(), 3 * qi::neg_two());
    assert_eq!(qu::five(), 5 * qu::one());
    assert_eq!(qi::neg_five(), 5 * qi::neg_one());
    assert_eq!(qu::twenty_five(), 5 * qu::five());
    assert_eq!(qi::neg_twenty_five(), 5 * qi::neg_five());
    assert_eq!(qr::neg_two(), 8 * qr::neg_1_4());
    assert_eq!(qr::one(), 120 * qr::pos_1_120());

    assert_eq!(qu::one(), qu::one() * qu::one());
    assert_eq!(qu::two(), qu::two() * qu::one());
    assert_eq!(qu::six(), qu::two() * qu::three());
    assert_eq!(qu::one_twenty_fifth(), qu::one_fifth() * qu::one_fifth());

    assert_eq!(qi::one(), qi::neg_one() * qi::neg_one());
    assert_eq!(qi::six(), qi::neg_two() * qi::neg_three());
    assert_eq!(qi::neg_twenty_five(), qi::five() * qi::neg_five());
    assert_eq!(qi::zero(), qi::two() * qi::zero());
    assert_eq!(qi::zero(), qi::neg_two() * qi::zero());
    assert_eq!(qi::one_twenty_fifth(), qi::one_fifth() * qi::one_fifth());

    assert_eq!(qr::neg_one(), qr::neg_1_24() * qr::four() * qr::six());
    assert_eq!(qr::neg_1_120(), qr::neg_1_24() * qr::pos_1_5());
    assert_eq!(qr::pos_25_16(), qr::twenty_five() * qr::pos_1_16());
    assert_eq!(qr::pos_30_31(), qr::neg_six() * qr::neg_5_31());

}

#[test]
fn pow_q_adic() {

    assert_eq!(qu::zero(), qu::zero().pow(2));
    assert_eq!(qu::zero(), qu::zero().pow(3));
    assert_eq!(qu::one(), qu::one().pow(2));
    assert_eq!(qu::one(), qu::one().pow(3));
    assert_eq!(qu::four(), qu::two().pow(2));
    assert_eq!(qu::eight(), qu::two().pow(3));
    assert_eq!(qu::twenty_five(), qu::five().pow(2));
    assert_eq!(qu::one_twenty_fifth(), qu::one_fifth().pow(2));

    assert_eq!(qi::one(), qi::neg_two().pow(0));
    assert_eq!(qi::neg_one(), qi::neg_one().pow(1));
    assert_eq!(qi::one(), qi::neg_one().pow(2));
    assert_eq!(qi::four(), qi::neg_two().pow(2));
    assert_eq!(qi::one_twenty_fifth(), qi::one_fifth().pow(2));

    assert_eq!(qr::pos_1_16(), qr::neg_1_4().pow(2));
    assert_eq!(qr::neg_1_64(), qr::neg_1_4().pow(3));
    assert_eq!(qr::pos_1_25(), qr::neg_1_5().pow(2));
    assert_eq!(qr::pos_1_25(), qr::pos_1_5().pow(2));

}

#[test]
fn inv_q_adic() {

    // Unit inverses are still units
    assert_eq!(Ok(qr::one()), qr::one().inv().qexact());
    assert_eq!(Ok(qr::pos_1_2()), qr::two().inv().qexact());
    assert_eq!(Ok(qr::pos_1_16()), qr::sixteen().inv().qexact());

    // Negative is preserved
    assert_eq!(Ok(qr::neg_one()), qr::neg_one().inv().qexact());
    assert_eq!(Ok(-qr::four()), qr::neg_1_4().inv().qexact());
    assert_eq!(Ok(-qr::six()), qr::neg_1_6().inv().qexact());

    // Non-unit inverses (positive valuation) preserve the inverse as fractional digits
    assert_eq!(Ok(qr::pos_1_6()), qr::six().inv().qexact());
    assert_eq!(Ok(qadic!(radic!(5, [1], [4, 0]), -1)), qr::thirty().inv().qexact());

    // Inversion of zero gives an error
    assert_eq!(Err(AdicError::DivideByZero), qu::zero().inv().qexact());
    assert_eq!(Err(AdicError::DivideByZero), qu::zero().inv().qapprox(-2));

    // Higher valuation lowers certainty
    assert_eq!(
        Ok(qadic!(zadic_approx!(5, 3, [1, 0, 0]), -1)),
        qadic!(zadic_approx!(5, 4, [0, 1, 0, 0]), 0).inv().qapprox(2)
    );
    assert_eq!(
        Ok(qadic!(zadic_approx!(5, 3, [3, 2, 2]), -1)),
        qadic!(zadic_approx!(5, 4, [0, 2, 0, 0]), 0).inv().qapprox(2)
    );
    assert_eq!(
        Ok(qadic!(zadic_approx!(5, 3, [2, 0, 3]), -1)),
        qadic!(zadic_approx!(5, 4, [0, 3, 2, 0]), 0).inv().qapprox(2)
    );
    assert_eq!(
        Ok(qadic!(zadic_approx!(5, 3, [3, 2, 0]), -1)),
        qadic!(zadic_approx!(5, 4, [0, 2, 0, 3]), 0).inv().qapprox(2)
    );

    // Valuation over half the certainty means non-empty inverse
    assert_eq!(
        Ok(qadic!(zadic_approx!(5, 2, [1, 0]), -2)),
        qadic!(zadic_approx!(5, 4, [0, 0, 1, 0]), 0).inv().qapprox(0)
    );
    assert_eq!(
        Ok(qadic!(zadic_approx!(5, 2, [3, 2]), -2)),
        qadic!(zadic_approx!(5, 4, [0, 0, 2, 0]), 0).inv().qapprox(0)
    );
    assert_eq!(
        Ok(qadic!(zadic_approx!(5, 1, [1]), -3)),
        qadic!(zadic_approx!(5, 4, [0, 0, 0, 1]), 0).inv().qapprox(-2)
    );

    // Inversion of empty is empty with opposite valuation
    assert_eq!(Ok(QAdic::empty(5, 0)), QAdic::empty(5, 0).inv().qapprox(0));
    assert_eq!(Ok(QAdic::empty(5, -4)), qadic!(zadic_approx!(5, 4, [0, 0, 0, 0]), 0).inv().qapprox(-4));

    // Non-unit inverses (positive valuation) do not truncate the inverse
    assert_eq!(
        Ok(qadic!(zadic_approx!(5, 4, [4, 3, 2, 1]), 0)),
        qadic!(zadic_approx!(5, 4, [4, 0, 1, 0]), 0).inv().qapprox(4)
    );
    assert_eq!(
        Ok(qadic!(zadic_approx!(5, 3, [4, 3, 2]), -1)),
        zadic_approx!(5, 4, [0, 4, 0, 1]).inv().qapprox(2)
    );

}

#[test]
#[ignore = "slow"]
fn inv_q_adic_perf() {

    let million = qadic!(RAdic::from_u32(5, 1000000), 0);
    let million_one = qadic!(RAdic::from_u32(5, 1000001), 0);
    let million_inv = million.clone().inv().qexact().unwrap();
    let million_one_inv = million_one.clone().inv().qexact().unwrap();
    // println!("{million}");
    // println!("{million_one}");
    // println!("{million_inv}");
    // println!("{million_one_inv}");
    assert_eq!(million, million_inv.inv().qexact().unwrap());
    assert_eq!(million_one, million_one_inv.inv().qexact().unwrap());
    // assert!(false);

}

#[test]
fn div_q_adic_exact() {

    assert_eq!(Ok(qr::one()), (qu::one() / qu::one()).qexact());
    assert_eq!(Ok(qr::two()), (qu::two() / qu::one()).qexact());
    assert_eq!(Ok(qr::one()), (qu::two() / qu::two()).qexact());
    assert_eq!(Ok(qr::pos_1_2()), (qu::one() / qu::two()).qexact());
    assert_eq!(Ok(qr::pos_1_24()), (qu::one() / qu::twenty_four()).qexact());
    assert_eq!(Ok(qr::pos_1_5()), (qu::one() / qu::five()).qexact());
    assert_eq!(Ok(qr::pos_1_5()), (qu::five() / qu::twenty_five()).qexact());
    assert_eq!(Err(AdicError::DivideByZero), (qu::one() / qu::zero()).qexact());

    assert_eq!(Ok(qr::two()), (qi::two() / qi::one()).qexact());
    assert_eq!(Ok(qr::neg_two()), (qi::neg_two() / qi::one()).qexact());
    assert_eq!(Ok(qr::neg_two()), (qi::two() / qi::neg_one()).qexact());
    assert_eq!(Ok(qr::two()), (qi::neg_two() / qi::neg_one()).qexact());
    assert_eq!(Ok(qr::neg_1_24()), (qi::neg_one() / qi::twenty_four()).qexact());
    assert_eq!(Ok(qr::pos_1_5()), (qi::one() / qi::five()).qexact());
    assert_eq!(Err(AdicError::DivideByZero), (qi::one() / qi::zero()).qexact());

    assert_eq!(Ok(qr::two()), (qr::two() / qr::one()).qexact());
    assert_eq!(Ok(qr::eight()), (qr::two() / qr::pos_1_4()).qexact());
    assert_eq!(Ok(qr::pos_1_8()), (qr::pos_1_4() / qr::two()).qexact());
    assert_eq!(Ok(qr::pos_1_5()), (qr::one() / qr::five()).qexact());
    assert_eq!(Err(AdicError::DivideByZero), (qr::one() / qr::zero()).qexact());

}

#[test]
fn div_q_adic_approx() {

    assert_eq!(Ok(qz::one_4()), (qu::one() / qu::one()).qapprox(4));
    assert_eq!(Ok(qz::two_4()), (qu::two() / qu::one()).qapprox(4));
    assert_eq!(Ok(qz::one_4()), (qu::two() / qu::two()).qapprox(4));
    assert_eq!(Ok(qz::pos_1_2_4()), (qu::one() / qu::two()).qapprox(4));
    assert_eq!(Ok(qz::pos_1_3_4()), (qu::one() / qu::three()).qapprox(4));
    assert_eq!(Ok(qz::pos_1_24_4()), (qu::one() / qu::twenty_four()).qapprox(4));
    assert_eq!(Ok(qz::pos_1_5_4()), (qu::one() / qu::five()).qapprox(3));
    assert_eq!(Ok(qz::pos_1_10_4()), (qu::one() / qu::ten()).qapprox(3));
    assert_eq!(Ok(qz::pos_1_15_4()), (qu::one() / qu::fifteen()).qapprox(3));
    assert_eq!(Ok(qz::pos_1_25_4()), (qu::one() / qu::twenty_five()).qapprox(2));
    assert_eq!(Ok(qz::five_4()), (qu::one() / qu::one_fifth()).qapprox(5));
    assert_eq!(Ok(qz::seventy_five_4()), (qu::three() / qu::one_twenty_fifth()).qapprox(6));

    assert_eq!(Ok(qz::one_4()), (qi::one() / qi::one()).qapprox(4));
    assert_eq!(Ok(qz::two_4()), (qi::two() / qi::one()).qapprox(4));
    assert_eq!(Ok(qz::one_4()), (qi::two() / qi::two()).qapprox(4));
    assert_eq!(Ok(qz::pos_1_2_4()), (qi::one() / qi::two()).qapprox(4));
    assert_eq!(Ok(qz::pos_1_24_4()), (qi::one() / qi::twenty_four()).qapprox(4));
    assert_eq!(Ok(qz::pos_1_10_4()), (qi::one() / qi::ten()).qapprox(3));
    assert_eq!(Ok(qz::seventy_five_4()), (qi::three() / qi::one_twenty_fifth()).qapprox(6));
    assert_eq!(Ok(qz::neg_three_4()), (qi::neg_six() / qi::two()).qapprox(4));
    assert_eq!(Ok(qz::three_4()), (qi::neg_six() / qi::neg_two()).qapprox(4));
    assert_eq!(Ok(qz::neg_1_4_4()), (qi::neg_six() / qi::twenty_four()).qapprox(4));

    assert_eq!(Ok(qz::one_4()), (qr::one() / qr::one()).qapprox(4));
    assert_eq!(Ok(qz::two_4()), (qr::two() / qr::one()).qapprox(4));
    assert_eq!(Ok(qz::pos_1_2_4()), (qr::one() / qr::two()).qapprox(4));
    assert_eq!(Ok(qz::pos_1_24_4()), (qr::one() / qr::twenty_four()).qapprox(4));
    assert_eq!(Ok(qz::pos_1_10_4()), (qr::one() / qr::ten()).qapprox(3));
    assert_eq!(Ok(qz::seventy_five_4()), (qr::three() / qr::pos_1_25()).qapprox(6));
    assert_eq!(Ok(qr::pos_1_4().into_approximation(4)), (qr::one() / qr::four()).qapprox(4));
    assert_eq!(Ok(qr::pos_1_5().into_approximation(4)), (qr::one() / qr::five()).qapprox(4));
    assert_eq!(Ok(qr::neg_1_24().into_approximation(4)), (qr::neg_one() / qr::twenty_four()).qapprox(4));
    assert_eq!(Ok(qr::neg_1_120().into_approximation(4)), (qr::neg_1_24() / qr::five()).qapprox(4));
    assert_eq!(Ok(qr::neg_1_120().into_approximation(4)), (qr::neg_1_4() / qr::thirty()).qapprox(4));
    assert_eq!(Ok(-qr::neg_1_6().into_approximation(4)), (qr::neg_5_24() / qr::neg_5_4()).qapprox(4));

    let four = qz::four_e();
    let twenty = qz::twenty_e();
    let neg_five = qz::neg_five_e();
    let neg_one = qz::neg_one_e();

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
    assert_eq!(Err(AdicError::DivideByZero), (&neg_one / QAdic::zero(5)).zapprox_max());

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
    assert_eq!(Err(AdicError::DivideByZero), (&neg_one / QAdic::zero(5)).qapprox_max());

}
