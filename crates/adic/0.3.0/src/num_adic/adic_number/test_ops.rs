use num::pow::Pow;
use crate::{iadic_pos, iadic_neg, qadic};

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
fn div_q_adic() {

    assert_eq!(Ok(qz::one_4()), (qu::one() / qu::one()).approx(4));
    assert_eq!(Ok(qz::two_4()), (qu::two() / qu::one()).approx(4));
    assert_eq!(Ok(qz::one_4()), (qu::two() / qu::two()).approx(4));
    assert_eq!(Ok(qz::pos_1_2_4()), (qu::one() / qu::two()).approx(4));
    assert_eq!(Ok(qz::pos_1_3_4()), (qu::one() / qu::three()).approx(4));
    assert_eq!(Ok(qz::pos_1_24_4()), (qu::one() / qu::twenty_four()).approx(4));
    assert_eq!(Ok(qz::pos_1_5_4()), (qu::one() / qu::five()).approx(4));
    assert_eq!(Ok(qz::pos_1_10_4()), (qu::one() / qu::ten()).approx(4));
    assert_eq!(Ok(qz::pos_1_15_4()), (qu::one() / qu::fifteen()).approx(4));
    assert_eq!(Ok(qz::pos_1_25_4()), (qu::one() / qu::twenty_five()).approx(4));
    assert_eq!(Ok(qz::five_4()), (qu::one() / qu::one_fifth()).approx(4));
    assert_eq!(Ok(qz::seventy_five_4()), (qu::three() / qu::one_twenty_fifth()).approx(4));

    assert_eq!(Ok(qz::one_4()), (qi::one() / qi::one()).approx(4));
    assert_eq!(Ok(qz::two_4()), (qi::two() / qi::one()).approx(4));
    assert_eq!(Ok(qz::one_4()), (qi::two() / qi::two()).approx(4));
    assert_eq!(Ok(qz::pos_1_2_4()), (qi::one() / qi::two()).approx(4));
    assert_eq!(Ok(qz::pos_1_24_4()), (qi::one() / qi::twenty_four()).approx(4));
    assert_eq!(Ok(qz::pos_1_10_4()), (qi::one() / qi::ten()).approx(4));
    assert_eq!(Ok(qz::seventy_five_4()), (qi::three() / qi::one_twenty_fifth()).approx(4));
    assert_eq!(Ok(qz::neg_three_4()), (qi::neg_six() / qi::two()).approx(4));
    assert_eq!(Ok(qz::three_4()), (qi::neg_six() / qi::neg_two()).approx(4));
    assert_eq!(Ok(qz::neg_1_4_4()), (qi::neg_six() / qi::twenty_four()).approx(4));

    assert_eq!(Ok(qz::one_4()), (qr::one() / qr::one()).approx(4));
    assert_eq!(Ok(qz::two_4()), (qr::two() / qr::one()).approx(4));
    assert_eq!(Ok(qz::pos_1_2_4()), (qr::one() / qr::two()).approx(4));
    assert_eq!(Ok(qz::pos_1_24_4()), (qr::one() / qr::twenty_four()).approx(4));
    assert_eq!(Ok(qz::pos_1_10_4()), (qr::one() / qr::ten()).approx(4));
    assert_eq!(Ok(qz::seventy_five_4()), (qr::three() / qr::pos_1_25()).approx(4));
    assert_eq!(Ok(qr::pos_1_4().into_approximation(4)), (qr::one() / qr::four()).approx(4));
    assert_eq!(Ok(qr::pos_1_5().into_approximation(4)), (qr::one() / qr::five()).approx(4));
    assert_eq!(Ok(qr::neg_1_24().into_approximation(4)), (qr::neg_one() / qr::twenty_four()).approx(4));
    assert_eq!(Ok(qr::neg_1_120().into_approximation(4)), (qr::neg_1_24() / qr::five()).approx(4));
    assert_eq!(Ok(qr::neg_1_120().into_approximation(4)), (qr::neg_1_4() / qr::thirty()).approx(4));
    assert_eq!(Ok(-qr::neg_1_6().into_approximation(4)), (qr::neg_5_24() / qr::neg_5_4()).approx(4));

}
