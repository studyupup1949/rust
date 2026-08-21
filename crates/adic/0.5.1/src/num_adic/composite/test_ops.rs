use num::traits::Pow;

use crate::num_adic::test_util::{e, u, qe, qu};


// PowAdic

#[test]
fn add_adic_power() {

    assert_eq!(apow!(u::two(), 2), apow!(u::one(), 2) + apow!(u::one(), 2));
    assert_eq!(apow!(u::five(), 2), apow!(u::two(), 2) + apow!(u::three(), 2));
    assert_eq!(apow!(u::twenty_five(), 2), apow!(u::ten(), 2) + apow!(u::fifteen(), 2));
    assert_eq!(apow!(qu::two(), 2), apow!(qu::one(), 2) + apow!(qu::one(), 2));
    assert_eq!(apow!(qu::five(), 2), apow!(qu::two(), 2) + apow!(qu::three(), 2));
    assert_eq!(apow!(qu::three_fifth(), 2), apow!(qu::one_fifth(), 2) + apow!(qu::two_fifth(), 2));

    assert_eq!(apow!(e::neg_two(), 2), apow!(e::neg_one(), 2) + apow!(e::neg_one(), 2));
    assert_eq!(apow!(qe::neg_five(), 2), apow!(qe::neg_two(), 2) + apow!(qe::neg_three(), 2));
    assert_eq!(apow!(e::zero(), 2), apow!(e::two(), 2) + apow!(e::neg_two(), 2));

    assert_eq!(apow!(e::two(), 5), apow!(e::one(), 5) + apow!(e::one(), 5));
    assert_eq!(apow!(qe::pos_3_5(), 2), apow!(qe::pos_1_5(), 2) + apow!(qe::pos_2_5(), 2));
    assert_eq!(apow!(e::two(), 2), apow!(e::neg_5_6(), 2) + apow!(e::pos_17_6(), 2));

}

#[test]
fn neg_adic_power() {

    assert_eq!(apow!(e::neg_one(), 2), -apow!(e::one(), 2));
    assert_eq!(apow!(e::zero(), 2), -apow!(e::zero(), 2));
    assert_eq!(apow!(e::neg_five(), 2), -apow!(e::five(), 2));
    assert_eq!(apow!(e::neg_twenty_five(), 2), -apow!(e::twenty_five(), 2));
    assert_eq!(apow!(qe::neg_one_fifth(), 2), -apow!(qe::one_fifth(), 2));

    assert_eq!(apow!(e::neg_five(), 2), -apow!(e::five(), 2));
    assert_eq!(apow!(e::neg_1_6(), 2), -apow!(e::pos_1_6(), 2));
    assert_eq!(apow!(e::neg_5_6(), 2), -apow!(e::pos_5_6(), 2));
    assert_eq!(apow!(qe::neg_1_120(), 2), -apow!(qe::pos_1_120(), 2));

}

#[test]
fn sub_adic_power() {

    assert_eq!(apow!(e::one(), 2), apow!(e::two(), 2) - apow!(e::one(), 2));
    assert_eq!(apow!(e::zero(), 2), apow!(e::one(), 2) - apow!(e::one(), 2));
    assert_eq!(apow!(e::neg_one(), 2), apow!(e::one(), 2) - apow!(e::two(), 2));
    assert_eq!(apow!(e::neg_five(), 2), apow!(e::one(), 2) - apow!(e::six(), 2));
    assert_eq!(apow!(qe::four_fifth(), 2), apow!(qe::one(), 2) - apow!(qe::one_fifth(), 2));
    assert_eq!(apow!(qe::neg_one_fifth(), 2), apow!(qe::three_fifth(), 2) - apow!(qe::four_fifth(), 2));

    assert_eq!(apow!(e::one(), 2), apow!(e::three(), 2) - apow!(e::two(), 2));
    assert_eq!(apow!(e::zero(), 2), apow!(e::four(), 2) - apow!(e::four(), 2));
    assert_eq!(apow!(e::six(), 2), apow!(e::one(), 2) - apow!(e::neg_five(), 2));
    assert_eq!(apow!(qe::neg_1_31(), 2), apow!(qe::pos_30_31(), 2) - apow!(qe::one(), 2));

}

