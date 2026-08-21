use std::{hash::Hash, iter::repeat, fmt};
use itertools::Either;
use crate::{
    AdicApproximate, AdicError, AdicInteger, AdicNumber, AdicResult, AdicValuation,
    Composite, Divisible, ExactIntegerVariant, HasDigits, IAdic, RAdic,
};
use super::ZAdic;


impl PartialEq for ZAdic {
    fn eq(&self, other: &Self) -> bool {
        use ExactIntegerVariant::{Unsigned, Signed, Rational};
        match (self.c, other.c) {
            (AdicValuation::PosInf, AdicValuation::PosInf) => {
                match (&self.variant, &other.variant) {
                    (Unsigned(a), Unsigned(b)) => *a == *b,
                    (Unsigned(a), Signed(b)) => IAdic::from(a.clone()) == *b,
                    (Unsigned(a), Rational(b)) => RAdic::from(a.clone()) == *b,
                    (Signed(a), Unsigned(b)) => *a == IAdic::from(b.clone()),
                    (Signed(a), Signed(b)) => *a == *b,
                    (Signed(a), Rational(b)) => RAdic::from(a.clone()) == *b,
                    (Rational(a), Unsigned(b)) => *a == RAdic::from(b.clone()),
                    (Rational(a), Signed(b)) => *a == RAdic::from(b.clone()),
                    (Rational(a), Rational(b)) => *a == *b,
                }
            },
            (AdicValuation::Finite(c0), AdicValuation::Finite(c1)) => (
                c0 == c1 &&
                self.digits().zip(other.digits()).take(c0).all(|(d0, d1)| d0 == d1)
            ),
            _ => false
        }
    }
}
impl Eq for ZAdic { }
impl Hash for ZAdic {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // This is implemented so it matches PartialEq above
        self.c.hash(state);
        match self.c {
            AdicValuation::PosInf => match &self.variant {
                ExactIntegerVariant::Unsigned(u) => { RAdic::from(u.clone()).hash(state); },
                ExactIntegerVariant::Signed(i) => { RAdic::from(i.clone()).hash(state); },
                ExactIntegerVariant::Rational(r) => { r.hash(state); },
            },
            AdicValuation::Finite(c) => {
                self.digits().take(c).collect::<Vec<_>>().hash(state);
            },
        }
    }
}


impl fmt::Display for ZAdic {
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


impl AdicApproximate for ZAdic {
    fn certainty(&self) -> AdicValuation<Self::DigitIndex> {
        self.c
    }
}


impl HasDigits for ZAdic {
    type DigitIndex = usize;
    fn base(&self) -> Composite {
        self.p().into()
    }
    fn min_index(&self) -> AdicValuation<Self::DigitIndex> {
        0.into()
    }
    fn num_digits(&self) -> AdicValuation<usize> {
        match self.c {
            AdicValuation::PosInf => match &self.variant {
                ExactIntegerVariant::Unsigned(u) => u.num_digits(),
                ExactIntegerVariant::Signed(i) => i.num_digits(),
                ExactIntegerVariant::Rational(r) => r.num_digits(),
            },
            c @ AdicValuation::Finite(_) => c,
        }
    }
    fn digit(&self, n: usize) -> AdicResult<u32> {
        match self.c {
            AdicValuation::PosInf => match &self.variant {
                ExactIntegerVariant::Unsigned(u) => u.digit(n),
                ExactIntegerVariant::Signed(i) => i.digit(n),
                ExactIntegerVariant::Rational(r) => r.digit(n),
            },
            AdicValuation::Finite(c) => {
                if n < c {
                    match &self.variant {
                        ExactIntegerVariant::Unsigned(u) => u.digit(n).or(Ok(0)),
                        ExactIntegerVariant::Signed(i) => i.digit(n).or(Ok(0)),
                        ExactIntegerVariant::Rational(r) => r.digit(n).or(Ok(0)),
                    }
                } else {
                    Err(AdicError::InappropriatePrecision(format!("Cannot retrieve digit {n} past certainty {c}")))
                }
            },
        }
    }
    fn digits(&self) -> impl Iterator<Item=u32> {
        let digit_iter = match &self.variant {
            ExactIntegerVariant::Unsigned(u) => Either::Left(u.digits()),
            ExactIntegerVariant::Signed(i) => Either::Right(Either::Left(i.digits())),
            ExactIntegerVariant::Rational(r) => Either::Right(Either::Right(r.digits())),
        };
        match self.c {
            AdicValuation::PosInf => Either::Left(digit_iter),
            AdicValuation::Finite(c) => Either::Right(digit_iter.chain(repeat(0)).take(c)),
        }
    }
    fn into_digits(self) -> impl Iterator<Item=u32> {
        let digit_iter = match self.variant {
            ExactIntegerVariant::Unsigned(u) => Either::Left(u.into_digits()),
            ExactIntegerVariant::Signed(i) => Either::Right(Either::Left(i.into_digits())),
            ExactIntegerVariant::Rational(r) => Either::Right(Either::Right(r.into_digits())),
        };
        match self.c {
            AdicValuation::PosInf => Either::Left(digit_iter),
            AdicValuation::Finite(c) => Either::Right(digit_iter.chain(repeat(0)).take(c)),
        }
    }
}



#[cfg(test)]
mod test {

