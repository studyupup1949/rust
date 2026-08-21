//! Provides constants for use in tests

#![allow(dead_code)]
#![allow(unreachable_pub)]

use crate::{
    iadic_neg, iadic_pos, qadic, radic, uadic, zadic_approx, zadic_exact,
};
use super::{AdicNumber, IAdic, QAdic, RAdic, UAdic, ZAdic};


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

pub (crate) mod i {
    use super::*;

    pub fn zero2() -> IAdic { iadic_pos!(2, []) }
    pub fn one2() -> IAdic { iadic_pos!(2, [1]) }
    pub fn two2() -> IAdic { iadic_pos!(2, [0, 1]) }
    pub fn three2() -> IAdic { iadic_pos!(2, [1, 1]) }
    pub fn four2() -> IAdic { iadic_pos!(2, [0, 0, 1]) }
    pub fn eight2() -> IAdic { iadic_pos!(2, [0, 0, 0, 1]) }
    pub fn sixteen2() -> IAdic { iadic_pos!(2, [0, 0, 0, 0, 1]) }
    pub fn seventeen2() -> IAdic { iadic_pos!(2, [1, 0, 0, 0, 1]) }

    pub fn zero3() -> IAdic { iadic_pos!(3, []) }
    pub fn one3() -> IAdic { iadic_pos!(3, [1]) }
    pub fn two3() -> IAdic { iadic_pos!(3, [0, 1]) }
    pub fn ten3() -> IAdic { iadic_pos!(3, [1, 0, 1]) }

    pub fn zero() -> IAdic { iadic_pos!(5, []) }
    pub fn one() -> IAdic { iadic_pos!(5, [1]) }
    pub fn two() -> IAdic { iadic_pos!(5, [2]) }
    pub fn three() -> IAdic { iadic_pos!(5, [3]) }
    pub fn four() -> IAdic { iadic_pos!(5, [4]) }
    pub fn five() -> IAdic { iadic_pos!(5, [0, 1]) }
    pub fn six() -> IAdic { iadic_pos!(5, [1, 1]) }
    pub fn eight() -> IAdic { iadic_pos!(5, [3, 1]) }
    pub fn ten() -> IAdic { iadic_pos!(5, [0, 2]) }
    pub fn fifteen() -> IAdic { iadic_pos!(5, [0, 3]) }
    pub fn twenty_four() -> IAdic { iadic_pos!(5, [4, 4]) }
    pub fn twenty_five() -> IAdic { iadic_pos!(5, [0, 0, 1]) }
    pub fn one_twenty_five() -> IAdic { iadic_pos!(5, [0, 0, 0, 1]) }
    pub fn six_twenty_five() -> IAdic { iadic_pos!(5, [0, 0, 0, 0, 1]) }
    pub fn neg_one() -> IAdic { iadic_neg!(5, []) }
    pub fn neg_two() -> IAdic { iadic_neg!(5, [3]) }
    pub fn neg_three() -> IAdic { iadic_neg!(5, [2]) }
    pub fn neg_five() -> IAdic { iadic_neg!(5, [0]) }
    pub fn neg_six() -> IAdic { iadic_neg!(5, [4, 3]) }
    pub fn neg_ten() -> IAdic { iadic_neg!(5, [0, 3]) }
    pub fn neg_twenty_five() -> IAdic { iadic_neg!(5, [0, 0] )}
    pub fn neg_one_twenty_five() -> IAdic { iadic_neg!(5, [0, 0, 0]) }
    pub fn neg_one_twenty_six() -> IAdic { iadic_neg!(5, [4, 4, 4, 3]) }

    pub fn zero7() -> IAdic { IAdic::zero(7) }
    pub fn one7() -> IAdic { IAdic::one(7) }
    pub fn two7() -> IAdic { iadic_pos!(7, [2]) }
    pub fn ninety_eight7() -> IAdic { iadic_pos!(7, [0, 0, 2]) }

}