#[test]
fn mul_adic_power() {

    assert_eq!(apow!(u::two(), 2), apow!(u::two(), 2) * apow!(u::one(), 2));
    assert_eq!(apow!(u::one(), 2), apow!(u::one(), 2) * apow!(u::one(), 2));
    assert_eq!(apow!(qu::one_twenty_fifth(), 2), apow!(qu::one_fifth(), 2) * apow!(qu::one_fifth(), 2));

    assert_eq!(apow!(e::one(), 2), apow!(e::neg_one(), 2) * apow!(e::neg_one(), 2));
    assert_eq!(apow!(e::six(), 2), apow!(e::neg_two(), 2) * apow!(e::neg_three(), 2));
    assert_eq!(apow!(e::neg_twenty_five(), 2), apow!(e::five(), 2) * apow!(e::neg_five(), 2));
    assert_eq!(apow!(e::zero(), 2), apow!(e::two(), 2) * apow!(e::zero(), 2));
    assert_eq!(apow!(e::zero(), 2), apow!(e::neg_two(), 2) * apow!(e::zero(), 2));
    assert_eq!(apow!(qe::one_twenty_fifth(), 2), apow!(qe::one_fifth(), 2) * apow!(qe::one_fifth(), 2));

    assert_eq!(apow!(e::neg_one(), 2), apow!(e::neg_1_24(), 2) * apow!(e::four(), 2) * apow!(e::six(), 2));
    assert_eq!(apow!(qe::neg_1_120(), 2), apow!(qe::neg_1_24(), 2) * apow!(qe::pos_1_5(), 2));
    assert_eq!(apow!(qe::pos_25_16(), 2), apow!(qe::twenty_five(), 2) * apow!(qe::pos_1_16(), 2));

}

#[test]
fn pow_adic_power() {

    assert_eq!(apow!(u::zero(), 2), apow!(u::zero(), 2).pow(2));
    assert_eq!(apow!(u::one(), 2), apow!(u::zero(), 2).pow(0));
    assert_eq!(apow!(u::one(), 2), apow!(u::one(), 2).pow(3));
    assert_eq!(apow!(u::four(), 2), apow!(u::two(), 2).pow(2));
    assert_eq!(apow!(u::eight(), 2), apow!(u::two(), 2).pow(3));
    assert_eq!(apow!(u::twenty_five(), 2), apow!(u::five(), 2).pow(2));
    assert_eq!(apow!(qu::one_twenty_fifth(), 2), apow!(qu::one_fifth(), 2).pow(2));

    assert_eq!(apow!(e::one(), 2), apow!(e::neg_two(), 2).pow(0));
    assert_eq!(apow!(e::neg_one(), 2), apow!(e::neg_one(), 2).pow(1));
    assert_eq!(apow!(e::one(), 2), apow!(e::neg_one(), 2).pow(2));
    assert_eq!(apow!(e::four(), 2), apow!(e::neg_two(), 2).pow(2));
    assert_eq!(apow!(qe::one_twenty_fifth(), 2), apow!(qe::one_fifth(), 2).pow(2));

    assert_eq!(apow!(e::pos_1_16(), 2), apow!(e::neg_1_4(), 2).pow(2));
    assert_eq!(apow!(e::neg_1_64(), 2), apow!(e::neg_1_4(), 2).pow(3));
    assert_eq!(apow!(qe::pos_1_25(), 2), apow!(qe::neg_1_5(), 2).pow(2));
    assert_eq!(apow!(qe::pos_1_25(), 2), apow!(qe::pos_1_5(), 2).pow(2));

}


// MAdic

mod c {

