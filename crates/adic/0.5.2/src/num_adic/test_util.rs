//! Provides constants for use in tests

#![allow(dead_code)]
#![allow(unreachable_pub)]

use crate::traits::AdicPrimitive;
use super::{EAdic, QAdic, UAdic, ZAdic};


pub (crate) mod u {
    use super::*;

    pub fn zero() -> UAdic { uadic!(5, []) }
    pub fn one() -> UAdic { uadic!(5, [1]) }
    pub fn two() -> UAdic { uadic!(5, [2]) }
    pub fn three() -> UAdic { uadic!(5, [3]) }
    pub fn four() -> UAdic { uadic!(5, [4]) }
    pub fn five() -> UAdic { uadic!(5, [0, 1]) }
    pub fn six() -> UAdic { uadic!(5, [1, 1]) }
    pub fn eight() -> UAdic { uadic!(5, [3, 1]) }
    pub fn ten() -> UAdic { uadic!(5, [0, 2]) }
    pub fn twelve() -> UAdic { uadic!(5, [2, 2]) }
    pub fn fifteen() -> UAdic { uadic!(5, [0, 3]) }
    pub fn twenty_four() -> UAdic { uadic!(5, [4, 4]) }
    pub fn twenty_five() -> UAdic { uadic!(5, [0, 0, 1]) }
    pub fn one_twenty_five() -> UAdic { uadic!(5, [0, 0, 0, 1]) }
    pub fn one_fifty_six() -> UAdic { uadic!(5, [1, 1, 1, 1]) }
    pub fn six_twenty_four() -> UAdic { uadic!(5, [4, 4, 4, 4]) }
    pub fn one_twenty_six() -> UAdic { uadic!(5, [1, 0, 0, 1]) }
    pub fn app_neg_one() -> UAdic { uadic!(5, [4, 4, 4, 4]) }
    pub fn app_neg_two() -> UAdic { uadic!(5, [3, 4, 4, 4]) }
    pub fn app_neg_three() -> UAdic { uadic!(5, [2, 4, 4, 4]) }
    pub fn app_neg_five() -> UAdic { uadic!(5, [0, 4, 4, 4]) }
    pub fn app_neg_ten() -> UAdic { uadic!(5, [0, 3, 4, 4]) }

}

