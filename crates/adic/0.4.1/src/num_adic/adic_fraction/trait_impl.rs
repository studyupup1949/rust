use std::fmt;
use itertools::repeat_n;
use num::{rational::Ratio, traits::Pow, Zero};
use crate::{
    AdicApproximate, AdicError, AdicFraction, AdicInteger, AdicNumber, AdicSized, AdicValuation,
    Composite, HasDigits,
};
use super::QAdic;


impl<A> fmt::Display for QAdic<A>
where A: AdicInteger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        let p = self.p();
        let (frac_str, int_str) = self.frac_int_digit_strs();

        if int_str.is_empty() {
            write!(f, "0.{frac_str}_{p}")
        } else {
            write!(f, "{int_str}.{frac_str}_{p}")
        }

    }
}


impl<A> AdicSized for QAdic<A>
where A: AdicInteger {

    type ValuationRing = isize;
    type AdicUnit = <Self as AdicFraction>::AI;

    fn valuation(&self) -> AdicValuation<isize> {
        self.valuation
    }

    fn norm(&self) -> Ratio<u32> {
        match self.valuation() {
            AdicValuation::PosInf => Ratio::zero(),
            AdicValuation::Finite(valuation) => {
                let v = i32::try_from(valuation).expect("norm isize -> i32 conversion");
                let inv_abs_norm = self.p().pow(v.unsigned_abs());
                if v >= 0 {
                    Ratio::new(1, u32::from(inv_abs_norm))
                } else {
                    Ratio::new(u32::from(inv_abs_norm), 1)
                }
            },
        }
    }

    fn unit(&self) -> Option<Self::AdicUnit> {
        if self.adic_unit.is_zero() {
            None
        } else {
            Some(self.adic_unit.clone())
        }
    }

    fn into_unit(self) -> Option<Self::AdicUnit> {
        if self.adic_unit.is_zero() {
            None
        } else {
            Some(self.adic_unit)
        }
    }

}

impl<A> AdicApproximate for QAdic<A>
where A: AdicInteger + AdicApproximate {

    fn certainty(&self) -> AdicValuation<isize> {
        match (self.adic_unit.certainty(), self.valuation()) {
            (AdicValuation::Finite(c), AdicValuation::Finite(v)) => {
                let cisize = isize::try_from(c).expect("certainty usize -> isize conversion");
                AdicValuation::Finite(cisize + v)
            },
            _ => AdicValuation::PosInf
        }
    }

}


impl<A> HasDigits for QAdic<A>
where A: AdicInteger {

    type DigitIndex = isize;

    fn base(&self) -> Composite {
        self.adic_unit.base()
    }

    fn min_index(&self) -> AdicValuation<Self::DigitIndex> {
        match self.valuation() {
            AdicValuation::Finite(v) if v < 0 => v.into(),
            _ => 0.into(),
        }
    }

    fn num_digits(&self) -> AdicValuation<usize> {
        match self.valuation() {
            AdicValuation::PosInf => 0.into(),
            AdicValuation::Finite(v) if v >= 0 => AdicValuation::from(v.unsigned_abs()) + self.unit_ref().num_digits(),
            AdicValuation::Finite(_) => self.unit_ref().num_digits(),
        }
    }

    fn digit(&self, n: isize) -> Result<u32, AdicError> {
        match self.valuation() {
            AdicValuation::PosInf => Ok(0),
            AdicValuation::Finite(v) => {
                if n < v {
                    Ok(0)
                } else {
                    self.unit_ref().digit((n-v).unsigned_abs())
                }
            }
        }
    }

    fn digits(&self) -> impl Iterator<Item=u32> {
        let num_zeros = match self.valuation() {
            AdicValuation::Finite(v) if v >= 0 => v.unsigned_abs(),
            _ => 0,
        };
        repeat_n(0, num_zeros).chain(
            self.unit().map(|u| u.clone().into_digits()).into_iter().flatten()
        )
    }

    fn into_digits(self) -> impl Iterator<Item = u32> {
        let num_zeros = match self.valuation() {
            AdicValuation::Finite(v) if v >= 0 => v.unsigned_abs(),
            _ => 0,
        };
        repeat_n(0, num_zeros).chain(
            self.into_unit().map(A::into_digits).into_iter().flatten()
        )
    }

}



#[cfg(test)]
mod test {

    use crate::{
        iadic_neg, qadic, radic, uadic,
        zadic_approx, zadic_exact,
        UAdic,
    };
    use super::{AdicNumber, QAdic};


