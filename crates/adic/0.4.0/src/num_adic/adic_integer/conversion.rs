use crate::{AdicError, AdicNumber, Divisible, HasDigits, Sign};
use super::{
    z_adic::ZAdicVariant, IAdic, RAdic, UAdic, ZAdic
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
impl From<UAdic> for ZAdic {
    fn from(a: UAdic) -> Self {
        Self::new_exact_pos(a.p(), a.into_digits_vec())
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
impl From<IAdic> for ZAdic {
    fn from(a: IAdic) -> Self {
        let p = a.p();
        let sgn = a.sign;
        let num_non_trailing = a.num_non_trailing();
        let digits = a.into_digits().take(num_non_trailing).collect::<Vec<_>>();
        match sgn {
            Sign::Pos => Self::new_exact_pos(p, digits),
            Sign::Neg => Self::new_exact_neg(p, digits),
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
impl TryFrom<RAdic> for ZAdic {
    type Error = AdicError;
    fn try_from(value: RAdic) -> Result<Self, Self::Error> {
        IAdic::try_from(value).map(ZAdic::from)
    }
}

impl TryFrom<ZAdic> for UAdic {
    type Error = AdicError;
    fn try_from(a: ZAdic) -> Result<Self, Self::Error> {
        if a.has_finite_digits() {
            Ok(Self::new(a.p(), a.into_digits().collect()))
        } else {
            Err(AdicError::BadConversion)
        }
    }
}
impl TryFrom<ZAdic> for IAdic {
    type Error = AdicError;
    fn try_from(a: ZAdic) -> Result<Self, Self::Error> {
        match a.variant {
            ZAdicVariant::Approx(_) => Err(AdicError::BadConversion),
            ZAdicVariant::Exact(var) => Ok(var),
        }
    }
}
impl TryFrom<ZAdic> for RAdic {
    type Error = AdicError;
    fn try_from(value: ZAdic) -> Result<Self, Self::Error> {
        IAdic::try_from(value).map(RAdic::from)
    }
}