    #![allow(dead_code)]
    #![allow(unreachable_pub)]
    use crate::{num_adic::MAdic, EAdic};

    type MEAdic = MAdic<EAdic>;

    pub fn zero() -> MEAdic { MAdic::from_i32(10, 0).unwrap() }
    pub fn one() -> MEAdic { MAdic::from_i32(10, 1).unwrap() }
    pub fn two() -> MEAdic { MAdic::from_i32(10, 2).unwrap() }
    pub fn three() -> MEAdic { MAdic::from_i32(10, 3).unwrap() }
    pub fn four() -> MEAdic { MAdic::from_i32(10, 4).unwrap() }
    pub fn five() -> MEAdic { MAdic::from_i32(10, 5).unwrap() }
    pub fn six() -> MEAdic { MAdic::from_i32(10, 6).unwrap() }
    pub fn seven() -> MEAdic { MAdic::from_i32(10, 7).unwrap() }
    pub fn eight() -> MEAdic { MAdic::from_i32(10, 8).unwrap() }
    pub fn nine() -> MEAdic { MAdic::from_i32(10, 9).unwrap() }
    pub fn ten() -> MEAdic { MAdic::from_i32(10, 10).unwrap() }
    pub fn twenty() -> MEAdic { MAdic::from_i32(10, 20).unwrap() }
    pub fn neg_one() -> MEAdic { MAdic::from_i32(10, -1).unwrap() }
    pub fn neg_two() -> MEAdic { MAdic::from_i32(10, -2).unwrap() }
    pub fn neg_three() -> MEAdic { MAdic::from_i32(10, -3).unwrap() }
    pub fn neg_five() -> MEAdic { MAdic::from_i32(10, -5).unwrap() }
    pub fn neg_ten() -> MEAdic { MAdic::from_i32(10, -10).unwrap() }

    pub fn pure_2_one() -> MEAdic { MAdic::from_pure_p_adic(10, eadic!(2, [1])).unwrap() }
    pub fn pure_2_two() -> MEAdic { MAdic::from_pure_p_adic(10, eadic!(2, [0, 1])).unwrap() }
    pub fn pure_2_three() -> MEAdic { MAdic::from_pure_p_adic(10, eadic!(2, [1, 1])).unwrap() }
    pub fn pure_2_four() -> MEAdic { MAdic::from_pure_p_adic(10, eadic!(2, [0, 0, 1])).unwrap() }
    pub fn pure_2_five() -> MEAdic { MAdic::from_pure_p_adic(10, eadic!(2, [1, 0, 1])).unwrap() }
    pub fn pure_2_ten() -> MEAdic { MAdic::from_pure_p_adic(10, eadic!(2, [0, 1, 0, 1])).unwrap() }
    pub fn pure_2_neg_one() -> MEAdic { MAdic::from_pure_p_adic(10, eadic_neg!(2, [])).unwrap() }
    pub fn pure_2_neg_five() -> MEAdic { MAdic::from_pure_p_adic(10, eadic_neg!(2, [1, 1, 0])).unwrap() }
    pub fn pure_2_neg_ten() -> MEAdic { MAdic::from_pure_p_adic(10, eadic_neg!(2, [0, 1, 1, 0])).unwrap() }

    pub fn pure_5_one() -> MEAdic { MAdic::from_pure_p_adic(10, eadic!(5, [1])).unwrap() }
    pub fn pure_5_two() -> MEAdic { MAdic::from_pure_p_adic(10, eadic!(5, [2])).unwrap() }
    pub fn pure_5_three() -> MEAdic { MAdic::from_pure_p_adic(10, eadic!(5, [3])).unwrap() }
    pub fn pure_5_four() -> MEAdic { MAdic::from_pure_p_adic(10, eadic!(5, [4])).unwrap() }
    pub fn pure_5_five() -> MEAdic { MAdic::from_pure_p_adic(10, eadic!(5, [0, 1])).unwrap() }
    pub fn pure_5_ten() -> MEAdic { MAdic::from_pure_p_adic(10, eadic!(5, [0, 2])).unwrap() }
    pub fn pure_5_neg_one() -> MEAdic { MAdic::from_pure_p_adic(10, eadic_neg!(5, [])).unwrap() }
    pub fn pure_5_neg_five() -> MEAdic { MAdic::from_pure_p_adic(10, eadic_neg!(5, [0, 4])).unwrap() }
    pub fn pure_5_neg_ten() -> MEAdic { MAdic::from_pure_p_adic(10, eadic_neg!(5, [0, 3])).unwrap() }

}

