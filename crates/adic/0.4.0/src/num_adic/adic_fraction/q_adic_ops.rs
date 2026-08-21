use num::{pow::Pow, traits::Inv, Integer};
use crate::{
    adic_valid, AdicInteger, AdicNumber, AdicSized, AdicValuation, LazyDiv,
};
use super::QAdic;



impl<A> std::ops::Add for &QAdic<A>
where A: AdicInteger {
    type Output = QAdic<A>;
    fn add(self, rhs: Self) -> Self::Output {

        // Adjust the bigger adic unit to match smaller
        // Add the adic integers together
        // Create new QAdic

        adic_valid::validate_mono_character(self.p(), rhs.p());
        let p = self.p();

        match (self.valuation(), rhs.valuation()) {
            (AdicValuation::PosInf, _) => rhs.clone(),
            (_, AdicValuation::PosInf) => self.clone(),
            (AdicValuation::Finite(sv), AdicValuation::Finite(rv)) => {
                if sv < rv {
                    let dv = (rv - sv).unsigned_abs();
                    let matched = rhs.adic_unit.clone() * A::p_power(p, dv);
                    let added = self.adic_unit.clone() + matched;
                    QAdic::new(added, AdicValuation::Finite(sv))
                } else {
                    let dv = (sv - rv).unsigned_abs();
                    let matched = self.adic_unit.clone() * A::p_power(p, dv);
                    let added = matched + rhs.adic_unit.clone();
                    QAdic::new(added, AdicValuation::Finite(rv))
                }
            }
        }

    }
}

impl<A> std::ops::Add<&QAdic<A>> for QAdic<A>
where A: AdicInteger {
    type Output = QAdic<A>;
    fn add(self, rhs: &QAdic<A>) -> Self::Output {
        (&self).add(rhs)
    }
}

impl<A> std::ops::Add<QAdic<A>> for &QAdic<A>
where A: AdicInteger {
    type Output = QAdic<A>;
    fn add(self, rhs: QAdic<A>) -> Self::Output {
        self.add(&rhs)
    }
}

impl<A> std::ops::Add for QAdic<A>
where A: AdicInteger {
    type Output = QAdic<A>;
    fn add(self, rhs: QAdic<A>) -> Self::Output {
        (&self).add(&rhs)
    }
}


impl<A> std::ops::Neg for &QAdic<A>
where A: AdicInteger + std::ops::Neg<Output=A> {
    type Output = QAdic<A>;
    fn neg(self) -> Self::Output {
        QAdic::new(-self.adic_unit.clone(), self.valuation)
    }
}

impl<A> std::ops::Neg for QAdic<A>
where A: AdicInteger + std::ops::Neg<Output=A> {
    type Output = QAdic<A>;
    fn neg(self) -> Self::Output {
        Self::new(-self.adic_unit, self.valuation)
    }
}


impl<A> std::ops::Sub for QAdic<A>
where A: AdicInteger + std::ops::Neg<Output=A> {
    type Output = QAdic<A>;
    fn sub(self, rhs: QAdic<A>) -> Self::Output {
        self + (-rhs)
    }
}

impl<A> std::ops::Sub<&QAdic<A>> for QAdic<A>
where A: AdicInteger + std::ops::Neg<Output=A> {
    type Output = QAdic<A>;
    fn sub(self, rhs: &QAdic<A>) -> Self::Output {
        self + (-rhs)
    }
}

impl<A> std::ops::Sub<QAdic<A>> for &QAdic<A>
where A: AdicInteger + std::ops::Neg<Output=A> {
    type Output = QAdic<A>;
    fn sub(self, rhs: QAdic<A>) -> Self::Output {
        self + (-rhs)
    }
}

impl<A> std::ops::Sub for &QAdic<A>
where A: AdicInteger + std::ops::Neg<Output=A> {
    type Output = QAdic<A>;
    fn sub(self, rhs: &QAdic<A>) -> Self::Output {
        self + (-rhs)
    }
}