pub (crate) mod e {
    use super::*;

    pub fn zero2() -> EAdic { eadic!(2, []) }
    pub fn one2() -> EAdic { eadic!(2, [1]) }
    pub fn two2() -> EAdic { eadic!(2, [0, 1]) }
    pub fn three2() -> EAdic { eadic!(2, [1, 1]) }
    pub fn four2() -> EAdic { eadic!(2, [0, 0, 1]) }
    pub fn eight2() -> EAdic { eadic!(2, [0, 0, 0, 1]) }
    pub fn sixteen2() -> EAdic { eadic!(2, [0, 0, 0, 0, 1]) }
    pub fn seventeen2() -> EAdic { eadic!(2, [1, 0, 0, 0, 1]) }

    pub fn zero3() -> EAdic { eadic!(3, []) }
    pub fn one3() -> EAdic { eadic!(3, [1]) }
    pub fn two3() -> EAdic { eadic!(3, [0, 1]) }
    pub fn ten3() -> EAdic { eadic!(3, [1, 0, 1]) }

    pub fn zero() -> EAdic { eadic!(5, []) }
    pub fn one() -> EAdic { eadic!(5, [1]) }
    pub fn two() -> EAdic { eadic!(5, [2]) }
    pub fn three() -> EAdic { eadic!(5, [3]) }
    pub fn four() -> EAdic { eadic!(5, [4]) }
    pub fn five() -> EAdic { eadic!(5, [0, 1]) }
    pub fn six() -> EAdic { eadic!(5, [1, 1]) }
    pub fn eight() -> EAdic { eadic!(5, [3, 1]) }
    pub fn ten() -> EAdic { eadic!(5, [0, 2]) }
    pub fn fifteen() -> EAdic { eadic!(5, [0, 3]) }
    pub fn twenty_four() -> EAdic { eadic!(5, [4, 4]) }
    pub fn twenty_five() -> EAdic { eadic!(5, [0, 0, 1]) }
    pub fn twenty_six() -> EAdic { eadic!(5, [1, 0, 1]) }
    pub fn one_twenty_five() -> EAdic { eadic!(5, [0, 0, 0, 1]) }
    pub fn six_twenty_five() -> EAdic { eadic!(5, [0, 0, 0, 0, 1]) }
    pub fn neg_one() -> EAdic { eadic_neg!(5, []) }
    pub fn neg_two() -> EAdic { eadic_neg!(5, [3]) }
    pub fn neg_three() -> EAdic { eadic_neg!(5, [2]) }
    pub fn neg_five() -> EAdic { eadic_neg!(5, [0]) }
    pub fn neg_six() -> EAdic { eadic_neg!(5, [4, 3]) }
    pub fn neg_ten() -> EAdic { eadic_neg!(5, [0, 3]) }
    pub fn neg_twenty_five() -> EAdic { eadic_neg!(5, [0, 0] )}
    pub fn neg_one_twenty_five() -> EAdic { eadic_neg!(5, [0, 0, 0]) }
    pub fn neg_one_twenty_six() -> EAdic { eadic_neg!(5, [4, 4, 4, 3]) }

    pub fn zero7() -> EAdic { EAdic::zero(7) }
    pub fn one7() -> EAdic { EAdic::one(7) }
    pub fn two7() -> EAdic { eadic!(7, [2]) }
    pub fn ninety_eight7() -> EAdic { eadic!(7, [0, 0, 2]) }

    pub fn zero_2() -> EAdic { eadic_rep!(2, [], []) }
    pub fn one_2() -> EAdic { eadic_rep!(2, [1], []) }
    pub fn two_2() -> EAdic { eadic_rep!(2, [0, 1], []) }
    pub fn three_2() -> EAdic { eadic_rep!(2, [1, 1], []) }
    pub fn four_2() -> EAdic { eadic_rep!(2, [0, 0, 1], []) }
    pub fn five_2() -> EAdic { eadic_rep!(2, [1, 0, 1], []) }
    pub fn eight_2() -> EAdic { eadic_rep!(2, [0, 0, 0, 1], []) }
    pub fn neg_one_2() -> EAdic { eadic_rep!(2, [], [1]) }
    pub fn neg_five_2() -> EAdic { eadic_rep!(2, [1, 1, 0], [1]) }
    pub fn neg_1_3_2() -> EAdic { eadic_rep!(2, [], [1, 0]) }
    pub fn neg_5_3_2() -> EAdic { eadic_rep!(2, [1, 0, 0], [1, 0]) }
    pub fn neg_8_3_2() -> EAdic { eadic_rep!(2, [0, 0], [0, 1]) }
    pub fn pos_1_9_2() -> EAdic { eadic_rep!(2, [1], [0, 0, 1, 1, 1, 0]) }
    pub fn pos_64_9_2() -> EAdic { eadic_rep!(2, [0, 0, 0, 0, 0, 0, 1], [0, 0, 1, 1, 1, 0]) }

    pub fn seven() -> EAdic { eadic_rep!(5, [2, 1], []) }
    pub fn nine() -> EAdic { eadic_rep!(5, [4, 1], []) }
    pub fn eleven() -> EAdic { eadic_rep!(5, [1, 2], []) }
    pub fn sixteen() -> EAdic { eadic_rep!(5, [1, 3], []) }
    pub fn thirty() -> EAdic { eadic_rep!(5, [0, 1, 1], []) }
    pub fn neg_four() -> EAdic { eadic_rep!(5, [1], [4]) }
    pub fn neg_1_4() -> EAdic { eadic_rep!(5, [], [1]) }
    pub fn pos_1_4() -> EAdic { eadic_rep!(5, [4], [3]) }
    pub fn neg_5_4() -> EAdic { eadic_rep!(5, [0], [1]) }
    pub fn neg_1_2() -> EAdic { eadic_rep!(5, [], [2]) }
    pub fn neg_1_3() -> EAdic { eadic_rep!(5, [], [3, 1]) }
    pub fn pos_1_2() -> EAdic { eadic_rep!(5, [3], [2]) }
    pub fn pos_3_2() -> EAdic { eadic_rep!(5, [4], [2]) }
    pub fn pos_1_3() -> EAdic { eadic_rep!(5, [2], [3, 1]) }
    pub fn pos_1_6() -> EAdic { eadic_rep!(5, [1], [4, 0]) }
    pub fn pos_5_6() -> EAdic { eadic_rep!(5, [0, 1], [4, 0]) }
    pub fn pos_1_8() -> EAdic { eadic_rep!(5, [2], [4, 1]) }
    pub fn pos_1_16() -> EAdic { eadic_rep!(5, [1], [2, 3, 4, 0]) }
    pub fn pos_1_24() -> EAdic { eadic_rep!(5, [4], [4, 3]) }
    pub fn pos_25_24() -> EAdic { eadic_rep!(5, [0, 0], [4, 3]) }
    pub fn neg_1_64() -> EAdic { eadic_rep!(5, [], [1, 3, 1, 1, 2, 4, 2, 2, 3, 0, 4, 3, 4, 1, 0, 0]) }
    pub fn pos_25_16() -> EAdic { eadic_rep!(5, [0, 0, 1], [2, 3, 4, 0]) }
    pub fn pos_43_4() -> EAdic { eadic_rep!(5, [2, 3], [1]) }
    pub fn neg_1_24() -> EAdic { eadic_rep!(5, [], [1, 0]) }
    pub fn neg_5_24() -> EAdic { eadic_rep!(5, [], [0, 1]) }
    pub fn neg_25_24() -> EAdic { eadic_rep!(5, [0], [0, 1]) }
    pub fn neg_1_31() -> EAdic { eadic_rep!(5, [], [4, 0, 0]) }
    pub fn pos_30_31() -> EAdic { one() + neg_1_31() }
    pub fn neg_5_31() -> EAdic { 5 * neg_1_31() }
    pub fn neg_30_31() -> EAdic { 6 * neg_5_31() }
    pub fn neg_1_6() -> EAdic { eadic_rep!(5, [], [4, 0]) }
    pub fn neg_5_6() -> EAdic { eadic_rep!(5, [], [0, 4]) }
    pub fn pos_17_6() -> EAdic { three() + neg_1_6() }

}

