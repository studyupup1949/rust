use std::{iter::repeat, fmt};
use itertools::Either;
use num::{rational::Ratio, traits::Pow, Zero};
use crate::{
    AdicApproximate, AdicInteger, AdicNumber, AdicResult, AdicSized, AdicValuation,
    Composite, Divisible, HasDigits, Sign, ZAdicValuation,
};
use super::{IAdic, RAdic, UAdic};


macro_rules! impl_display {
    ( $AdicInt:ty ) => {
        impl fmt::Display for $AdicInt {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let p = self.p();
                let digits = self.digit_str();
                if digits.is_empty() {
                    let zero = p.display_zero();
                    write!(f, "{zero}._{p}")
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



impl<A> AdicSized for A
where A: AdicInteger {

    type ValuationRing = usize;
    type AdicUnit = Self;

    fn valuation(&self) -> AdicValuation<usize> {
        if self.is_zero() {
            AdicValuation::PosInf
        } else {
            AdicValuation::Finite(
                self.digits().take_while(Zero::is_zero).count()
            )
        }
    }

    fn norm(&self) -> Ratio<u32> {
        match self.valuation() {
            AdicValuation::PosInf => Ratio::zero(),
            AdicValuation::Finite(valuation) => {
                let v = u32::try_from(valuation).expect("norm usize -> u32 conversion");
                let inv_norm = self.p().pow(v);
                Ratio::new(1, u32::from(inv_norm))
            },
        }
    }

    fn unit(&self) -> Option<Self::AdicUnit> {
        match self.valuation() {
            AdicValuation::PosInf => None,
            AdicValuation::Finite(valuation) => Some(self.quotient(valuation)),
        }
    }

    fn into_unit(self) -> Option<Self::AdicUnit> {
        match self.valuation() {
            AdicValuation::PosInf => None,
            AdicValuation::Finite(valuation) => Some(self.into_quotient(valuation)),
        }
    }

}


impl AdicApproximate for IAdic {
    fn certainty(&self) -> AdicValuation<Self::DigitIndex> {
        ZAdicValuation::PosInf
    }
}

impl AdicApproximate for RAdic {
    fn certainty(&self) -> AdicValuation<Self::DigitIndex> {
        ZAdicValuation::PosInf
    }
}

impl AdicApproximate for UAdic {
    fn certainty(&self) -> AdicValuation<Self::DigitIndex> {
        ZAdicValuation::PosInf
    }
}


impl HasDigits for IAdic {
    type DigitIndex = usize;
    fn base(&self) -> Composite {
        self.p.into()
    }
    fn min_index(&self) -> AdicValuation<Self::DigitIndex> {
        0.into()
    }
    fn num_digits(&self) -> AdicValuation<usize> {
        match self.sign {
            Sign::Pos => AdicValuation::Finite(self.d.len()),
            Sign::Neg => AdicValuation::PosInf,
        }
    }
    fn digit(&self, n: usize) -> AdicResult<u32> {
        Ok(self.d.get(n).copied().unwrap_or(match self.sign {
            Sign::Pos => 0,
            Sign::Neg => self.p.m1(),
        }))
    }
    fn digits(&self) -> impl Iterator<Item=u32> {
        // Returns infinite iterator if num_digits PosInf and finite else
        match self.sign {
            Sign::Pos => Either::Left(self.d.iter().copied()),
            Sign::Neg => Either::Right(self.d.iter().copied().chain(repeat(self.p.m1())))
        }
    }
    fn into_digits(self) -> impl Iterator<Item=u32> {
        // Returns infinite iterator if num_digits PosInf and finite else
        match self.sign {
            Sign::Pos => Either::Left(self.d.into_iter()),
            Sign::Neg => Either::Right(self.d.into_iter().chain(repeat(self.p.m1())))
        }
    }
}

impl HasDigits for RAdic {
    type DigitIndex = usize;
    fn base(&self) -> Composite {
        self.p.into()
    }
    fn min_index(&self) -> AdicValuation<Self::DigitIndex> {
        0.into()
    }
    fn num_digits(&self) -> AdicValuation<usize> {
        if self.rep_d.is_empty() {
            AdicValuation::Finite(self.fix_d.len())
        } else {
            AdicValuation::PosInf
        }
    }
    fn digit(&self, n: usize) -> AdicResult<u32> {
        if n < self.fix_d.len() {
            Ok(self.fix_d.get(n).copied().unwrap_or(0))
        } else if self.rep_d.is_empty() {
            Ok(0)
        } else {
            let diff = n - self.fix_d.len();
            let n_phase = diff % self.rep_d.len();
            Ok(self.rep_d.get(n_phase).copied().unwrap_or(0))
        }

    }
    fn digits(&self) -> impl Iterator<Item=u32> {
        self.fix_d.iter().chain(self.rep_d.iter().cycle()).copied()
    }
    fn into_digits(self) -> impl Iterator<Item=u32> {
        self.fix_d.into_iter().chain(self.rep_d.into_iter().cycle())
    }
}

impl HasDigits for UAdic {
    type DigitIndex = usize;
    fn base(&self) -> Composite {
        self.p.into()
    }
    fn min_index(&self) -> AdicValuation<Self::DigitIndex> {
        0.into()
    }
    fn num_digits(&self) -> AdicValuation<usize> {
        AdicValuation::Finite(self.d.len())
    }
    fn digit(&self, n: usize) -> AdicResult<u32> {
        Ok(self.d.get(n).copied().unwrap_or(0))
    }
    fn digits(&self) -> impl Iterator<Item=u32> {
        self.d.iter().copied()
    }
    fn into_digits(self) -> impl Iterator<Item=u32> {
        self.d.into_iter()
    }
}



#[cfg(test)]
mod test {

    use crate::{
        iadic_pos, iadic_neg, radic, uadic,
        AdicNumber, SignedAdicNumber,
    };
    use super::{IAdic, RAdic, UAdic};


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
        assert_eq!("uka1._31", uadic!(31, [1, 10, 20, 30]).to_string());
        assert_eq!("[30][20][10][1]._37", uadic!(37, [1, 10, 20, 30]).to_string());

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
        assert_eq!("(u)ka1._31", iadic_neg!(31, [1, 10, 20]).to_string());
        assert_eq!("([36])[20][10][1]._37", iadic_neg!(37, [1, 10, 20]).to_string());

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
        assert_eq!("(uk)a1._31", radic!(31, [1, 10], [20, 30]).to_string());
        assert_eq!("([30][20])[10][1]._37", radic!(37, [1, 10], [20, 30]).to_string());

    }

}
