use num::{
    traits::{Inv, Pow},
    CheckedDiv,
};
use crate::{
    traits::{CanApproximate, PrimedFrom},
    EAdic, QAdic, ZAdic,
};

use crate::num_adic::test_util::{qe, qu, qz};


#[test]
fn add_q_adic() {

    assert_eq!(qu::two(), qu::one() + qu::one());
    assert_eq!(qu::three(), qu::two() + qu::one());
    assert_eq!(qu::five(), qu::two() + qu::three());
    assert_eq!(qu::three_fifth(), qu::one_fifth() + qu::two_fifth());
    assert_eq!(qu::one(), qu::two_fifth() + qu::three_fifth());

    assert_eq!(qe::neg_two(), qe::neg_one() + qe::neg_one());
    assert_eq!(qe::neg_five(), qe::neg_two() + qe::neg_three());
    assert_eq!(qe::neg_ten(), qe::neg_five() + qe::neg_five());
    assert_eq!(qe::zero(), qe::two() + qe::neg_two());
    assert_eq!(qe::three_fifth(), qe::one_fifth() + qe::two_fifth());
    assert_eq!(qe::one(), qe::two_fifth() + qe::three_fifth());

    assert_eq!(qe::two(), qe::one() + qe::one());
    assert_eq!(qe::neg_two(), qe::neg_one() + qe::neg_one());
    assert_eq!(qe::neg_five(), qe::neg_two() + qe::neg_three());
    assert_eq!(qe::zero(), qe::two() + qe::neg_two());
    assert_eq!(qe::pos_3_5(), qe::pos_1_5() + qe::pos_2_5());
    assert_eq!(qe::two(), qe::neg_5_6() + qe::pos_17_6());

}

#[test]
fn neg_q_adic() {

    assert_eq!(qe::neg_one(), -qe::one());
    assert_eq!(qe::zero(), -qe::zero());
    assert_eq!(qe::neg_five(), -qe::five());
    assert_eq!(qe::neg_one_fifth(), -qe::one_fifth());
    let neg_p_to_neg_third = -qadic!(eadic!(5, [1]), -3);
    assert_eq!(qadic!(eadic_neg!(5, []), -3), neg_p_to_neg_third);

    assert_eq!(qe::neg_five(), -qe::five());
    assert_eq!(qe::neg_1_6(), -qe::pos_1_6());
    assert_eq!(qe::neg_5_6(), -qe::pos_5_6());
    assert_eq!(qe::neg_1_120(), -qe::pos_1_120());

}

#[test]
fn sub_q_adic() {

    assert_eq!(qe::one(), qe::two() - qe::one());
    assert_eq!(qe::zero(), qe::one() - qe::one());
    assert_eq!(qe::neg_one(), qe::one() - qe::two());
    assert_eq!(qe::neg_five(), qe::one() - qe::six());
    assert_eq!(qe::four_fifth(), qe::one() - qe::one_fifth());
    assert_eq!(qe::neg_one_fifth(), qe::three_fifth() - qe::four_fifth());

    assert_eq!(qe::one(), qe::three() - qe::two());
    assert_eq!(qe::zero(), qe::four() - qe::four());
    assert_eq!(qe::six(), qe::one() - qe::neg_five());
    assert_eq!(qe::neg_1_31(), qe::pos_30_31() - qe::one());

}

