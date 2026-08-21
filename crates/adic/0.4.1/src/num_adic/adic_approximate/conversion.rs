use crate::{AdicValuation, ExactIntegerVariant, IAdic, RAdic, UAdic};
use super::ZAdic;

impl From<UAdic> for ZAdic {
    fn from(a: UAdic) -> Self {
        ZAdic {
            c: AdicValuation::PosInf,
            variant: ExactIntegerVariant::Unsigned(a),
        }
    }
}

impl From<IAdic> for ZAdic {
    fn from(a: IAdic) -> Self {
        ZAdic {
            c: AdicValuation::PosInf,
            variant: ExactIntegerVariant::Signed(a),
        }
    }
}

impl From<RAdic> for ZAdic {
    fn from(a: RAdic) -> Self {
        ZAdic {
            c: AdicValuation::PosInf,
            variant: ExactIntegerVariant::Rational(a),
        }
    }
}
