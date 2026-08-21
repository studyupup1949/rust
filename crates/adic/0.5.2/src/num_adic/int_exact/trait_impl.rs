use std::{hash::Hash, fmt};
use itertools::Either;
use num::{rational::Ratio, traits::Pow, Zero};
use crate::{
    divisible::{Composite, Divisible, PrimePower},
    error::{validate_matching_p, AdicError, AdicResult},
    local_num::{LocalInteger, LocalNum, LocalOne, LocalZero},
    normed::{Normed, UltraNormed, Valuation},
    num_adic::{IAdic, RAdic},
    traits::{AdicPrimitive, CanApproximate, CanTruncate, HasApproximateDigits, HasDigitDisplay, HasDigits, PrimedFrom},
    UAdic, ZAdic,
};
use super::{EAdic, IntegerVariant};



impl PartialEq for EAdic {
    fn eq(&self, other: &Self) -> bool {
        use IntegerVariant::{Unsigned, Signed, Rational};
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
    }
}
impl Eq for EAdic { }
impl Hash for EAdic {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // This is implemented so it matches PartialEq above
        match &self.variant {
            IntegerVariant::Unsigned(u) => { RAdic::from(u.clone()).hash(state); },
            IntegerVariant::Signed(i) => { RAdic::from(i.clone()).hash(state); },
            IntegerVariant::Rational(r) => { r.hash(state); },
        }
    }
}


impl fmt::Display for EAdic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let p = self.p();
        let digits = self.digit_display();
        if digits.is_empty() {
            let zero = p.display_zero();
            write!(f, "{zero}._{p}")
        } else {
            write!(f, "{digits}._{p}")
        }
    }
}



// local_num

impl LocalZero for EAdic {
    fn local_zero(&self) -> Self {
        Self::zero(self.p())
    }
    fn is_local_zero(&self) -> bool {
        self.variant.is_local_zero()
    }
}
impl LocalOne for EAdic {
    fn local_one(&self) -> Self {
        Self::one(self.p())
    }
    fn is_local_one(&self) -> bool {
        self.variant.is_local_one()
    }
}
impl LocalNum for EAdic {
    type FromStrRadixErr = AdicError;
    fn from_str_radix(
        s: &str,
        radix: u32,
    ) -> Result<Self, Self::FromStrRadixErr> {
        let digits = s.chars().map(
            |c| c.to_digit(radix).ok_or(AdicError::BadDigit)
        ).collect::<Result<Vec<_>, _>>()?;
        Ok(EAdic::new(radix, digits))
    }
}

impl LocalInteger for EAdic {
    fn gcd(&self, other: &Self) -> Self {
        validate_matching_p([self.p(), other.p()]);
        let p = self.p();
        match std::cmp::min(self.valuation(), other.valuation()) {
            Valuation::PosInf => Self::zero(p),
            Valuation::Finite(v) => {
                let pp = PrimePower::try_from((p, v)).expect("PrimePower from usize");
                Self::from_prime_power(pp)
            },
        }
    }
    fn lcm(&self, other: &Self) -> Self {
        validate_matching_p([self.p(), other.p()]);
        let p = self.p();
        match std::cmp::max(self.valuation(), other.valuation()) {
            Valuation::PosInf => Self::zero(p),
            Valuation::Finite(v) => {
                let pp = PrimePower::try_from((p, v)).expect("PrimePower from usize");
                Self::from_prime_power(pp)
            }
        }
    }
    fn is_multiple_of(&self, other: &Self) -> bool {
        self.valuation() >= other.valuation()
    }
}

impl Normed for EAdic {
    type Norm = Ratio<u32>;
    type Unit = EAdic;
    fn norm(&self) -> Self::Norm {
        match self.valuation() {
            Valuation::PosInf => Ratio::zero(),
            Valuation::Finite(valuation) => {
                let v = u32::try_from(valuation).expect("norm usize -> u32 conversion");
                let inv_norm = self.p().pow(v);
                Ratio::new(1, u32::from(inv_norm))
            },
        }
    }
    fn unit(&self) -> Option<Self::Unit> {
        match self.valuation() {
            Valuation::PosInf => None,
            Valuation::Finite(valuation) => Some(self.quotient(valuation)),
        }
    }
    fn into_unit(self) -> Option<Self::Unit> {
        match self.valuation() {
            Valuation::PosInf => None,
            Valuation::Finite(valuation) => Some(self.into_quotient(valuation)),
        }
    }
    fn from_norm_and_unit(norm: Self::Norm, u: Self::Unit) -> Self {
        let (numer, denom) = norm.into_raw();
        if numer == 0 {
            u.local_zero()
        } else if numer == 1 {
            let adic_inv_norm = Self::primed_from(u.p(), denom);
            adic_inv_norm * u
        } else {
            panic!("Cannot invert norm to adic integer");
        }
    }
    fn from_unit(u: Self::Unit) -> Self {
        u
    }
    fn is_unit(&self) -> bool {
        self.valuation().is_zero()
    }
}

impl UltraNormed for EAdic {
    type ValuationRing = usize;
    fn from_unit_and_valuation(u: Self::Unit, v: Valuation<Self::ValuationRing>) -> Self {
        let p = u.p();
        match v {
            Valuation::Finite(v) => {
                let pp = PrimePower::try_from((p, v)).expect("PrimePower from usize");
                u * Self::from_prime_power(pp)
            },
            Valuation::PosInf => Self::zero(p),
        }
    }
    fn valuation(&self) -> Valuation<usize> {
        if self.is_local_zero() {
            Valuation::PosInf
        } else {
            Valuation::Finite(
                self.digits().take_while(Zero::is_zero).count()
            )
        }
    }
}