pub (crate) mod z {
    use super::*;

    pub fn empty(p: u32) -> ZAdic { ZAdic::empty(p) }

    // Exact numbers
    pub fn zero_e() -> ZAdic { ZAdic::from(uadic!(5, [])) }
    pub fn one_e() -> ZAdic { ZAdic::from(uadic!(5, [1])) }
    pub fn two_e() -> ZAdic { ZAdic::from(uadic!(5, [2])) }
    pub fn three_e() -> ZAdic { ZAdic::from(uadic!(5, [3])) }
    pub fn four_e() -> ZAdic { ZAdic::from(uadic!(5, [4])) }
    pub fn five_e() -> ZAdic { ZAdic::from(uadic!(5, [0, 1])) }
    pub fn six_e() -> ZAdic { ZAdic::from(uadic!(5, [1, 1])) }
    pub fn eight_e() -> ZAdic { ZAdic::from(uadic!(5, [3, 1])) }
    pub fn ten_e() -> ZAdic { ZAdic::from(uadic!(5, [0, 2])) }
    pub fn twenty_e() -> ZAdic { ZAdic::from(uadic!(5, [0, 4])) }
    pub fn twenty_five_e() -> ZAdic { ZAdic::from(uadic!(5, [0, 0, 1])) }
    pub fn neg_one_e() -> ZAdic { ZAdic::from(eadic_neg!(5, [])) }
    pub fn neg_two_e() -> ZAdic { ZAdic::from(eadic_neg!(5, [3])) }
    pub fn neg_five_e() -> ZAdic { ZAdic::from(eadic_neg!(5, [0])) }

    // Numbers with 4-digit certainty
    pub fn zero_4() -> ZAdic { zadic_approx!(5, 4, []) }
    pub fn one_4() -> ZAdic { zadic_approx!(5, 4, [1]) }
    pub fn two_4() -> ZAdic { zadic_approx!(5, 4, [2]) }
    pub fn three_4() -> ZAdic { zadic_approx!(5, 4, [3]) }
    pub fn four_4() -> ZAdic { zadic_approx!(5, 4, [4]) }
    pub fn five_4() -> ZAdic { zadic_approx!(5, 4, [0, 1]) }
    pub fn six_4() -> ZAdic { zadic_approx!(5, 4, [1, 1]) }
    pub fn ten_4() -> ZAdic { zadic_approx!(5, 4, [0, 2]) }
    pub fn fifteen_4() -> ZAdic { zadic_approx!(5, 4, [0, 3]) }
    pub fn twenty_four_4() -> ZAdic { zadic_approx!(5, 4, [4, 4]) }
    pub fn twenty_five_4() -> ZAdic { zadic_approx!(5, 4, [0, 0, 1]) }
    pub fn one_twenty_five_4() -> ZAdic { zadic_approx!(5, 4, [0, 0, 0, 1]) }
    pub fn neg_one_4() -> ZAdic { zadic_approx!(5, 4, [4, 4, 4, 4]) }
    pub fn neg_two_4() -> ZAdic { zadic_approx!(5, 4, [3, 4, 4, 4]) }
    pub fn neg_three_4() -> ZAdic { zadic_approx!(5, 4, [2, 4, 4, 4]) }
    pub fn neg_five_4() -> ZAdic { zadic_approx!(5, 4, [0, 4, 4, 4]) }
    pub fn neg_ten_4() -> ZAdic { zadic_approx!(5, 4, [0, 3, 4, 4]) }

    pub fn neg_1_4_4() -> ZAdic { zadic_approx!(5, 4, [1, 1, 1, 1]) }
    pub fn pos_1_2_4() -> ZAdic { zadic_approx!(5, 4, [3, 2, 2, 2]) }
    pub fn pos_1_3_4() -> ZAdic { zadic_approx!(5, 4, [2, 3, 1, 3]) }
    pub fn pos_1_24_4() -> ZAdic { zadic_approx!(5, 4, [4, 4, 3, 4]) }

    pub fn sqrt_2_7_adic() -> ZAdic { zadic_approx!(7, 4, [3, 1, 2, 6]) }
    pub fn sqrt_2_7_adic2() -> ZAdic { zadic_approx!(7, 4, [4, 5, 4, 0]) }

}