#[test]
fn add_adic_composite() {

    assert_eq!(c::two(), c::one() + c::one());
    assert_eq!(c::five(), c::two() + c::three());
    assert_eq!(c::seven(), c::three() + c::four());
    assert_eq!(c::ten(), c::two() + c::eight());

    assert_eq!(c::pure_2_five(), c::pure_2_two() + c::pure_2_three());
    assert_eq!(c::pure_5_five(), c::pure_5_two() + c::pure_5_three());
    assert_eq!(c::five(), c::pure_2_five() + c::pure_5_five());

}

#[test]
fn neg_adic_composite() {

    assert_eq!(c::neg_one(), -c::one());
    assert_eq!(c::zero(), -c::zero());
    assert_eq!(c::neg_five(), -c::five());
    assert_eq!(c::neg_ten(), -c::ten());

    assert_eq!(c::pure_2_neg_ten(), -c::pure_2_ten());
    assert_eq!(c::pure_5_neg_ten(), -c::pure_5_ten());
    assert_eq!(c::pure_5_neg_ten(), -c::pure_5_ten());
    assert_eq!(c::pure_5_five() + c::pure_2_neg_one(), -(c::pure_5_neg_five() + c::pure_2_one()));

}

#[test]
fn sub_adic_composite() {

    assert_eq!(c::one(), c::two() - c::one());
    assert_eq!(c::zero(), c::three() - c::three());
    assert_eq!(c::neg_one(), c::one() - c::two());
    assert_eq!(c::neg_ten(), c::neg_five() - c::five());
    assert_eq!(c::five(), c::neg_five() - c::neg_ten());

    assert_eq!(c::ten(), c::pure_2_ten() - c::pure_5_neg_ten());
    assert_eq!(c::pure_2_five(), c::five() - c::pure_5_five());
    assert_eq!(c::pure_5_five(), c::five() - c::pure_2_five());

}

#[test]
fn mul_adic_composite() {

    assert_eq!(c::six(), c::two() * c::three());
    assert_eq!(c::neg_ten(), c::five() * c::neg_two());
    assert_eq!(c::seven(), c::seven() * c::one());
    assert_eq!(c::zero(), c::neg_two() * c::zero());

    assert_eq!(c::pure_2_ten(), c::pure_2_two() * c::pure_2_five());
    assert_eq!(c::pure_5_ten(), c::pure_5_two() * c::pure_5_five());
    assert_eq!(c::zero(), c::pure_2_two() * c::pure_5_five());

}

#[test]
fn pow_adic_composite() {

    assert_eq!(c::eight(), c::two().pow(3));
    assert_eq!(c::seven(), c::seven().pow(1));
    assert_eq!(c::one(), c::two().pow(0));
    assert_eq!(c::zero(), c::zero().pow(3));
    assert_eq!(c::one(), c::zero().pow(0));

    assert_eq!(c::pure_2_four(), c::pure_2_two().pow(2));
    assert_eq!(c::pure_5_four(), c::pure_5_two().pow(2));
    assert_eq!(c::pure_2_two(), c::pure_2_two().pow(1));
    assert_eq!(c::pure_5_two(), c::pure_5_two().pow(1));
    assert_eq!(c::one(), c::pure_2_two().pow(0));
    assert_eq!(c::one(), c::pure_5_two().pow(0));

}
