use std::ops;
use num::{
    traits::Pow,
    Integer,
};
use crate::{
    local_num::LocalZero,
    num_adic::{IAdic, IntegerVariant, RAdic},
    traits::{AdicPrimitive, PrimedFrom},
    LazyDiv, UAdic,
};
use super::EAdic;



impl std::ops::Mul<EAdic> for u32 {
    type Output=EAdic;
    fn mul(self, adic_int: EAdic) -> Self::Output {
        let p = adic_int.p();
        EAdic::primed_from(p, self) * adic_int
    }
}



impl ops::Add for EAdic {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        match ExactDoubleVariant::new(self, rhs) {
            ExactDoubleVariant::Unsigned(ua, ub) => EAdic::from(ua + ub),
            ExactDoubleVariant::Signed(ia, ib) => EAdic::from(ia + ib),
            ExactDoubleVariant::Rational(ra, rb) => EAdic::from(ra + rb),
        }
    }
}


impl ops::Neg for EAdic {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self.variant {
            IntegerVariant::Unsigned(u) => IntegerVariant::Signed(-IAdic::from(u)),
            IntegerVariant::Signed(i) => IntegerVariant::Signed(-i),
            IntegerVariant::Rational(r) => IntegerVariant::Rational(-r),
        }.into()
    }
}


impl ops::Mul for EAdic {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        match ExactDoubleVariant::new(self, rhs) {
            ExactDoubleVariant::Unsigned(ua, ub) => EAdic::from(ua * ub),
            ExactDoubleVariant::Signed(ia, ib) => EAdic::from(ia * ib),
            ExactDoubleVariant::Rational(ra, rb) => EAdic::from(ra * rb),
        }
    }
}



impl num::traits::Inv for EAdic {
    type Output = EAdic;
    fn inv(self) -> Self::Output {
        Self::one(self.p()) / self
    }
}

impl ops::Div for EAdic {
    type Output = EAdic;
    fn div(self, rhs: Self) -> Self::Output {
        LazyDiv::new(self, rhs).int_div_exact().expect("Error during EAdic division")
    }
}

impl num::CheckedDiv for EAdic {
    fn checked_div(&self, v: &Self) -> Option<Self> {
        if v.is_local_zero() {
            return None
        }
        Some(self.clone() / v.clone())
    }
}


impl ops::Rem for EAdic {
    type Output = EAdic;
    fn rem(self, rhs: Self) -> Self::Output {
        LazyDiv::new(self, rhs).rem_exact().expect("Error during EAdic remainder")
    }
}



impl_add_assign_from_add!(EAdic);
impl_mul_assign_from_mul!(EAdic);
impl_sub_from_neg!(EAdic);
impl_sub_assign_from_sub!(EAdic);
impl_pow_by_squaring!(EAdic);



enum ExactDoubleVariant {
    Unsigned(UAdic, UAdic),
    Signed(IAdic, IAdic),
    Rational(RAdic, RAdic),
}
impl ExactDoubleVariant {
    fn new(a: EAdic, b: EAdic) -> Self {
        use IntegerVariant::{Unsigned, Signed, Rational};
        match (a.variant, b.variant) {
            (Unsigned(ua), Unsigned(ub)) => ExactDoubleVariant::Unsigned(ua, ub),
            (Unsigned(ua), Signed(ib)) => ExactDoubleVariant::Signed(IAdic::from(ua), ib),
            (Signed(ia), Unsigned(ub)) => ExactDoubleVariant::Signed(ia, IAdic::from(ub)),
            (Signed(ia), Signed(ib)) => ExactDoubleVariant::Signed(ia, ib),
            (Unsigned(ua), Rational(rb)) => ExactDoubleVariant::Rational(RAdic::from(ua), rb),
            (Signed(ia), Rational(rb)) => ExactDoubleVariant::Rational(RAdic::from(ia), rb),
            (Rational(ra), Unsigned(ub)) => ExactDoubleVariant::Rational(ra, RAdic::from(ub)),
            (Rational(ra), Signed(ib)) => ExactDoubleVariant::Rational(ra, RAdic::from(ib)),
            (Rational(ra), Rational(rb)) => ExactDoubleVariant::Rational(ra, rb),
        }
    }
}
