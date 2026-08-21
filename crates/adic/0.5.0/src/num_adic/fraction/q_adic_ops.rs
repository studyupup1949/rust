use num::{
    pow::Pow,
    traits::Inv,
    Integer,
};
use crate::{
    divisible::PrimePower,
    error::validate_matching_p,
    local_num::LocalZero,
    normed::{UltraNormed, Valuation},
    traits::{AdicInteger, AdicPrimitive, HasApproximateDigits, PrimedFrom},
    EAdic, LazyDiv, ZAdic,
};
use super::QAdic;



impl<A> std::ops::Add for QAdic<A>
where A: AdicInteger {
    type Output = QAdic<A>;
    fn add(self, rhs: Self) -> Self::Output {

        // Adjust the bigger adic unit to match smaller
        // Add the adic integers together
        // Create new QAdic

        validate_matching_p([self.p(), rhs.p()]);
        let p = self.p();

        match (self.valuation(), rhs.valuation()) {
            (Valuation::PosInf, _) => rhs,
            (_, Valuation::PosInf) => self,
            (Valuation::Finite(sv), Valuation::Finite(rv)) => {
                if sv < rv {
                    let dv = (rv - sv).unsigned_abs();
                    let pp = PrimePower::try_from((p, dv)).expect("PrimePower from usize");
                    let matched = rhs.internal_int * A::from_prime_power(pp);
                    let added = self.internal_int + matched;
                    QAdic::new(added, Valuation::Finite(sv))
                } else {
                    let dv = (sv - rv).unsigned_abs();
                    let pp = PrimePower::try_from((p, dv)).expect("PrimePower from usize");
                    let matched = self.internal_int * A::from_prime_power(pp);
                    let added = matched + rhs.internal_int;
                    QAdic::new(added, Valuation::Finite(rv))
                }
            }
        }

    }
}

impl<A> std::ops::AddAssign for QAdic<A>
where A: AdicInteger {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.clone() + rhs;
    }
}


impl<A> std::ops::Neg for QAdic<A>
where A: AdicInteger + std::ops::Neg<Output=A> {
    type Output = QAdic<A>;
    fn neg(self) -> Self::Output {
        Self::new(-self.internal_int, self.valuation)
    }
}


impl<A> std::ops::Sub for QAdic<A>
where A: AdicInteger + std::ops::Neg<Output=A> {
    type Output = QAdic<A>;
    fn sub(self, rhs: QAdic<A>) -> Self::Output {
        self + (-rhs)
    }
}

impl<A> std::ops::SubAssign for QAdic<A>
where A: AdicInteger + std::ops::Neg<Output=A> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.clone() - rhs;
    }
}


impl<A> std::ops::Mul for QAdic<A>
where A: AdicInteger {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {

        // Multiply the units, add the valuations
        // Create new QAdic

        validate_matching_p([self.p(), rhs.p()]);
        let p = self.p();

        match (self.valuation(), rhs.valuation()) {
            (Valuation::PosInf, _) | (_, Valuation::PosInf) => Self::zero(p),
            (Valuation::Finite(sv), Valuation::Finite(rv)) => {
                let mult = self.internal_int * rhs.internal_int;
                let val = Valuation::Finite(sv + rv);
                QAdic::new(mult, val)
            }
        }

    }
}

impl<A> std::ops::MulAssign for QAdic<A>
where A: AdicInteger {
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.clone() * rhs;
    }
}


impl<A> std::ops::Mul<QAdic<A>> for u32
where A: AdicInteger {
    type Output=QAdic<A>;
    fn mul(self, adic_int: QAdic<A>) -> QAdic<A> {
        let p = adic_int.p();
        QAdic::new(A::primed_from(p, self), Valuation::Finite(0)) * adic_int
    }
}


impl<A> Pow<u32> for QAdic<A>
where A: AdicInteger {
    type Output = QAdic<A>;
    fn pow(self, mut power: u32) -> Self::Output {

        // Exponentiation by squaring
        let mut out = QAdic::one(self.p());
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


impl Inv for QAdic<EAdic> {
    type Output = Self;
    fn inv(self) -> Self::Output {
        QAdic::one(self.p()) / self
    }
}

impl Inv for QAdic<ZAdic> {
    type Output = Self;
    fn inv(self) -> Self::Output {
        QAdic::one(self.p()) / self
    }
}


impl std::ops::Div for QAdic<EAdic> {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        LazyDiv::new(self, rhs).div_exact().expect("Error during QAdic<EAdic> division")
    }
}

impl std::ops::Div for QAdic<ZAdic> {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        if self.is_certain() && rhs.is_certain() {
            let a = QAdic::<EAdic>::new(self.internal_int.exact_variant(), self.valuation);
            let b = QAdic::<EAdic>::new(rhs.internal_int.exact_variant(), rhs.valuation);
            let div = LazyDiv::new(a, b);
            let div = div.div_exact().expect("Error during exact QAdic<ZAdic> division");
            QAdic::from(div)
        } else {
            let div = LazyDiv::new(self, rhs);
            div.div_approx_max().expect("Error during approximate QAdic<ZAdic> division")
        }
    }
}

impl num::CheckedDiv for QAdic<EAdic> {
    fn checked_div(&self, v: &Self) -> Option<Self> {
        if v.is_local_zero() {
            return None
        }
        Some(self.clone() / v.clone())
    }
}

impl num::CheckedDiv for QAdic<ZAdic> {
    fn checked_div(&self, v: &Self) -> Option<Self> {
        if v.is_local_zero() {
            return None
        }
        Some(self.clone() / v.clone())
    }
}
