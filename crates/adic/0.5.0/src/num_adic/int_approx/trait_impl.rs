use std::{hash::Hash, iter::repeat, fmt};
use itertools::Either;
use num::{rational::Ratio, traits::Pow, Zero};
use crate::{
    divisible::{Composite, Divisible, PrimePower},
    error::{validate_matching_p, AdicError, AdicResult},
    local_num::{LocalInteger, LocalNum, LocalOne, LocalZero},
    normed::{Normed, UltraNormed, Valuation},
    traits::{AdicPrimitive, CanApproximate, CanTruncate, HasApproximateDigits, HasDigitDisplay, HasDigits, PrimedFrom},
    EAdic, UAdic,
};
use super::ZAdic;


impl PartialEq for ZAdic {
    fn eq(&self, other: &Self) -> bool {
        match (self.c, other.c) {
            (Valuation::PosInf, Valuation::PosInf) => {
                self.variant.eq(&other.variant)
            },
            (Valuation::Finite(c0), Valuation::Finite(c1)) => (
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
            Valuation::PosInf => self.variant.hash(state),
            Valuation::Finite(c) => {
                self.digits().take(c).collect::<Vec<_>>().hash(state);
            },
        }
    }
}


impl fmt::Display for ZAdic {
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

impl LocalZero for ZAdic {
    fn local_zero(&self) -> Self {
        Self::zero(self.p())
    }
    fn is_local_zero(&self) -> bool {
        // Only exact numbers can be zero
        !self.c.is_finite() && self.variant.is_local_zero()
    }
}
impl LocalOne for ZAdic {
    fn local_one(&self) -> Self {
        Self::one(self.p())
    }
    fn is_local_one(&self) -> bool {
        // Only exact numbers can be one
        !self.c.is_finite() && self.variant.is_local_one()
    }
}
impl LocalNum for ZAdic {
    type FromStrRadixErr = AdicError;
    fn from_str_radix(
        s: &str,
        radix: u32,
    ) -> Result<Self, Self::FromStrRadixErr> {
        let digits = s.chars().map(
            |c| c.to_digit(radix).ok_or(AdicError::BadDigit)
        ).collect::<Result<Vec<_>, _>>()?;
        Ok(ZAdic::from(EAdic::new(radix, digits)))
    }
}

impl LocalInteger for ZAdic {
    fn gcd(&self, other: &Self) -> Self {
        validate_matching_p([self.p(), other.p()]);
        let p = self.p();
        match (self.valuation(), other.valuation()) {
            (Valuation::PosInf, Valuation::PosInf) => Self::zero(p),
            (Valuation::PosInf, Valuation::Finite(ov)) => {
                if other.is_approx_zero() {
                    Self::new_approx(p, ov, vec![])
                } else {
                    let pp = PrimePower::try_from((p, ov)).expect("PrimePower from usize");
                    Self::from_prime_power(pp)
                }
            },
            (Valuation::Finite(sv), Valuation::Finite(ov)) if sv > ov => {
                if other.is_approx_zero() {
                    Self::new_approx(p, ov, vec![])
                } else {
                    let pp = PrimePower::try_from((p, ov)).expect("PrimePower from usize");
                    Self::from_prime_power(pp)
                }
            },
            (Valuation::Finite(sv), Valuation::PosInf) => {
                if self.is_approx_zero() {
                    Self::new_approx(p, sv, vec![])
                } else {
                    let pp = PrimePower::try_from((p, sv)).expect("PrimePower from usize");
                    Self::from_prime_power(pp)
                }
            },
            (Valuation::Finite(sv), Valuation::Finite(_ov)) => {
                if self.is_approx_zero() {
                    Self::new_approx(p, sv, vec![])
                } else {
                    let pp = PrimePower::try_from((p, sv)).expect("PrimePower from usize");
                    Self::from_prime_power(pp)
                }
            },
        }
    }
    fn lcm(&self, other: &Self) -> Self {
        validate_matching_p([self.p(), other.p()]);
        let p = self.p();
        match (self.valuation(), other.valuation()) {
            (Valuation::PosInf, _) => Self::zero(p),
            (_, Valuation::PosInf) => Self::zero(p),
            (Valuation::Finite(sv), Valuation::Finite(ov)) if sv > ov => {
                if self.is_approx_zero() || other.is_approx_zero() {
                    Self::new_approx(p, sv, vec![])
                } else {
                    let pp = PrimePower::try_from((p, sv)).expect("PrimePower from usize");
                    Self::from_prime_power(pp)
                }
            },
            (Valuation::Finite(_sv), Valuation::Finite(ov)) => {
                if self.is_approx_zero() || other.is_approx_zero() {
                    Self::new_approx(p, ov, vec![])
                } else {
                    let pp = PrimePower::try_from((p, ov)).expect("PrimePower from usize");
                    Self::from_prime_power(pp)
                }
            },
        }
    }
    fn is_multiple_of(&self, other: &Self) -> bool {
        self.valuation() >= other.valuation()
    }
}

impl Normed for ZAdic {
    type Norm = Ratio<u32>;
    type Unit = ZAdic;
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
            Valuation::Finite(_) if self.is_approx_zero() => None,
            Valuation::Finite(valuation) => Some(self.quotient(valuation)),
        }
    }
    fn into_unit(self) -> Option<Self::Unit> {
        match self.valuation() {
            Valuation::PosInf => None,
            Valuation::Finite(_) if self.is_approx_zero() => None,
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
        if self.has_no_certainty() {
            false
        } else {
            self.valuation().is_zero()
        }
    }
}

impl UltraNormed for ZAdic {
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

impl HasApproximateDigits for ZAdic {
    fn certainty(&self) -> Valuation<Self::DigitIndex> {
        self.c
    }
}
impl CanApproximate for ZAdic {
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


impl HasDigits for ZAdic {
    type DigitIndex = usize;
    fn base(&self) -> Composite {
        self.p().into()
    }
    fn min_index(&self) -> Valuation<Self::DigitIndex> {
        0.into()
    }
    fn num_digits(&self) -> Valuation<usize> {
        match self.c {
            Valuation::PosInf => self.variant.num_digits(),
            c @ Valuation::Finite(_) => c,
        }
    }
    fn digit(&self, n: usize) -> AdicResult<u32> {
        match self.c {
            Valuation::PosInf => self.variant.digit(n),
            Valuation::Finite(c) => {
                if n < c {
                    self.variant.digit(n).or(Ok(0))
                } else {
                    Err(AdicError::InappropriatePrecision(format!("Cannot retrieve digit {n} past certainty {c}")))
                }
            },
        }
    }
    fn digits(&self) -> impl Iterator<Item=u32> {
        let digit_iter = self.variant.digits();
        match self.c {
            Valuation::PosInf => Either::Left(digit_iter),
            Valuation::Finite(c) => Either::Right(digit_iter.chain(repeat(0)).take(c)),
        }
    }
}
impl CanTruncate for ZAdic {
    type Quotient = Self;
    type Truncation = UAdic;
    fn split(&self, n: Self::DigitIndex) -> (Self::Truncation, Self::Quotient) {
        self.clone().into_split(n)
    }
    fn into_split(self, n: Self::DigitIndex) -> (Self::Truncation, Self::Quotient) {
        match self.c {
            Valuation::PosInf => {
                let (before, after) = self.variant.split(n);
                (before, after.into())
            },
            Valuation::Finite(c) => {
                let p = self.variant.p();
                let (before, after) = self.variant.split(n);
                if n < c {
                    (before, ZAdic { c: (c - n).into(), variant: after })
                } else {
                    // This may be bad; if certainty is less than n, it still returns a UAdic,
                    //  even though the UAdic is not known to that certainty.
                    // Consider returning an error instead.
                    (before, Self::empty(p))
                }
            },
        }
    }
}



#[cfg(test)]
mod test {