pub (crate) mod qu {
    use super::*;

    type QUAdic = QAdic<UAdic>;

    pub fn zero() -> QUAdic { qadic!(u::zero(), 0) }
    pub fn one() -> QUAdic { qadic!(u::one(), 0) }
    pub fn two() -> QUAdic { qadic!(u::two(), 0) }
    pub fn three() -> QUAdic { qadic!(u::three(), 0) }
    pub fn four() -> QUAdic { qadic!(u::four(), 0) }
    pub fn five() -> QUAdic { qadic!(u::five(), 0) }
    pub fn six() -> QUAdic { qadic!(u::six(), 0) }
    pub fn eight() -> QUAdic { qadic!(u::eight(), 0) }
    pub fn ten() -> QUAdic { qadic!(u::ten(), 0) }
    pub fn fifteen() -> QUAdic { qadic!(u::fifteen(), 0) }
    pub fn twenty_four() -> QUAdic { qadic!(u::twenty_four(), 0) }
    pub fn twenty_five() -> QUAdic { qadic!(u::twenty_five(), 0) }
    pub fn one_fifth() -> QUAdic { qadic!(u::one(), -1) }
    pub fn two_fifth() -> QUAdic { qadic!(u::two(), -1) }
    pub fn three_fifth() -> QUAdic { qadic!(u::three(), -1) }
    pub fn five_fifth() -> QUAdic { qadic!(u::five(), -1) }
    pub fn one_twenty_fifth() -> QUAdic { qadic!(u::one(), -2) }

}