#[test]
fn mul_q_adic() {

    assert_eq!(qu::two(), 2 * qu::one());
    assert_eq!(qe::neg_two(), 2 * qe::neg_one());
    assert_eq!(qu::six(), 3 * qu::two());
    assert_eq!(qe::neg_six(), 3 * qe::neg_two());
    assert_eq!(qu::five(), 5 * qu::one());
    assert_eq!(qe::neg_five(), 5 * qe::neg_one());
    assert_eq!(qu::twenty_five(), 5 * qu::five());
    assert_eq!(qe::neg_twenty_five(), 5 * qe::neg_five());
    assert_eq!(qe::neg_two(), 8 * qe::neg_1_4());
    assert_eq!(qe::one(), 120 * qe::pos_1_120());

    assert_eq!(qu::one(), qu::one() * qu::one());
    assert_eq!(qu::two(), qu::two() * qu::one());
    assert_eq!(qu::six(), qu::two() * qu::three());
    assert_eq!(qu::one_twenty_fifth(), qu::one_fifth() * qu::one_fifth());

    assert_eq!(qe::one(), qe::neg_one() * qe::neg_one());
    assert_eq!(qe::six(), qe::neg_two() * qe::neg_three());
    assert_eq!(qe::neg_twenty_five(), qe::five() * qe::neg_five());
    assert_eq!(qe::zero(), qe::two() * qe::zero());
    assert_eq!(qe::zero(), qe::neg_two() * qe::zero());
    assert_eq!(qe::one_twenty_fifth(), qe::one_fifth() * qe::one_fifth());

    assert_eq!(qe::neg_one(), qe::neg_1_24() * qe::four() * qe::six());
    assert_eq!(qe::neg_1_120(), qe::neg_1_24() * qe::pos_1_5());
    assert_eq!(qe::pos_25_16(), qe::twenty_five() * qe::pos_1_16());
    assert_eq!(qe::pos_30_31(), qe::neg_six() * qe::neg_5_31());

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

    assert_eq!(qe::one(), qe::neg_two().pow(0));
    assert_eq!(qe::neg_one(), qe::neg_one().pow(1));
    assert_eq!(qe::one(), qe::neg_one().pow(2));
    assert_eq!(qe::four(), qe::neg_two().pow(2));
    assert_eq!(qe::one_twenty_fifth(), qe::one_fifth().pow(2));

    assert_eq!(qe::pos_1_16(), qe::neg_1_4().pow(2));
    assert_eq!(qe::neg_1_64(), qe::neg_1_4().pow(3));
    assert_eq!(qe::pos_1_25(), qe::neg_1_5().pow(2));
    assert_eq!(qe::pos_1_25(), qe::pos_1_5().pow(2));

}

#[test]
fn inv_q_adic() {

    // Unit inverses are still units
    assert_eq!(qe::one(), qe::one().inv());
    assert_eq!(qe::pos_1_2(), qe::two().inv());
    assert_eq!(qe::pos_1_16(), qe::sixteen().inv());

    // Negative is preserved
    assert_eq!(qe::neg_one(), qe::neg_one().inv());
    assert_eq!(-qe::four(), qe::neg_1_4().inv());
    assert_eq!(-qe::six(), qe::neg_1_6().inv());

    // Non-unit inverses (positive valuation) preserve the inverse as fractional digits
    assert_eq!(qe::pos_1_6(), qe::six().inv());
    assert_eq!(qadic!(eadic_rep!(5, [1], [4, 0]), -1), qe::thirty().inv());

    // Inversion of zero gives an error
    // Formerly returned an error, now panics
    // assert_eq!(Err(AdicError::DivideByZero), qu::zero().inv());

    // Higher valuation lowers certainty
    assert_eq!(
        qadic!(zadic_approx!(5, 3, [1, 0, 0]), -1),
        qadic!(zadic_approx!(5, 4, [0, 1, 0, 0]), 0).inv().approximation(2)
    );
    assert_eq!(
        qadic!(zadic_approx!(5, 3, [3, 2, 2]), -1),
        qadic!(zadic_approx!(5, 4, [0, 2, 0, 0]), 0).inv().approximation(2)
    );
    assert_eq!(
        qadic!(zadic_approx!(5, 3, [2, 0, 3]), -1),
        qadic!(zadic_approx!(5, 4, [0, 3, 2, 0]), 0).inv().approximation(2)
    );
    assert_eq!(
        qadic!(zadic_approx!(5, 3, [3, 2, 0]), -1),
        qadic!(zadic_approx!(5, 4, [0, 2, 0, 3]), 0).inv().approximation(2)
    );

    // Valuation over half the certainty means non-empty inverse
    assert_eq!(
        qadic!(zadic_approx!(5, 2, [1, 0]), -2),
        qadic!(zadic_approx!(5, 4, [0, 0, 1, 0]), 0).inv().approximation(0)
    );
    assert_eq!(
        qadic!(zadic_approx!(5, 2, [3, 2]), -2),
        qadic!(zadic_approx!(5, 4, [0, 0, 2, 0]), 0).inv().approximation(0)
    );
    assert_eq!(
        qadic!(zadic_approx!(5, 1, [1]), -3),
        qadic!(zadic_approx!(5, 4, [0, 0, 0, 1]), 0).inv().approximation(-2)
    );

    // Inversion of empty is empty with opposite valuation
    assert_eq!(QAdic::empty(5, 0), QAdic::empty(5, 0).inv().approximation(0));
    assert_eq!(QAdic::empty(5, -4), qadic!(zadic_approx!(5, 4, [0, 0, 0, 0]), 0).inv().approximation(-4));

    // Non-unit inverses (positive valuation) do not truncate the inverse
    assert_eq!(
        qadic!(zadic_approx!(5, 4, [4, 3, 2, 1]), 0),
        qadic!(zadic_approx!(5, 4, [4, 0, 1, 0]), 0).inv().approximation(4)
    );
    assert_eq!(
        qadic!(zadic_approx!(5, 3, [4, 3, 2]), -1),
        qadic!(zadic_approx!(5, 4, [0, 4, 0, 1]), 0).inv().approximation(2)
    );

}