    use num::rational::Ratio;
    use crate::{
        normed::{Normed, UltraNormed, Valuation},
        traits::{AdicPrimitive, CanApproximate, PrimedFrom},
    };
    use super::ZAdic;

    #[test]
    fn display() {

        // ZAdic exact
        assert_eq!("0._5", ZAdic::from(uadic!(5, [])).to_string());
        assert_eq!("1._5", ZAdic::one(5).to_string());
        assert_eq!("2._5", ZAdic::from(uadic!(5, [2])).to_string());
        assert_eq!("10._5", ZAdic::from(uadic!(5, [0, 1])).to_string());
        assert_eq!("11._5", ZAdic::primed_from(5, 6).to_string());
        assert_eq!("23._5", ZAdic::from(uadic!(5, [3, 2, 0, 0])).to_string());
        assert_eq!("(4)._5", ZAdic::from(eadic_neg!(5, [])).to_string());
        assert_eq!("(4)3._5", ZAdic::from(eadic_neg!(5, [3])).to_string());
        assert_eq!("(4)2._5", (-ZAdic::from(uadic!(5, [3]))).to_string());
        assert_eq!("(4)0._5", ZAdic::from(eadic_neg!(5, [0])).to_string());
        assert_eq!("(4)34._5", ZAdic::primed_from(5, -6).to_string());
        assert_eq!("(4)30._5", ZAdic::from(eadic_neg!(5, [0, 3])).to_string());
        assert_eq!("(4)00._5", ZAdic::from(eadic_neg!(5, [0, 0])).to_string());
        assert_eq!("(u)ka1._31", ZAdic::from(eadic_neg!(31, [1, 10, 20])).to_string());

        // ZAdic approx
        assert_eq!("...0000._5", zadic_approx!(5, 4, []).to_string());
        assert_eq!("...0001._5", zadic_approx!(5, 4, [1]).to_string());
        assert_eq!("...6213._7", zadic_approx!(7, 4, [3, 1, 2, 6, 1, 2]).to_string());
        assert_eq!("...0454._7", (-zadic_approx!(7, 4, [3, 1, 2, 6, 1, 2])).to_string());
        assert_eq!("...1111._5", eadic_rep!(5, [], [1]).into_approximation(4).to_string());
        assert_eq!("...uka1._31", zadic_approx!(31, 4, [1, 10, 20, 30]).to_string());
        assert_eq!("...[30][20][10][1]._37", zadic_approx!(37, 4, [1, 10, 20, 30]).to_string());

    }

