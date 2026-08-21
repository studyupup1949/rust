use crate::UAdic;
use crate::num_adic::{IAdic, RAdic};
use super::{EAdic, IntegerVariant};


impl From<IntegerVariant> for EAdic {
    fn from(value: IntegerVariant) -> Self {
        Self {
            variant: value,
        }
    }
}

impl From<UAdic> for EAdic {
    fn from(a: UAdic) -> Self {
        IntegerVariant::from(a).into()
    }
}

impl From<IAdic> for EAdic {
    fn from(a: IAdic) -> Self {
        IntegerVariant::from(a).into()
    }
}

impl From<RAdic> for EAdic {
    fn from(a: RAdic) -> Self {
        IntegerVariant::from(a).into()
    }
}

impl From<EAdic> for RAdic {
    fn from(a: EAdic) -> Self {
        match a.variant {
            IntegerVariant::Unsigned(u) => u.into(),
            IntegerVariant::Signed(i) => i.into(),
            IntegerVariant::Rational(r) => r,
        }
    }
}