#[test]
#[ignore = "slow"]
fn inv_q_adic_perf() {

    let million = qadic!(EAdic::primed_from(5, 1000000), 0);
    let million_one = qadic!(EAdic::primed_from(5, 1000001), 0);
    let million_inv = million.clone().inv();
    let million_one_inv = million_one.clone().inv();
    // println!("{million}");
    // println!("{million_one}");
    // println!("{million_inv}");
    // println!("{million_one_inv}");
    assert_eq!(million, million_inv.inv());
    assert_eq!(million_one, million_one_inv.inv());
    // assert!(false);

}

#[test]
fn div_q_adic_exact() {

    assert_eq!(qe::one(), qe::one() / qe::one());
    assert_eq!(qe::two(), qe::two() / qe::one());
    assert_eq!(qe::one(), qe::two() / qe::two());
    assert_eq!(qe::pos_1_2(), qe::one() / qe::two());
    assert_eq!(qe::pos_1_24(), qe::one() / qe::twenty_four());
    assert_eq!(qe::pos_1_5(), qe::one() / qe::five());
    assert_eq!(qe::pos_1_5(), qe::five() / qe::twenty_five());

    assert_eq!(qe::two(), qe::two() / qe::one());
    assert_eq!(qe::neg_two(), qe::neg_two() / qe::one());
    assert_eq!(qe::neg_two(), qe::two() / qe::neg_one());
    assert_eq!(qe::two(), qe::neg_two() / qe::neg_one());
    assert_eq!(qe::neg_1_24(), qe::neg_one() / qe::twenty_four());
    assert_eq!(qe::pos_1_5(), qe::one() / qe::five());

    assert_eq!(qe::two(), qe::two() / qe::one());
    assert_eq!(qe::eight(), qe::two() / qe::pos_1_4());
    assert_eq!(qe::pos_1_8(), qe::pos_1_4() / qe::two());
    assert_eq!(qe::pos_1_5(), qe::one() / qe::five());

    assert_eq!(None, qe::one().checked_div(&qe::zero()));

}