pub (crate) mod r {
    use super::*;

    pub fn zero_2() -> RAdic { radic!(2, [], []) }
    pub fn one_2() -> RAdic { radic!(2, [1], []) }
    pub fn eight_2() -> RAdic { radic!(2, [0, 0, 0, 1], []) }
    pub fn neg_one_2() -> RAdic { radic!(2, [], [1]) }
    pub fn neg_1_3_2() -> RAdic { radic!(2, [], [1, 0]) }
    pub fn neg_8_3_2() -> RAdic { radic!(2, [0, 0], [0, 1]) }
    pub fn pos_1_9_2() -> RAdic { radic!(2, [1], [0, 0, 1, 1, 1, 0]) }
    pub fn pos_64_9_2() -> RAdic { radic!(2, [0, 0, 0, 0, 0, 0, 1], [0, 0, 1, 1, 1, 0]) }

    pub fn zero() -> RAdic { radic!(5, [], []) }
    pub fn one() -> RAdic { radic!(5, [1], []) }
    pub fn two() -> RAdic { radic!(5, [2], []) }
    pub fn three() -> RAdic { radic!(5, [3], []) }
    pub fn four() -> RAdic { radic!(5, [4], []) }
    pub fn five() -> RAdic { radic!(5, [0, 1], []) }
    pub fn six() -> RAdic { radic!(5, [1, 1], []) }
    pub fn seven() -> RAdic { radic!(5, [2, 1], []) }
    pub fn eight() -> RAdic { radic!(5, [3, 1], []) }
    pub fn nine() -> RAdic { radic!(5, [4, 1], []) }
    pub fn ten() -> RAdic { radic!(5, [0, 2], []) }
    pub fn eleven() -> RAdic { radic!(5, [1, 2], []) }
    pub fn fifteen() -> RAdic { radic!(5, [0, 3], []) }
    pub fn sixteen() -> RAdic { radic!(5, [1, 3], []) }
    pub fn twenty_four() -> RAdic { radic!(5, [4, 4], []) }
    pub fn twenty_five() -> RAdic { radic!(5, [0, 0, 1], []) }
    pub fn thirty() -> RAdic { radic!(5, [0, 1, 1], []) }
    pub fn neg_one() -> RAdic { radic!(5, [], [4]) }
    pub fn neg_two() -> RAdic { radic!(5, [3], [4]) }
    pub fn neg_three() -> RAdic { radic!(5, [2], [4]) }
    pub fn neg_four() -> RAdic { radic!(5, [1], [4]) }
    pub fn neg_five() -> RAdic { radic!(5, [0], [4]) }
    pub fn neg_six() -> RAdic { radic!(5, [4, 3], [4]) }
    pub fn neg_ten() -> RAdic { radic!(5, [0, 3], [4]) }
    pub fn neg_1_4() -> RAdic { radic!(5, [], [1]) }
    pub fn pos_1_4() -> RAdic { radic!(5, [4], [3]) }
    pub fn neg_5_4() -> RAdic { radic!(5, [0], [1]) }
    pub fn neg_1_2() -> RAdic { radic!(5, [], [2]) }
    pub fn pos_1_2() -> RAdic { radic!(5, [3], [2]) }
    pub fn pos_3_2() -> RAdic { radic!(5, [4], [2]) }
    pub fn pos_1_3() -> RAdic { radic!(5, [2], [3, 1]) }
    pub fn pos_1_6() -> RAdic { radic!(5, [1], [4, 0]) }
    pub fn pos_5_6() -> RAdic { radic!(5, [0, 1], [4, 0]) }
    pub fn pos_1_8() -> RAdic { radic!(5, [2], [4, 1]) }
    pub fn pos_1_16() -> RAdic { radic!(5, [1], [2, 3, 4, 0]) }
    pub fn pos_1_24() -> RAdic { radic!(5, [4], [4, 3]) }
    pub fn pos_25_24() -> RAdic { radic!(5, [0, 0], [4, 3]) }
    pub fn neg_1_64() -> RAdic { radic!(5, [], [1, 3, 1, 1, 2, 4, 2, 2, 3, 0, 4, 3, 4, 1, 0, 0]) }
    pub fn pos_25_16() -> RAdic { radic!(5, [0, 0, 1], [2, 3, 4, 0]) }
    pub fn pos_43_4() -> RAdic { radic!(5, [2, 3], [1]) }
    pub fn neg_1_24() -> RAdic { radic!(5, [], [1, 0]) }
    pub fn neg_5_24() -> RAdic { radic!(5, [], [0, 1]) }
    pub fn neg_25_24() -> RAdic { radic!(5, [0], [0, 1]) }
    pub fn neg_1_31() -> RAdic { radic!(5, [], [4, 0, 0]) }
    pub fn pos_30_31() -> RAdic { one() + &neg_1_31() }
    pub fn neg_5_31() -> RAdic { 5 * neg_1_31() }
    pub fn neg_30_31() -> RAdic { 6 * neg_5_31() }
    pub fn neg_1_6() -> RAdic { radic!(5, [], [4, 0]) }
    pub fn neg_5_6() -> RAdic { radic!(5, [], [0, 4]) }
    pub fn pos_17_6() -> RAdic { three() + neg_1_6() }

}

