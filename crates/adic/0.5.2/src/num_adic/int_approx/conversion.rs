use crate::{
    normed::Valuation,
    EAdic, IAdic, RAdic, UAdic,
};
use super::ZAdic;

impl From<UAdic> for ZAdic {
    fn from(a: UAdic) -> Self {
        ZAdic {
            c: Valuation::PosInf,
            variant: a.into(),
        }
    }
}

impl From<IAdic> for ZAdic {
    fn from(a: IAdic) -> Self {
        ZAdic {
            c: Valuation::PosInf,
            variant: a.into(),
        }
    }
}

impl From<RAdic> for ZAdic {
    fn from(a: RAdic) -> Self {
        ZAdic {
            c: Valuation::PosInf,
            variant: a.into(),
        }
    }
}

impl From<EAdic> for ZAdic {
   fn from(a: EAdic) -> Self {
        ZAdic {
            c: Valuation::PosInf,
            variant: a,
        }
    }
}