pub (crate) mod qe {
    use super::*;

    type QEAdic = QAdic<EAdic>;

    pub fn zero() -> QEAdic { qadic!(e::zero(), 0) }
    pub fn one() -> QEAdic { qadic!(e::one(), 0) }
    pub fn two() -> QEAdic { qadic!(e::two(), 0) }
    pub fn three() -> QEAdic { qadic!(e::three(), 0) }
    pub fn four() -> QEAdic { qadic!(e::four(), 0) }
    pub fn five() -> QEAdic { qadic!(e::five(), 0) }
    pub fn six() -> QEAdic { qadic!(e::six(), 0) }
    pub fn ten() -> QEAdic { qadic!(e::ten(), 0) }
    pub fn twenty_four() -> QEAdic { qadic!(e::twenty_four(), 0) }
    pub fn neg_one() -> QEAdic { qadic!(e::neg_one(), 0) }
    pub fn neg_two() -> QEAdic { qadic!(e::neg_two(), 0) }
    pub fn neg_three() -> QEAdic { qadic!(e::neg_three(), 0) }
    pub fn neg_five() -> QEAdic { qadic!(e::neg_five(), 0) }
    pub fn neg_six() -> QEAdic { qadic!(e::neg_six(), 0) }
    pub fn neg_ten() -> QEAdic { qadic!(e::neg_ten(), 0) }
    pub fn neg_twenty_five() -> QEAdic { qadic!(e::neg_twenty_five(), 0) }
    pub fn one_fifth() -> QEAdic { qadic!(e::one(), -1) }
    pub fn two_fifth() -> QEAdic { qadic!(e::two(), -1) }
    pub fn three_fifth() -> QEAdic { qadic!(e::three(), -1) }
    pub fn four_fifth() -> QEAdic { qadic!(e::four(), -1) }
    pub fn one_twenty_fifth() -> QEAdic { qadic!(e::one(), -2) }
    pub fn neg_one_fifth() -> QEAdic { qadic!(e::neg_one(), -1) }

    pub fn eight() -> QEAdic { qadic!(e::eight(), 0) }
    pub fn fifteen() -> QEAdic { qadic!(e::fifteen(), 0) }
    pub fn sixteen() -> QEAdic { qadic!(e::sixteen(), 0) }
    pub fn twenty_five() -> QEAdic { qadic!(e::twenty_five(), 0) }
    pub fn thirty() -> QEAdic { qadic!(e::thirty(), 0) }
    pub fn neg_1_2() -> QEAdic { qadic!(e::neg_1_2(), 0) }
    pub fn pos_1_2() -> QEAdic { qadic!(e::pos_1_2(), 0) }
    pub fn neg_1_4() -> QEAdic { qadic!(e::neg_1_4(), 0) }
    pub fn pos_1_4() -> QEAdic { qadic!(e::pos_1_4(), 0) }
    pub fn neg_5_4() -> QEAdic { qadic!(e::neg_1_4(), 1) }
    pub fn pos_1_8() -> QEAdic { qadic!(e::pos_1_8(), 0) }
    pub fn pos_1_16() -> QEAdic { qadic!(e::pos_1_16(), 0) }
    pub fn pos_1_24() -> QEAdic { qadic!(e::pos_1_24(), 0) }
    pub fn neg_1_64() -> QEAdic { qadic!(e::neg_1_64(), 0) }
    pub fn pos_25_16() -> QEAdic { qadic!(e::pos_25_16(), 0) }
    pub fn neg_1_24() -> QEAdic { qadic!(e::neg_1_24(), 0) }
    pub fn neg_5_24() -> QEAdic { qadic!(e::neg_1_24(), 1) }
    pub fn neg_1_120() -> QEAdic { qadic!(e::neg_1_24(), -1) }
    pub fn pos_1_120() -> QEAdic { qadic!(e::pos_1_24(), -1) }
    pub fn neg_1_31() -> QEAdic { qadic!(e::neg_1_31(), 0) }
    pub fn pos_30_31() -> QEAdic { one() + neg_1_31().clone() }
    pub fn neg_5_31() -> QEAdic { 5 * neg_1_31() }
    pub fn neg_1_6() -> QEAdic { qadic!(e::neg_1_6(), 0) }
    pub fn pos_1_6() -> QEAdic { qadic!(e::pos_1_6(), 0) }
    pub fn neg_5_6() -> QEAdic { qadic!(e::neg_1_6(), 1) }
    pub fn pos_5_6() -> QEAdic { qadic!(e::pos_1_6(), 1) }
    pub fn pos_17_6() -> QEAdic { three() + neg_1_6() }
    pub fn pos_1_5() -> QEAdic { qadic!(e::one(), -1) }
    pub fn pos_2_5() -> QEAdic { qadic!(e::two(), -1) }
    pub fn pos_3_5() -> QEAdic { qadic!(e::three(), -1) }
    pub fn pos_1_25() -> QEAdic { qadic!(e::one(), -2) }
    pub fn neg_1_5() -> QEAdic { qadic!(e::neg_one(), -1) }
}

