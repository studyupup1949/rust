use std::ops;
use num::{
    traits::Pow,
    Integer,
};
use crate::{
    error::validate_matching_p,
    local_num::LocalZero,
    normed::{UltraNormed, Valuation},
    traits::{AdicPrimitive, CanTruncate, HasApproximateDigits, HasDigits, PrimedFrom},
    IAdic, LazyDiv,
};
use super::ZAdic;



impl std::ops::Mul<ZAdic> for u32 {
    type Output=ZAdic;
    fn mul(self, adic_int: ZAdic) -> ZAdic {
        let p = adic_int.p();
        ZAdic::primed_from(p, self) * adic_int
    }
}



impl ops::Add for ZAdic {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {

        validate_matching_p([self.p(), rhs.p()]);
        let p = self.p();

        match std::cmp::min(self.c, rhs.c) {
            Valuation::PosInf => ZAdic::from(self.variant.clone() + rhs.variant.clone()),
            Valuation::Finite(c) => {
                let (ua, ub) = (self.variant.truncation(c), rhs.variant.truncation(c));
                ZAdic::new_approx(
                    p, c, ua.add(ub).digits().take(c).collect()
                )
            },
        }

    }
}


impl ops::Neg for ZAdic {
    type Output = Self;
    fn neg(self) -> Self::Output {

        let variant = match self.c {
            Valuation::PosInf => (-self.variant.clone()),
            Valuation::Finite(c) => {
                let u = self.variant.truncation(c);
                (-IAdic::from(u)).into()
            },
        };
        ZAdic{ c: self.c, variant }

    }
}


impl ops::Mul for ZAdic {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {

        validate_matching_p([self.p(), rhs.p()]);

        let p = self.p();
        let ac = self.certainty();
        let av = self.valuation();
        let bc = rhs.certainty();
        let bv = rhs.valuation();

        match std::cmp::min(ac + bv, bc + av) {
            Valuation::PosInf => ZAdic::from(self.variant.clone() * rhs.variant.clone()),
            Valuation::Finite(c) => {
                let (ua, ub) = (self.variant.truncation(c), rhs.variant.truncation(c));
                ZAdic::new_approx(
                    p, c, ua.mul(ub).digits().take(c).collect()
                )
            },
        }

    }
}



impl num::traits::Inv for ZAdic {
    type Output = ZAdic;
    fn inv(self) -> Self::Output {
        Self::one(self.p()) / self
    }
}

impl ops::Div for ZAdic {
    type Output = ZAdic;
    fn div(self, rhs: Self) -> Self::Output {
        if self.is_certain() && rhs.is_certain() {
            let div = LazyDiv::new(self.variant, rhs.variant);
            div.int_div_exact().expect("Error during exact ZAdic division").into()
        } else {
            let div = LazyDiv::new(self, rhs);
            div.int_div_approx_max().expect("Error during approximate ZAdic division")
        }
    }
}

impl num::CheckedDiv for ZAdic {
    fn checked_div(&self, v: &Self) -> Option<Self> {
        if v.is_local_zero() {
            return None
        }
        Some(self.clone() / v.clone())
    }
}

impl ops::Rem for ZAdic {
    type Output = ZAdic;
    fn rem(self, rhs: Self) -> Self::Output {
        if self.is_certain() && rhs.is_certain() {
            let div = LazyDiv::new(self.variant, rhs.variant);
            div.rem_exact().expect("Error during exact ZAdic remainder").into()
        } else {
            let div = LazyDiv::new(self, rhs);
            div.rem_approx_max().expect("Error during approximate ZAdic remainder")
        }
    }
}



impl_add_assign_from_add!(ZAdic);
impl_mul_assign_from_mul!(ZAdic);
impl_sub_from_neg!(ZAdic);
impl_sub_assign_from_sub!(ZAdic);
impl_pow_by_squaring!(ZAdic);