#[test]
fn div_q_adic_approx() {

    assert_eq!(qz::one_4(), (qe::one() / qe::one()).approximation(4));
    assert_eq!(qz::two_4(), (qe::two() / qe::one()).approximation(4));
    assert_eq!(qz::one_4(), (qe::two() / qe::two()).approximation(4));
    assert_eq!(qz::pos_1_2_4(), (qe::one() / qe::two()).approximation(4));
    assert_eq!(qz::pos_1_3_4(), (qe::one() / qe::three()).approximation(4));
    assert_eq!(qz::pos_1_24_4(), (qe::one() / qe::twenty_four()).approximation(4));
    assert_eq!(qz::pos_1_5_4(), (qe::one() / qe::five()).approximation(3));
    assert_eq!(qz::pos_1_10_4(), (qe::one() / qe::ten()).approximation(3));
    assert_eq!(qz::pos_1_15_4(), (qe::one() / qe::fifteen()).approximation(3));
    assert_eq!(qz::pos_1_25_4(), (qe::one() / qe::twenty_five()).approximation(2));
    assert_eq!(qz::five_4(), (qe::one() / qe::one_fifth()).approximation(5));
    assert_eq!(qz::seventy_five_4(), (qe::three() / qe::one_twenty_fifth()).approximation(6));

    assert_eq!(qz::one_4(), (qe::one() / qe::one()).approximation(4));
    assert_eq!(qz::two_4(), (qe::two() / qe::one()).approximation(4));
    assert_eq!(qz::one_4(), (qe::two() / qe::two()).approximation(4));
    assert_eq!(qz::pos_1_2_4(), (qe::one() / qe::two()).approximation(4));
    assert_eq!(qz::pos_1_24_4(), (qe::one() / qe::twenty_four()).approximation(4));
    assert_eq!(qz::pos_1_10_4(), (qe::one() / qe::ten()).approximation(3));
    assert_eq!(qz::seventy_five_4(), (qe::three() / qe::one_twenty_fifth()).approximation(6));
    assert_eq!(qz::neg_three_4(), (qe::neg_six() / qe::two()).approximation(4));
    assert_eq!(qz::three_4(), (qe::neg_six() / qe::neg_two()).approximation(4));
    assert_eq!(qz::neg_1_4_4(), (qe::neg_six() / qe::twenty_four()).approximation(4));

    assert_eq!(qz::one_4(), (qe::one() / qe::one()).approximation(4));
    assert_eq!(qz::two_4(), (qe::two() / qe::one()).approximation(4));
    assert_eq!(qz::pos_1_2_4(), (qe::one() / qe::two()).approximation(4));
    assert_eq!(qz::pos_1_24_4(), (qe::one() / qe::twenty_four()).approximation(4));
    assert_eq!(qz::pos_1_10_4(), (qe::one() / qe::ten()).approximation(3));
    assert_eq!(qz::seventy_five_4(), (qe::three() / qe::pos_1_25()).approximation(6));
    assert_eq!(qe::pos_1_4().into_approximation(4), (qe::one() / qe::four()).approximation(4));
    assert_eq!(qe::pos_1_5().into_approximation(4), (qe::one() / qe::five()).approximation(4));
    assert_eq!(qe::neg_1_24().into_approximation(4), (qe::neg_one() / qe::twenty_four()).approximation(4));
    assert_eq!(qe::neg_1_120().into_approximation(4), (qe::neg_1_24() / qe::five()).approximation(4));
    assert_eq!(qe::neg_1_120().into_approximation(4), (qe::neg_1_4() / qe::thirty()).approximation(4));
    assert_eq!(-qe::neg_1_6().into_approximation(4), (qe::neg_5_24() / qe::neg_5_4()).approximation(4));

    let four = qz::four_e();
    let twenty = qz::twenty_e();
    let neg_five = qz::neg_five_e();
    let neg_one = qz::neg_one_e();

    assert_eq!(qadic!(zadic_approx!(5, 2, [1, 1]), 0), neg_one.clone() / four.approximation(2));
    assert_eq!(qadic!(zadic_approx!(5, 2, [1, 1]), 0), neg_one.approximation(3) / four.approximation(2));
    assert_eq!(qadic!(zadic_approx!(5, 4, [1, 1, 1, 1]), 0), neg_one.clone() / four.approximation(4));
    assert_eq!(qadic!(zadic_approx!(5, 3, [1, 1, 1]), 0), neg_one.approximation(3) / four.approximation(4));
    assert_eq!(qadic!(zadic_approx!(5, 3, [1, 1, 1]), 0), neg_one.approximation(4) / four.approximation(3));
    assert_eq!(qadic!(zadic_approx!(5, 3, [1, 1, 1]), -1), neg_one.approximation(3) / twenty.approximation(4));
    assert_eq!(qadic!(zadic_approx!(5, 2, [1, 1]), -1), neg_one.approximation(4) / twenty.approximation(3));
    assert_eq!(qadic!(zadic_approx!(5, 2, [1, 1]), 0), neg_five.approximation(3) / twenty.approximation(4));
    assert_eq!(qadic!(zadic_approx!(5, 2, [1, 1]), 0), neg_five.approximation(4) / twenty.approximation(3));
    assert_eq!(qadic!(zadic_approx!(5, 2, [1, 1]), 1), neg_five.approximation(3) / four.approximation(4));
    assert_eq!(qadic!(zadic_approx!(5, 4, [0, 1, 1, 1]), 0), neg_five.approximation(4) / four.approximation(3));

    assert_eq!(QAdic::<ZAdic>::from_integer(eadic_rep!(5, [], [1]).into()), neg_one.clone() / four.clone());
    assert_eq!(None, qe::one().checked_div(&qe::zero()));

}