impl<A> std::ops::Mul for QAdic<A>
where A: AdicInteger {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {

        // Multiply the units, add the valuations
        // Create new QAdic

        adic_valid::validate_mono_character(self.p(), rhs.p());
        let p = self.p();

        match (self.valuation(), rhs.valuation()) {
            (AdicValuation::PosInf, _) | (_, AdicValuation::PosInf) => Self::zero(p),
            (AdicValuation::Finite(sv), AdicValuation::Finite(rv)) => {
                let mult = self.adic_unit * rhs.adic_unit;
                let val = AdicValuation::Finite(sv + rv);
                QAdic::new(mult, val)
            }
        }

    }
}

impl<A> std::ops::Mul<&QAdic<A>> for QAdic<A>
where A: AdicInteger {
    type Output = QAdic<A>;
    fn mul(self, rhs: &QAdic<A>) -> Self::Output {
        self * rhs.clone()
    }
}

impl<A> std::ops::Mul<QAdic<A>> for &QAdic<A>
where A: AdicInteger {
    type Output = QAdic<A>;
    fn mul(self, rhs: QAdic<A>) -> Self::Output {
        self.clone() * rhs
    }
}

impl<A> std::ops::Mul for &QAdic<A>
where A: AdicInteger {
    type Output = QAdic<A>;
    fn mul(self, rhs: &QAdic<A>) -> Self::Output {
        self.clone() * rhs.clone()
    }
}


impl<A> std::ops::Mul<u32> for QAdic<A>
where A: AdicInteger {
    type Output=QAdic<A>;
    fn mul(self, coeff: u32) -> QAdic<A> {
        let p = self.p();
        self * QAdic::new(A::from_u32(p, coeff), AdicValuation::Finite(0))
    }
}

impl<A> std::ops::Mul<u32> for &QAdic<A>
where A: AdicInteger {
    type Output=QAdic<A>;
    fn mul(self, coeff: u32) -> QAdic<A> {
        let p = self.p();
        self * QAdic::new(A::from_u32(p, coeff), AdicValuation::Finite(0))
    }
}

impl<A> std::ops::Mul<QAdic<A>> for u32
where A: AdicInteger {
    type Output=QAdic<A>;
    fn mul(self, adic_int: QAdic<A>) -> QAdic<A> {
        let p = adic_int.p();
        QAdic::new(A::from_u32(p, self), AdicValuation::Finite(0)) * adic_int
    }
}

impl<A> std::ops::Mul<&QAdic<A>> for u32
where A: AdicInteger {
    type Output=QAdic<A>;
    fn mul(self, adic_int: &QAdic<A>) -> QAdic<A> {
        let p = adic_int.p();
        QAdic::new(A::from_u32(p, self), AdicValuation::Finite(0)) * adic_int
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

impl<A> Pow<u32> for &QAdic<A>
where A: AdicInteger {
    type Output = QAdic<A>;
    fn pow(self, power: u32) -> Self::Output {
        self.clone().pow(power)
    }
}


impl<A> Inv for QAdic<A>
where A: AdicInteger {
    type Output = LazyDiv<QAdic<A>>;
    fn inv(self) -> Self::Output {
        LazyDiv::new(QAdic::one(self.p()), self)
    }
}


impl<A> std::ops::Div<QAdic<A>> for QAdic<A>
where A: AdicInteger {
    type Output = LazyDiv<QAdic<A>>;
    fn div(self, rhs: QAdic<A>) -> Self::Output {
        LazyDiv::new(self, rhs)
    }
}

impl<A> std::ops::Div<&QAdic<A>> for QAdic<A>
where A: AdicInteger {
    type Output = LazyDiv<QAdic<A>>;
    fn div(self, rhs: &QAdic<A>) -> Self::Output {
        LazyDiv::new(self, rhs.clone())
    }
}

impl<A> std::ops::Div<QAdic<A>> for &QAdic<A>
where A: AdicInteger {
    type Output = LazyDiv<QAdic<A>>;
    fn div(self, rhs: QAdic<A>) -> Self::Output {
        LazyDiv::new(self.clone(), rhs)
    }
}

impl<A> std::ops::Div<&QAdic<A>> for &QAdic<A>
where A: AdicInteger {
    type Output = LazyDiv<QAdic<A>>;
    fn div(self, rhs: &QAdic<A>) -> Self::Output {
        LazyDiv::new(self.clone(), rhs.clone())
    }
}
