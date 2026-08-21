use std::{fmt, iter::repeat};
use num::pow::Pow;
use crate::{adic_valid, AdicInteger, LazyQDiv, QAdicValuation};
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
            (QAdicValuation::PosInf, _) => rhs.clone(),
            (_, QAdicValuation::PosInf) => self.clone(),
            (QAdicValuation::Finite(sv), QAdicValuation::Finite(rv)) => {
                if sv < rv {
                    let dv = (rv - sv).unsigned_abs();
                    let matched = rhs.adic_unit.clone() * A::p_power(p, dv);
                    let added = self.adic_unit.clone() + matched;
                    QAdic::new(added, QAdicValuation::Finite(sv))
                } else {
                    let dv = (sv - rv).unsigned_abs();
                    let matched = self.adic_unit.clone() * A::p_power(p, dv);
                    let added = matched + rhs.adic_unit.clone();
                    QAdic::new(added, QAdicValuation::Finite(rv))
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
            (QAdicValuation::PosInf, _) | (_, QAdicValuation::PosInf) => Self::zero(p),
            (QAdicValuation::Finite(sv), QAdicValuation::Finite(rv)) => {
                let mult = self.adic_unit * rhs.adic_unit;
                let val = QAdicValuation::Finite(sv + rv);
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
        self * QAdic::new(A::from_u32(p, coeff), QAdicValuation::Finite(0))
    }
}

impl<A> std::ops::Mul<u32> for &QAdic<A>
where A: AdicInteger {
    type Output=QAdic<A>;
    fn mul(self, coeff: u32) -> QAdic<A> {
        let p = self.p();
        self * QAdic::new(A::from_u32(p, coeff), QAdicValuation::Finite(0))
    }
}

impl<A> std::ops::Mul<QAdic<A>> for u32
where A: AdicInteger {
    type Output=QAdic<A>;
    fn mul(self, adic_int: QAdic<A>) -> QAdic<A> {
        let p = adic_int.p();
        QAdic::new(A::from_u32(p, self), QAdicValuation::Finite(0)) * adic_int
    }
}

impl<A> std::ops::Mul<&QAdic<A>> for u32
where A: AdicInteger {
    type Output=QAdic<A>;
    fn mul(self, adic_int: &QAdic<A>) -> QAdic<A> {
        let p = adic_int.p();
        QAdic::new(A::from_u32(p, self), QAdicValuation::Finite(0)) * adic_int
    }
}


impl<A> Pow<u32> for &QAdic<A>
where A: AdicInteger {
    type Output = QAdic<A>;
    fn pow(self, power: u32) -> Self::Output {
        repeat(
            self.clone()
        ).take(
            usize::try_from(power).expect("pow u32 -> usize conversion")
        ).reduce(
            |acc, e| acc * e
        ).unwrap_or(
            QAdic::one(self.p())
        )
    }
}


impl<A> std::ops::Div<QAdic<A>> for QAdic<A>
where A: AdicInteger {
    type Output = LazyQDiv<A>;
    fn div(self, rhs: QAdic<A>) -> Self::Output {
        LazyQDiv::new(self, rhs)
    }
}

impl<A> std::ops::Div<&QAdic<A>> for QAdic<A>
where A: AdicInteger {
    type Output = LazyQDiv<A>;
    fn div(self, rhs: &QAdic<A>) -> Self::Output {
        LazyQDiv::new(self, rhs.clone())
    }
}

impl<A> std::ops::Div<QAdic<A>> for &QAdic<A>
where A: AdicInteger {
    type Output = LazyQDiv<A>;
    fn div(self, rhs: QAdic<A>) -> Self::Output {
        LazyQDiv::new(self.clone(), rhs)
    }
}

impl<A> std::ops::Div<&QAdic<A>> for &QAdic<A>
where A: AdicInteger {
    type Output = LazyQDiv<A>;
    fn div(self, rhs: &QAdic<A>) -> Self::Output {
        LazyQDiv::new(self.clone(), rhs.clone())
    }
}


impl<A> fmt::Display for QAdic<A>
where A: AdicInteger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let p = self.p();
        let (int, frac) = self.int_and_rem();
        let int_str = int.digit_str();
        let frac_str = frac.digit_str();
        write!(f, "{int_str}.{frac_str}_{p}")
    }
}
