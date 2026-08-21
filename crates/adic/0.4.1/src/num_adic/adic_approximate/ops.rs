use std::{
    ops,
};
use num::{
    traits::Pow,
    Integer,
};
use overload::overload;
use crate::{
    adic_valid,
    AdicApproximate, AdicNumber, AdicSized, AdicValuation,
    ExactIntegerVariant, HasDigits, IAdic, LazyDiv, UAdic, RAdic,
};
use super::ZAdic;



impl std::ops::Mul<u32> for ZAdic {
    type Output=ZAdic;
    fn mul(self, coeff: u32) -> ZAdic {
        let p = self.p();
        self * ZAdic::from_u32(p, coeff)
    }
}

impl std::ops::Mul<u32> for &ZAdic {
    type Output=ZAdic;
    fn mul(self, coeff: u32) -> ZAdic {
        let p = self.p();
        self * ZAdic::from_u32(p, coeff)
    }
}

impl std::ops::Mul<ZAdic> for u32 {
    type Output=ZAdic;
    fn mul(self, adic_int: ZAdic) -> ZAdic {
        let p = adic_int.p();
         ZAdic::from_u32(p, self) * adic_int
    }
}

impl std::ops::Mul<&ZAdic> for u32 {
    type Output=ZAdic;
    fn mul(self, adic_int: &ZAdic) -> ZAdic {
        let p = adic_int.p();
         ZAdic::from_u32(p, self) * adic_int
    }
}

impl num::traits::Inv for ZAdic {
    type Output = LazyDiv<ZAdic>;
    fn inv(self) -> Self::Output {
        LazyDiv::new(Self::one(self.p()), self)
    }
}

impl std::ops::Div for ZAdic {
    type Output = LazyDiv<ZAdic>;
    fn div(self, rhs: ZAdic) -> Self::Output {
        LazyDiv::new(self, rhs)
    }
}

impl std::ops::Div<&ZAdic> for ZAdic {
    type Output = LazyDiv<ZAdic>;
    fn div(self, rhs: &ZAdic) -> Self::Output {
        LazyDiv::new(self, rhs.clone())
    }
}

impl std::ops::Div<ZAdic> for &ZAdic {
    type Output = LazyDiv<ZAdic>;
    fn div(self, rhs: ZAdic) -> Self::Output {
        LazyDiv::new(self.clone(), rhs)
    }
}

impl std::ops::Div<&ZAdic> for &ZAdic {
    type Output = LazyDiv<ZAdic>;
    fn div(self, rhs: &ZAdic) -> Self::Output {
        LazyDiv::new(self.clone(), rhs.clone())
    }
}

impl Pow<u32> for ZAdic {
    type Output = ZAdic;
    fn pow(self, mut power: u32) -> Self::Output {

        // Exponentiation by squaring
        let mut out = ZAdic::one(self.p());
        if power == 0 {
            return out;
        }

        let mut mult = self;
        while power > 1 {
            if power.is_odd() {
                out = out * mult.clone();
                power = power - 1;
            }
            mult = mult.clone() * mult;
            power = power / 2;
        }
        out * mult

    }
}

impl Pow<u32> for &ZAdic {
    type Output = ZAdic;
    fn pow(self, power: u32) -> Self::Output {
        self.clone().pow(power)
    }
}



// ZAdic

overload!((a: ?ZAdic) + (b: ?ZAdic) -> ZAdic {

    adic_valid::validate_mono_character(a.p(), b.p());
    let p = a.p();

    match std::cmp::min(a.c, b.c) {
        AdicValuation::PosInf => {
            match ExactDoubleVariant::new(a.variant.clone(), b.variant.clone()) {
                ExactDoubleVariant::Unsigned(ua, ub) => ZAdic::from(ua + ub),
                ExactDoubleVariant::Signed(ia, ib) => ZAdic::from(ia + ib),
                ExactDoubleVariant::Rational(ra, rb) => ZAdic::from(ra + rb),
            }
        },
        AdicValuation::Finite(c) => {
            let (ua, ub) = (a.variant.truncation(c), b.variant.truncation(c));
            ZAdic::new_approx(
                p, c, ua.add(ub).into_digits().take(c).collect()
            )
        },
    }

});

overload!( - (a: ?ZAdic) -> ZAdic {

    let variant = match a.c {
        AdicValuation::PosInf => match &a.variant {
            ExactIntegerVariant::Unsigned(u) => ExactIntegerVariant::Signed(-IAdic::from(u.clone())),
            ExactIntegerVariant::Signed(i) => ExactIntegerVariant::Signed(-i),
            ExactIntegerVariant::Rational(r) => ExactIntegerVariant::Rational(-r),
        },
        AdicValuation::Finite(c) => {
            let u = a.variant.truncation(c);
            ExactIntegerVariant::Signed(-IAdic::from(u))
        }
    };
    ZAdic{ c: a.c, variant }

});

overload!((a: ?ZAdic) - (b: ?ZAdic) -> ZAdic {
    a + (-b)
});

overload!((a: ?ZAdic) * (b: ?ZAdic) -> ZAdic {

    adic_valid::validate_mono_character(a.p(), b.p());

    let p = a.p();
    let ac = a.certainty();
    let av = a.valuation();
    let bc = b.certainty();
    let bv = b.valuation();

    match std::cmp::min(ac + bv, bc + av) {
        AdicValuation::PosInf => {
            match ExactDoubleVariant::new(a.variant.clone(), b.variant.clone()) {
                ExactDoubleVariant::Unsigned(ua, ub) => ZAdic::from(ua * ub),
                ExactDoubleVariant::Signed(ia, ib) => ZAdic::from(ia * ib),
                ExactDoubleVariant::Rational(ra, rb) => ZAdic::from(ra * rb),
            }
        },
        AdicValuation::Finite(c) => {
            let (ua, ub) = (a.variant.truncation(c), b.variant.truncation(c));
            ZAdic::new_approx(
                p, c, ua.mul(ub).into_digits().take(c).collect()
            )
        },
    }

});



enum ExactDoubleVariant {
    Unsigned(UAdic, UAdic),
    Signed(IAdic, IAdic),
    Rational(RAdic, RAdic),
}
impl ExactDoubleVariant {
    fn new(a: ExactIntegerVariant, b: ExactIntegerVariant) -> Self {
        use ExactIntegerVariant::{Unsigned, Signed, Rational};
        match (a, b) {
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