    #[test]
    fn normed() {

        let a = ZAdic::from(eadic!(5, [1, 2, 3]));
        assert!(a.is_unit());
        assert_eq!(Ratio::from(1), a.norm());
        assert_eq!(Valuation::Finite(0), a.valuation());
        assert_eq!(Some(a.clone()), a.unit());

        let a = ZAdic::from(eadic!(5, [0, 1, 2]));
        assert!(!a.is_unit());
        assert_eq!(Ratio::new(1, 5), a.norm());
        assert_eq!(Valuation::Finite(1), a.valuation());
        assert_eq!(Some(ZAdic::from(eadic!(5, [1, 2]))), a.unit());

        let a = zadic_approx!(5, 4, [0, 1, 2, 3]);
        assert!(!a.is_unit());
        assert_eq!(Ratio::new(1, 5), a.norm());
        assert_eq!(Valuation::Finite(1), a.valuation());
        assert_eq!(Some(zadic_approx!(5, 3, [1, 2, 3])), a.unit());

        let a = zadic_approx!(5, 4, [0, 0, 0, 0]);
        assert!(!a.is_unit());
        assert_eq!(Ratio::new(1, 625), a.norm());
        assert_eq!(Valuation::Finite(4), a.valuation());
        assert_eq!(None, a.unit());

        let a = ZAdic::empty(5);
        assert!(!a.is_unit());
        assert_eq!(Ratio::from(1), a.norm());
        assert_eq!(Valuation::Finite(0), a.valuation());
        assert_eq!(None, a.unit());

    }

}