pub (crate) mod z {
    use super::*;

    pub fn empty(p: u32) -> ZAdic { ZAdic::empty(p) }

    // Exact numbers
    pub fn zero_e() -> ZAdic { zadic_exact!(uadic!(5, [])) }
    pub fn one_e() -> ZAdic { zadic_exact!(uadic!(5, [1])) }
    pub fn two_e() -> ZAdic { zadic_exact!(uadic!(5, [2])) }
    pub fn three_e() -> ZAdic { zadic_exact!(uadic!(5, [3])) }
    pub fn four_e() -> ZAdic { zadic_exact!(uadic!(5, [4])) }
    pub fn five_e() -> ZAdic { zadic_exact!(uadic!(5, [0, 1])) }
    pub fn six_e() -> ZAdic { zadic_exact!(uadic!(5, [1, 1])) }
    pub fn eight_e() -> ZAdic { zadic_exact!(uadic!(5, [3, 1])) }
    pub fn ten_e() -> ZAdic { zadic_exact!(uadic!(5, [0, 2])) }
    pub fn twenty_e() -> ZAdic { zadic_exact!(uadic!(5, [0, 4])) }
    pub fn twenty_five_e() -> ZAdic { zadic_exact!(uadic!(5, [0, 0, 1])) }
    pub fn neg_one_e() -> ZAdic { zadic_exact!(iadic_neg!(5, [])) }
    pub fn neg_two_e() -> ZAdic { zadic_exact!(iadic_neg!(5, [3])) }
    pub fn neg_five_e() -> ZAdic { zadic_exact!(iadic_neg!(5, [0])) }

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

pub (crate) mod qi {
    use super::*;

    type QIAdic = QAdic<IAdic>;

    pub fn zero() -> QIAdic { qadic!(i::zero(), 0) }
    pub fn one() -> QIAdic { qadic!(i::one(), 0) }
    pub fn two() -> QIAdic { qadic!(i::two(), 0) }
    pub fn three() -> QIAdic { qadic!(i::three(), 0) }
    pub fn four() -> QIAdic { qadic!(i::four(), 0) }
    pub fn five() -> QIAdic { qadic!(i::five(), 0) }
    pub fn six() -> QIAdic { qadic!(i::six(), 0) }
    pub fn ten() -> QIAdic { qadic!(i::ten(), 0) }
    pub fn twenty_four() -> QIAdic { qadic!(i::twenty_four(), 0) }
    pub fn neg_one() -> QIAdic { qadic!(i::neg_one(), 0) }
    pub fn neg_two() -> QIAdic { qadic!(i::neg_two(), 0) }
    pub fn neg_three() -> QIAdic { qadic!(i::neg_three(), 0) }
    pub fn neg_five() -> QIAdic { qadic!(i::neg_five(), 0) }
    pub fn neg_six() -> QIAdic { qadic!(i::neg_six(), 0) }
    pub fn neg_ten() -> QIAdic { qadic!(i::neg_ten(), 0) }
    pub fn neg_twenty_five() -> QIAdic { qadic!(i::neg_twenty_five(), 0) }
    pub fn one_fifth() -> QIAdic { qadic!(i::one(), -1) }
    pub fn two_fifth() -> QIAdic { qadic!(i::two(), -1) }
    pub fn three_fifth() -> QIAdic { qadic!(i::three(), -1) }
    pub fn four_fifth() -> QIAdic { qadic!(i::four(), -1) }
    pub fn one_twenty_fifth() -> QIAdic { qadic!(i::one(), -2) }
    pub fn neg_one_fifth() -> QIAdic { qadic!(i::neg_one(), -1) }

}

pub (crate) mod qr {
    use super::*;

    type QRAdic = QAdic<RAdic>;

    pub fn zero() -> QRAdic { qadic!(r::zero(), 0) }
    pub fn one() -> QRAdic { qadic!(r::one(), 0) }
    pub fn two() -> QRAdic { qadic!(r::two(), 0) }
    pub fn three() -> QRAdic { qadic!(r::three(), 0) }
    pub fn four() -> QRAdic { qadic!(r::four(), 0) }
    pub fn five() -> QRAdic { qadic!(r::five(), 0) }
    pub fn six() -> QRAdic { qadic!(r::six(), 0) }
    pub fn eight() -> QRAdic { qadic!(r::eight(), 0) }
    pub fn ten() -> QRAdic { qadic!(r::ten(), 0) }
    pub fn sixteen() -> QRAdic { qadic!(r::sixteen(), 0) }
    pub fn twenty_four() -> QRAdic { qadic!(r::twenty_four(), 0) }
    pub fn twenty_five() -> QRAdic { qadic!(r::twenty_five(), 0) }
    pub fn thirty() -> QRAdic { qadic!(r::thirty(), 0) }
    pub fn neg_one() -> QRAdic { qadic!(r::neg_one(), 0) }
    pub fn neg_two() -> QRAdic { qadic!(r::neg_two(), 0) }
    pub fn neg_three() -> QRAdic { qadic!(r::neg_three(), 0) }
    pub fn neg_five() -> QRAdic { qadic!(r::neg_one(), 1) }
    pub fn neg_six() -> QRAdic { qadic!(r::neg_six(), 0) }
    pub fn neg_1_2() -> QRAdic { qadic!(r::neg_1_2(), 0) }
    pub fn pos_1_2() -> QRAdic { qadic!(r::pos_1_2(), 0) }
    pub fn neg_1_4() -> QRAdic { qadic!(r::neg_1_4(), 0) }
    pub fn pos_1_4() -> QRAdic { qadic!(r::pos_1_4(), 0) }
    pub fn neg_5_4() -> QRAdic { qadic!(r::neg_1_4(), 1) }
    pub fn pos_1_8() -> QRAdic { qadic!(r::pos_1_8(), 0) }
    pub fn pos_1_16() -> QRAdic { qadic!(r::pos_1_16(), 0) }
    pub fn pos_1_24() -> QRAdic { qadic!(r::pos_1_24(), 0) }
    pub fn neg_1_64() -> QRAdic { qadic!(r::neg_1_64(), 0) }
    pub fn pos_25_16() -> QRAdic { qadic!(r::pos_25_16(), 0) }
    pub fn neg_1_24() -> QRAdic { qadic!(r::neg_1_24(), 0) }
    pub fn neg_5_24() -> QRAdic { qadic!(r::neg_1_24(), 1) }
    pub fn neg_1_120() -> QRAdic { qadic!(r::neg_1_24(), -1) }
    pub fn pos_1_120() -> QRAdic { qadic!(r::pos_1_24(), -1) }
    pub fn neg_1_31() -> QRAdic { qadic!(r::neg_1_31(), 0) }
    pub fn pos_30_31() -> QRAdic { one() + neg_1_31().clone() }
    pub fn neg_5_31() -> QRAdic { 5 * neg_1_31() }
    pub fn neg_1_6() -> QRAdic { qadic!(r::neg_1_6(), 0) }
    pub fn pos_1_6() -> QRAdic { qadic!(r::pos_1_6(), 0) }
    pub fn neg_5_6() -> QRAdic { qadic!(r::neg_1_6(), 1) }
    pub fn pos_5_6() -> QRAdic { qadic!(r::pos_1_6(), 1) }
    pub fn pos_17_6() -> QRAdic { three() + neg_1_6() }
    pub fn pos_1_5() -> QRAdic { qadic!(r::one(), -1) }
    pub fn pos_2_5() -> QRAdic { qadic!(r::two(), -1) }
    pub fn pos_3_5() -> QRAdic { qadic!(r::three(), -1) }
    pub fn pos_1_25() -> QRAdic { qadic!(r::one(), -2) }
    pub fn neg_1_5() -> QRAdic { qadic!(r::neg_one(), -1) }
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
