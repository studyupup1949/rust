use crate::{AdicError, AdicNumber, Divisible, HasDigits, Sign};
use super::{
    IAdic, RAdic, UAdic,
};

impl From<UAdic> for IAdic {
    fn from(a: UAdic) -> Self {
        IAdic::new_pos(a.p(), a.into_digits_vec())
    }
}
impl From<UAdic> for RAdic {
    fn from(value: UAdic) -> Self {
        Self::new(value.p(), value.into_digits_vec(), vec![])
    }
}

impl TryFrom<IAdic> for UAdic {
    type Error = AdicError;
    fn try_from(a: IAdic) -> Result<Self, Self::Error> {
        match a.sign {
            Sign::Pos => Ok(UAdic::new(a.p(), a.into_digits().collect())),
            Sign::Neg => Err(AdicError::BadConversion),
        }
    }
}
impl From<IAdic> for RAdic {
    fn from(a: IAdic) -> Self {
        let p = a.p();
        if a.has_finite_digits() {
            Self::new(p, a.into_digits().collect(), vec![])
        } else {
            let num_non_trailing = a.num_non_trailing();
            Self::new(p, a.into_digits().take(num_non_trailing).collect(), vec![p.m1()])
        }
    }
}

impl TryFrom<RAdic> for UAdic {
    type Error = AdicError;
    fn try_from(a: RAdic) -> Result<Self, Self::Error> {
        let p = a.p();
        if a.repeat_digits().next().is_none() {
            Ok(Self::new(p, a.into_fixed_digits().collect()))
        } else {
            Err(AdicError::BadConversion)
        }
    }
}
impl TryFrom<RAdic> for IAdic {
    type Error = AdicError;
    fn try_from(a: RAdic) -> Result<Self, Self::Error> {
        let p = a.p();
        let mut repeat_iter = a.repeat_digits();
        match repeat_iter.next() {
            None => Ok(Self::new_pos(p, a.fixed_digits().collect::<Vec<_>>())),
            Some(digit) => {
                if repeat_iter.next().is_none() && digit == a.p().m1() {
                    Ok(Self::new_neg(p, a.fixed_digits().collect::<Vec<_>>()))
                } else {
                    Err(AdicError::BadConversion)
                }
            }
        }
    }
}