pub (crate) mod qz {
    use super::*;

    type QZAdic = QAdic<ZAdic>;

    // Exact numbers
    pub fn zero_e() -> QZAdic { QZAdic::zero(5) }
    pub fn one_e() -> QZAdic { qadic!(z::one_e(), 0) }
    pub fn two_e() -> QZAdic { qadic!(z::two_e(), 0) }
    pub fn three_e() -> QZAdic { qadic!(z::three_e(), 0) }
    pub fn four_e() -> QZAdic { qadic!(z::four_e(), 0) }
    pub fn five_e() -> QZAdic { qadic!(z::five_e(), 0) }
    pub fn six_e() -> QZAdic { qadic!(z::six_e(), 0) }
    pub fn eight_e() -> QZAdic { qadic!(z::eight_e(), 0) }
    pub fn ten_e() -> QZAdic { qadic!(z::ten_e(), 0) }
    pub fn twenty_e() -> QZAdic { qadic!(z::twenty_e(), 0) }
    pub fn twenty_five_e() -> QZAdic { qadic!(z::twenty_five_e(), 0) }
    pub fn neg_one_e() -> QZAdic { qadic!(z::neg_one_e(), 0) }
    pub fn neg_two_e() -> QZAdic { qadic!(z::neg_two_e(), 0) }
    pub fn neg_five_e() -> QZAdic { qadic!(z::neg_five_e(), 0) }

    // Numbers with 4-digit certainty
    pub fn one_4() -> QZAdic { qadic!(z::one_4(), 0) }
    pub fn two_4() -> QZAdic { qadic!(z::two_4(), 0) }
    pub fn three_4() -> QZAdic { qadic!(z::three_4(), 0) }
    pub fn five_4() -> QZAdic { qadic!(z::one_4(), 1) }
    pub fn seventy_five_4() -> QZAdic { qadic!(z::three_4(), 2) }
    pub fn neg_three_4() -> QZAdic { qadic!(z::neg_three_4(), 0) }
    pub fn neg_1_4_4() -> QZAdic { qadic!(z::neg_1_4_4(), 0) }
    pub fn pos_1_2_4() -> QZAdic { qadic!(z::pos_1_2_4(), 0) }
    pub fn pos_1_3_4() -> QZAdic { qadic!(z::pos_1_3_4(), 0) }
    pub fn pos_1_24_4() -> QZAdic { qadic!(z::pos_1_24_4(), 0) }
    pub fn pos_1_5_4() -> QZAdic { qadic!(z::one_4(), -1) }
    pub fn pos_1_10_4() -> QZAdic { qadic!(z::pos_1_2_4(), -1) }
    pub fn pos_1_15_4() -> QZAdic { qadic!(z::pos_1_3_4(), -1) }
    pub fn pos_1_25_4() -> QZAdic { qadic!(z::one_4(), -2) }

}
