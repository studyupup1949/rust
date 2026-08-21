use crate::{traits::AdicInteger, EAdic, IAdic, QAdic, RAdic, UAdic, ZAdic};


impl<A> From<UAdic> for QAdic<A>
where A: AdicInteger {
    fn from(value: UAdic) -> Self {
        QAdic::new(A::from(value), 0)
    }
}

impl From<QAdic<UAdic>> for QAdic<ZAdic> {
    fn from(value: QAdic<UAdic>) -> Self {
        QAdic::new(ZAdic::from(value.internal_int), value.valuation)
    }
}

impl From<QAdic<IAdic>> for QAdic<ZAdic> {
    fn from(value: QAdic<IAdic>) -> Self {
        QAdic::new(ZAdic::from(value.internal_int), value.valuation)
    }
}

impl From<QAdic<RAdic>> for QAdic<ZAdic> {
    fn from(value: QAdic<RAdic>) -> Self {
        QAdic::new(ZAdic::from(value.internal_int), value.valuation)
    }
}

impl From<QAdic<EAdic>> for QAdic<ZAdic> {
    fn from(value: QAdic<EAdic>) -> Self {
        QAdic::new(ZAdic::from(value.internal_int), value.valuation)
    }
}