    #[test]
    fn display() {

        // UAdic
        assert_eq!("0._5", qadic!(uadic!(5, []), 0).to_string());
        assert_eq!("0._5", qadic!(uadic!(5, []), 1).to_string());
        assert_eq!("1._5", QAdic::<UAdic>::one(5).to_string());
        assert_eq!("2._5", qadic!(uadic!(5, [2]), 0).to_string());
        assert_eq!("20._5", qadic!(uadic!(5, [2]), 1).to_string());
        assert_eq!("0.2_5", qadic!(uadic!(5, [2]), -1).to_string());
        assert_eq!("100._5", qadic!(uadic!(5, [0, 1]), 1).to_string());
        assert_eq!("11._5", QAdic::<UAdic>::from_u32(5, 6).to_string());
        assert_eq!("220._5", qadic!(uadic!(5, [2, 2, 0, 0]), 1).to_string());
        assert_eq!("3.2_5", qadic!(uadic!(5, [2, 3]), -1).to_string());
        assert_eq!("0.032_5", qadic!(uadic!(5, [2, 3]), -3).to_string());
        assert_eq!("uk.a1_31", qadic!(uadic!(31, [1, 10, 20, 30]), -2).to_string());
        assert_eq!("[30][20].[10][1]_37", qadic!(uadic!(37, [1, 10, 20, 30]), -2).to_string());

        // IAdic
        assert_eq!("(4)._5", qadic!(iadic_neg!(5, []), 0).to_string());
        assert_eq!("(4)3._5", qadic!(iadic_neg!(5, [3]), 0).to_string());
        assert_eq!("(4)30._5", qadic!(iadic_neg!(5, [3]), 1).to_string());
        assert_eq!("(4).3_5", qadic!(iadic_neg!(5, [3]), -1).to_string());
        assert_eq!("(4).443_5", qadic!(iadic_neg!(5, [3]), -3).to_string());
        assert_eq!("(u)k.a1_31", qadic!(iadic_neg!(31, [1, 10, 20]), -2).to_string());
        assert_eq!("([36])[20].[10][1]_37", qadic!(iadic_neg!(37, [1, 10, 20]), -2).to_string());

        // RAdic
        assert_eq!("(4)._5", qadic!(radic!(5, [], [4]), 0).to_string());
        assert_eq!("(01)._5", qadic!(radic!(5, [], [1, 0]), 0).to_string());
        assert_eq!("(10)._5", qadic!(radic!(5, [], [1, 0]), 1).to_string());
        assert_eq!("(10)00._5", qadic!(radic!(5, [], [1, 0]), 3).to_string());
        assert_eq!("(10).1_5", qadic!(radic!(5, [], [1, 0]), -1).to_string());
        assert_eq!("(01).0101_5", qadic!(radic!(5, [], [1, 0]), -4).to_string());
        assert_eq!("(uk)a.1_31", qadic!(radic!(31, [1, 10], [20, 30]), -1).to_string());
        assert_eq!("([30][20])[10].[1]_37", qadic!(radic!(37, [1, 10], [20, 30]), -1).to_string());

        // ZAdic
        assert_eq!("20._5", qadic!(zadic_exact!(uadic!(5, [2])), 1).to_string());
        assert_eq!("...0000._5", qadic!(zadic_approx!(5, 4, []), 0).to_string());
        assert_eq!("...00000._5", qadic!(zadic_approx!(5, 4, []), 1).to_string());
        assert_eq!("...000._5", qadic!(zadic_approx!(5, 4, []), -1).to_string());
        assert_eq!("...0001._5", qadic!(zadic_approx!(5, 4, [1]), 0).to_string());
        assert_eq!("...00010._5", qadic!(zadic_approx!(5, 4, [1]), 1).to_string());
        assert_eq!("...00.01_5", qadic!(zadic_approx!(5, 4, [1]), -2).to_string());
        assert_eq!("...6213._7", qadic!(zadic_approx!(7, 4, [3, 1, 2, 6, 1, 2]), 0).to_string());
        assert_eq!("...621300._7", qadic!(zadic_approx!(7, 4, [3, 1, 2, 6, 1, 2]), 2).to_string());
        assert_eq!("...62.13_7", qadic!(zadic_approx!(7, 4, [3, 1, 2, 6, 1, 2]), -2).to_string());
        assert_eq!("...uk.a1_31", qadic!(zadic_approx!(31, 4, [1, 10, 20, 30]), -2).to_string());
        assert_eq!("...[30][20].[10][1]_37", qadic!(zadic_approx!(37, 4, [1, 10, 20, 30]), -2).to_string());

    }

}
