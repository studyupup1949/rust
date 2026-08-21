use std::fmt;
use super::{AdicInteger, IAdic, RAdic, UAdic, ZAdic};


macro_rules! impl_display {
    ( $AdicInt:ty ) => {
        impl fmt::Display for $AdicInt {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let p = self.p();
                let digits = self.digit_str();
                if digits.is_empty() {
                    write!(f, "0._{p}")
                } else {
                    write!(f, "{digits}._{p}")
                }
            }
        }
    }
}

impl_display!(IAdic);
impl_display!(RAdic);
impl_display!(UAdic);
impl_display!(ZAdic);



#[cfg(test)]
mod test {

    use crate::{
        iadic_pos, iadic_neg, radic, uadic,
        zadic_approx, zadic_exact_pos, zadic_exact_neg,
        SignedAdicInteger,
    };
    use super::{AdicInteger, IAdic, RAdic, UAdic, ZAdic};


    #[test]
    fn display() {

        // UAdic
        assert_eq!("0._5", uadic!(5, []).to_string());
        assert_eq!("1._5", UAdic::one(5).to_string());
        assert_eq!("2._5", uadic!(5, [2]).to_string());
        assert_eq!("3._5", uadic!(5, [3]).to_string());
        assert_eq!("4._5", uadic!(5, [4]).to_string());
        assert_eq!("10._5", uadic!(5, [0, 1]).to_string());
        assert_eq!("11._5", UAdic::from_u32(5, 6).to_string());
        assert_eq!("20._5", uadic!(5, [0, 2]).to_string());
        assert_eq!("44._5", uadic!(5, [4, 4]).to_string());
        assert_eq!("100._5", uadic!(5, [0, 0, 1]).to_string());
        assert_eq!("22._5", uadic!(5, [2, 2, 0, 0]).to_string());
        assert_eq!("1111._5", uadic!(5, [1, 1, 1, 1]).to_string());
        assert_eq!("4444._5", uadic!(5, [4, 4, 4, 4]).to_string());
        assert_eq!("1000._5", uadic!(5, [0, 0, 0, 1, 0, 0]).to_string());
        assert_eq!("1001._5", uadic!(5, [1, 0, 0, 1]).to_string());

        // IAdic
        assert_eq!("0._5", iadic_pos!(5, []).to_string());
        assert_eq!("1._5", IAdic::one(5).to_string());
        assert_eq!("2._5", iadic_pos!(5, [2]).to_string());
        assert_eq!("3._5", iadic_pos!(5, [3]).to_string());
        assert_eq!("4._5", iadic_pos!(5, [4]).to_string());
        assert_eq!("10._5", iadic_pos!(5, [0, 1]).to_string());
        assert_eq!("11._5", IAdic::from_i32(5, 6).to_string());
        assert_eq!("20._5", iadic_pos!(5, [0, 2]).to_string());
        assert_eq!("44._5", iadic_pos!(5, [4, 4]).to_string());
        assert_eq!("100._5", iadic_pos!(5, [0, 0, 1]).to_string());
        assert_eq!("22._5", iadic_pos!(5, [2, 2, 0, 0]).to_string());
        assert_eq!("(4)._5", iadic_neg!(5, []).to_string());
        assert_eq!("(4)3._5", iadic_neg!(5, [3]).to_string());
        assert_eq!("(4)2._5", (-iadic_pos!(5, [3])).to_string());
        assert_eq!("(4)0._5", iadic_neg!(5, [0]).to_string());
        assert_eq!("(4)34._5", IAdic::from_i32(5, -6).to_string());
        assert_eq!("(4)30._5", iadic_neg!(5, [0, 3]).to_string());
        assert_eq!("(4)00._5", iadic_neg!(5, [0, 0]).to_string());

        // RAdic
        assert_eq!("0._5", radic!(5, [], []).to_string());
        assert_eq!("1._5", RAdic::one(5).to_string());
        assert_eq!("2._5", radic!(5, [2], []).to_string());
        assert_eq!("3._5", radic!(5, [3], []).to_string());
        assert_eq!("4._5", radic!(5, [4], []).to_string());
        assert_eq!("10._5", radic!(5, [0, 1], []).to_string());
        assert_eq!("11._5", RAdic::from_i32(5, 6).to_string());
        assert_eq!("20._5", radic!(5, [0, 2], []).to_string());
        assert_eq!("22._5", radic!(5, [2, 2], []).to_string());
        assert_eq!("100._5", radic!(5, [0, 0, 1], []).to_string());
        assert_eq!("(4)._5", radic!(5, [], [4]).to_string());
        assert_eq!("(4)3._5", radic!(5, [3], [4]).to_string());
        assert_eq!("(4)2._5", (-radic!(5, [3], [])).to_string());
        assert_eq!("(4)1._5", radic!(5, [1], [4]).to_string());
        assert_eq!("(4)0._5", radic!(5, [0], [4]).to_string());
        assert_eq!("(4)30._5", radic!(5, [0, 3], [4]).to_string());
        assert_eq!("(1)._5", radic!(5, [], [1]).to_string());
        assert_eq!("(3)4._5", (-radic!(5, [], [1])).to_string());
        assert_eq!("(1)0._5", radic!(5, [0], [1]).to_string());
        assert_eq!("(1)32._5", radic!(5, [2, 3, 1, 1], [1]).to_string());
        assert_eq!("(01)._5", radic!(5, [], [1, 0]).to_string());
        assert_eq!("(10)._5", (radic!(5, [0, 1], []) * radic!(5, [], [1, 0])).to_string());
        assert_eq!("(004)._5", radic!(5, [], [4, 0, 0, 4, 0, 0]).to_string());
        assert_eq!("(04)._5", radic!(5, [4, 0, 4], [0, 4]).to_string());

        // ZAdic exact
        assert_eq!("0._5", zadic_exact_pos!(5, []).to_string());
        assert_eq!("1._5", ZAdic::one(5).to_string());
        assert_eq!("2._5", zadic_exact_pos!(5, [2]).to_string());
        assert_eq!("10._5", zadic_exact_pos!(5, [0, 1]).to_string());
        assert_eq!("11._5", ZAdic::from_i32(5, 6).to_string());
        assert_eq!("23._5", zadic_exact_pos!(5, [3, 2, 0, 0]).to_string());
        assert_eq!("(4)._5", zadic_exact_neg!(5, []).to_string());
        assert_eq!("(4)3._5", zadic_exact_neg!(5, [3]).to_string());
        assert_eq!("(4)2._5", (-zadic_exact_pos!(5, [3])).to_string());
        assert_eq!("(4)0._5", zadic_exact_neg!(5, [0]).to_string());
        assert_eq!("(4)34._5", ZAdic::from_i32(5, -6).to_string());
        assert_eq!("(4)30._5", zadic_exact_neg!(5, [0, 3]).to_string());
        assert_eq!("(4)00._5", zadic_exact_neg!(5, [0, 0]).to_string());

        // ZAdic approx
        assert_eq!("...0000._5", zadic_approx!(5, 4, []).to_string());
        assert_eq!("...0001._5", zadic_approx!(5, 4, [1]).to_string());
        assert_eq!("...6213._7", zadic_approx!(7, 4, [3, 1, 2, 6, 1, 2]).to_string());
        assert_eq!("...0454._7", (-zadic_approx!(7, 4, [3, 1, 2, 6, 1, 2])).to_string());
        assert_eq!("...1111._5", radic!(5, [], [1]).into_approximation(4).to_string());

    }

}