// HasDigits/CanTruncate

impl HasApproximateDigits for EAdic {
    fn certainty(&self) -> Valuation<Self::DigitIndex> {
        Valuation::PosInf
    }
}
impl CanApproximate for EAdic {
    type Approximation = ZAdic;
    fn approximation(&self, n: usize) -> Self::Approximation {
        let c = match self.certainty() {
            Valuation::PosInf => n,
            Valuation::Finite(v) => std::cmp::min(v, n),
        };
        ZAdic::new_approx(self.p(), c, self.digits().take(c).collect())
    }
    fn into_approximation(self, n: usize) -> Self::Approximation {
        let c = match self.certainty() {
            Valuation::PosInf => n,
            Valuation::Finite(v) => std::cmp::min(v, n),
        };
        ZAdic::new_approx(self.p(), c, self.digits().take(c).collect())
    }
}

impl HasDigits for EAdic {
    type DigitIndex = usize;
    fn base(&self) -> Composite {
        self.p().into()
    }
    fn min_index(&self) -> Valuation<Self::DigitIndex> {
        0.into()
    }
    fn num_digits(&self) -> Valuation<usize> {
        match &self.variant {
            IntegerVariant::Unsigned(u) => u.num_digits(),
            IntegerVariant::Signed(i) => i.num_digits(),
            IntegerVariant::Rational(r) => r.num_digits(),
        }
    }
    fn digit(&self, n: usize) -> AdicResult<u32> {
        match &self.variant {
            IntegerVariant::Unsigned(u) => u.digit(n),
            IntegerVariant::Signed(i) => i.digit(n),
            IntegerVariant::Rational(r) => r.digit(n),
        }
    }
    fn digits(&self) -> impl Iterator<Item=u32> {
        match &self.variant {
            IntegerVariant::Unsigned(u) => Either::Left(u.digits()),
            IntegerVariant::Signed(i) => Either::Right(Either::Left(i.digits())),
            IntegerVariant::Rational(r) => Either::Right(Either::Right(r.digits())),
        }
    }
}
impl CanTruncate for EAdic {
    type Quotient = Self;
    type Truncation = UAdic;
    fn split(&self, n: Self::DigitIndex) -> (Self::Truncation, Self::Quotient) {
        self.clone().into_split(n)
    }
    fn into_split(self, n: Self::DigitIndex) -> (Self::Truncation, Self::Quotient) {
        match self.variant {
            IntegerVariant::Unsigned(u) => {
                let (before, after) = u.into_split(n);
                (before, Self::from(after))
            },
            IntegerVariant::Signed(i) => {
                let (before, after) = i.into_split(n);
                (before, Self::from(after))
            },
            IntegerVariant::Rational(r) => {
                let (before, after) = r.into_split(n);
                (before, Self::from(after))
            },
        }
    }
}


#[cfg(test)]
mod test {

    use num::rational::Ratio;
    use crate::{
        normed::{Normed, UltraNormed, Valuation},
        traits::{AdicPrimitive, PrimedFrom},
    };
    use super::EAdic;

    #[test]
    fn display() {

        assert_eq!("0._5", eadic!(5, []).to_string());
        assert_eq!("1._5", EAdic::one(5).to_string());
        assert_eq!("2._5", eadic!(5, [2]).to_string());
        assert_eq!("10._5", eadic!(5, [0, 1]).to_string());
        assert_eq!("11._5", EAdic::primed_from(5, 6).to_string());
        assert_eq!("23._5", eadic!(5, [3, 2, 0, 0]).to_string());
        assert_eq!("(4)._5", eadic_neg!(5, []).to_string());
        assert_eq!("(4)3._5", eadic_neg!(5, [3]).to_string());
        assert_eq!("(4)2._5", (-eadic!(5, [3])).to_string());
        assert_eq!("(4)0._5", eadic_neg!(5, [0]).to_string());
        assert_eq!("(4)34._5", EAdic::primed_from(5, -6).to_string());
        assert_eq!("(4)30._5", eadic_neg!(5, [0, 3]).to_string());
        assert_eq!("(4)00._5", eadic_neg!(5, [0, 0]).to_string());
        assert_eq!("(u)ka1._31", eadic_neg!(31, [1, 10, 20]).to_string());

    }

    #[test]
    fn normed() {

        let a = eadic!(5, [1, 2, 3]);
        assert!(a.is_unit());
        assert_eq!(Ratio::from(1), a.norm());
        assert_eq!(Valuation::Finite(0), a.valuation());
        assert_eq!(Some(a.clone()), a.unit());

        let a = eadic!(5, [0, 1, 2]);
        assert!(!a.is_unit());
        assert_eq!(Ratio::new(1, 5), a.norm());
        assert_eq!(Valuation::Finite(1), a.valuation());
        assert_eq!(Some(eadic!(5, [1, 2])), a.unit());

        let a = eadic_neg!(5, [0, 1, 2]);
        assert!(!a.is_unit());
        assert_eq!(Ratio::new(1, 5), a.norm());
        assert_eq!(Valuation::Finite(1), a.valuation());
        assert_eq!(Some(eadic_neg!(5, [1, 2])), a.unit());

    }

}