    use crate::{
        iadic_neg, uadic, radic, zadic_approx, zadic_exact,
        AdicInteger, AdicNumber, SignedAdicNumber,
    };
    use super::ZAdic;


    #[test]
    fn display() {

        // ZAdic exact
        assert_eq!("0._5", zadic_exact!(uadic!(5, [])).to_string());
        assert_eq!("1._5", ZAdic::one(5).to_string());
        assert_eq!("2._5", zadic_exact!(uadic!(5, [2])).to_string());
        assert_eq!("10._5", zadic_exact!(uadic!(5, [0, 1])).to_string());
        assert_eq!("11._5", ZAdic::from_i32(5, 6).to_string());
        assert_eq!("23._5", zadic_exact!(uadic!(5, [3, 2, 0, 0])).to_string());
        assert_eq!("(4)._5", zadic_exact!(iadic_neg!(5, [])).to_string());
        assert_eq!("(4)3._5", zadic_exact!(iadic_neg!(5, [3])).to_string());
        assert_eq!("(4)2._5", (-zadic_exact!(uadic!(5, [3]))).to_string());
        assert_eq!("(4)0._5", zadic_exact!(iadic_neg!(5, [0])).to_string());
        assert_eq!("(4)34._5", ZAdic::from_i32(5, -6).to_string());
        assert_eq!("(4)30._5", zadic_exact!(iadic_neg!(5, [0, 3])).to_string());
        assert_eq!("(4)00._5", zadic_exact!(iadic_neg!(5, [0, 0])).to_string());
        assert_eq!("(u)ka1._31", zadic_exact!(iadic_neg!(31, [1, 10, 20])).to_string());

        // ZAdic approx
        assert_eq!("...0000._5", zadic_approx!(5, 4, []).to_string());
        assert_eq!("...0001._5", zadic_approx!(5, 4, [1]).to_string());
        assert_eq!("...6213._7", zadic_approx!(7, 4, [3, 1, 2, 6, 1, 2]).to_string());
        assert_eq!("...0454._7", (-zadic_approx!(7, 4, [3, 1, 2, 6, 1, 2])).to_string());
        assert_eq!("...1111._5", radic!(5, [], [1]).into_approximation(4).to_string());
        assert_eq!("...uka1._31", zadic_approx!(31, 4, [1, 10, 20, 30]).to_string());
        assert_eq!("...[30][20][10][1]._37", zadic_approx!(37, 4, [1, 10, 20, 30]).to_string());

    }

}
