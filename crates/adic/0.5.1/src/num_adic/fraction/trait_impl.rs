use std::fmt;
use itertools::repeat_n;
use num::{rational::Ratio, traits::Pow, Zero};
use crate::{
    divisible::{Composite, PrimePower},
    error::AdicResult,
    local_num::{LocalOne, LocalZero},
    normed::{Normed, UltraNormed, Valuation},
    traits::{AdicInteger, AdicPrimitive, CanApproximate, CanTruncate, HasApproximateDigits, HasDigitDisplay, HasDigits, PrimedFrom},
    UAdic, ZAdic,
};
use super::QAdic;


impl<A> fmt::Display for QAdic<A>
where A: AdicInteger + HasDigitDisplay<DigitDisplay = String> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        let p = self.p();
        let (frac_str, int_str) = self.digit_display();

        if int_str.is_empty() {
            write!(f, "0.{frac_str}_{p}")
        } else {
            write!(f, "{int_str}.{frac_str}_{p}")
        }

    }
}


impl<A> LocalZero for QAdic<A>
where A: AdicInteger {
    fn local_zero(&self) -> Self {
        Self::zero(self.p())
    }
    fn is_local_zero(&self) -> bool {
        !self.valuation().is_finite()
    }
}

impl<A> LocalOne for QAdic<A>
where A: AdicInteger {
    fn local_one(&self) -> Self {
        Self::one(self.p())
    }
    fn is_local_one(&self) -> bool {
        self.valuation().finite().is_some_and(|v| v == 0) && self.internal_int.is_local_one()
    }
}


impl<A> Normed for QAdic<A>
where A: AdicInteger {
    type Norm = Ratio<u32>;
    type Unit = A;
    fn norm(&self) -> Ratio<u32> {
        match self.valuation() {
            Valuation::PosInf => Ratio::zero(),
            Valuation::Finite(valuation) => {
                let v = i32::try_from(valuation).expect("norm isize -> i32 conversion");
                let inv_abs_norm = self.p().pow(v.unsigned_abs());
                if v >= 0 {
                    Ratio::new(1, u32::from(inv_abs_norm))
                } else {
                    Ratio::new(u32::from(inv_abs_norm), 1)
                }
            },
        }
    }
    fn unit(&self) -> Option<Self::Unit> {
        self.internal_int.unit()
    }
    fn into_unit(self) -> Option<Self::Unit> {
        self.internal_int.into_unit()
    }
    fn from_norm_and_unit(norm: Self::Norm, u: Self::Unit) -> Self {
        let (numer, denom) = norm.into_raw();
        if numer == 0 {
            Self::from_unit(u.local_zero())
        } else if numer == 1 {
            let adic_inv_norm = Self::primed_from(u.p(), denom);
            adic_inv_norm * Self::from_unit(u)
        } else if denom == 1 {
            let adic_norm = Self::primed_from(u.p(), numer);
            let valuation = (-adic_norm.valuation()).expect("Nonzero norm has no valuation");
            Self::from_unit_and_valuation(u, valuation)
        } else {
            panic!("Cannot invert norm to adic integer");
        }
    }
    fn from_unit(u: Self::Unit) -> Self {
        Self::new(u, 0)
    }
    fn is_unit(&self) -> bool {
        self.valuation().is_zero()
    }
}

impl<A> UltraNormed for QAdic<A>
where A: AdicInteger {
    type ValuationRing = isize;
    fn from_unit_and_valuation(u: Self::Unit, v: Valuation<Self::ValuationRing>) -> Self {
        Self::new(u, v)
    }
    fn valuation(&self) -> Valuation<isize> {
        self.valuation
    }
}

impl<A> HasApproximateDigits for QAdic<A>
where A: AdicInteger {

    fn certainty(&self) -> Valuation<isize> {
        match (self.internal_int.certainty(), self.valuation()) {
            (Valuation::Finite(c), Valuation::Finite(v)) => {
                let cisize = isize::try_from(c).expect("certainty usize -> isize conversion");
                Valuation::Finite(cisize + v)
            },
            _ => Valuation::PosInf
        }
    }

}

impl<A> CanApproximate for QAdic<A>
where A: AdicInteger {

    type Approximation = QAdic<ZAdic>;

    fn approximation(&self, n: isize) -> Self::Approximation {
        let p = self.p();
        let c = match self.certainty() {
            Valuation::PosInf => n,
            Valuation::Finite(v) => std::cmp::min(v, n),
        };
        match self.unit_and_valuation() {
            (Some(unit), Valuation::Finite(v)) if n > v => {
                QAdic::new(unit.into_approximation((n-v).unsigned_abs()), v)
            },
            (None, Valuation::Finite(v)) if n > v => {
                QAdic::new(ZAdic::zero(p).into_approximation((n-v).unsigned_abs()), v)
            },
            _ => QAdic::new(ZAdic::empty(p), c),
        }
    }

    fn into_approximation(self, n: isize) -> Self::Approximation
    where A: HasApproximateDigits {
        let p = self.p();
        let c = match self.certainty() {
            Valuation::PosInf => n,
            Valuation::Finite(v) => std::cmp::min(v, n),
        };
        match self.into_unit_and_valuation() {
            (Some(unit), Valuation::Finite(v)) if n > v => {
                QAdic::new(unit.into_approximation((n-v).unsigned_abs()), v)
            },
            (None, Valuation::Finite(v)) if n > v => {
                QAdic::new(ZAdic::zero(p).into_approximation((n-v).unsigned_abs()), v)
            },
            _ => QAdic::new(ZAdic::empty(p), c),
        }
    }

}


impl<A> HasDigits for QAdic<A>
where A: AdicInteger {

    type DigitIndex = isize;

    fn base(&self) -> Composite {
        self.internal_int.base()
    }

    fn min_index(&self) -> Valuation<Self::DigitIndex> {
        match self.valuation() {
            Valuation::Finite(v) if v < 0 => v.into(),
            _ => 0.into(),
        }
    }

    fn num_digits(&self) -> Valuation<usize> {
        match self.valuation() {
            Valuation::PosInf => 0.into(),
            Valuation::Finite(v) if v >= 0 => Valuation::from(v.unsigned_abs()) + self.internal_int.num_digits(),
            Valuation::Finite(_) => self.internal_int.num_digits(),
        }
    }

    fn digit(&self, n: isize) -> AdicResult<u32> {
        match self.valuation() {
            Valuation::PosInf => Ok(0),
            Valuation::Finite(v) => {
                if n < v {
                    Ok(0)
                } else {
                    self.internal_int.digit((n-v).unsigned_abs())
                }
            }
        }
    }

    fn digits(&self) -> impl Iterator<Item=u32> {
        let num_zeros = match self.valuation() {
            Valuation::Finite(v) if v >= 0 => v.unsigned_abs(),
            _ => 0,
        };
        repeat_n(0, num_zeros).chain(self.internal_int.digits())
    }

}

impl<A> HasDigitDisplay for QAdic<A>
where A: AdicInteger + HasDigitDisplay<DigitDisplay = String> {
    type DigitDisplay = (String, String);
    fn digit_display(&self) -> Self::DigitDisplay {

        let (frac, int) = self.frac_and_int();

        let int_str = int.digit_display();

        let frac_str = match (frac.valuation(), frac.unit(), frac.unit().map(|u| u.num_digits())) {
            (Valuation::Finite(v), Some(frac_unit), Some(Valuation::Finite(n))) if v < 0 => {
                let num_zeros = v.unsigned_abs() - n;
                [str::repeat("0", num_zeros), frac_unit.digit_display()].concat()
            },
            _ => String::new()
        };

        (frac_str, int_str)

    }
}



impl<A> CanTruncate for QAdic<A>
where A: AdicInteger {
    type Quotient = A;
    type Truncation = QAdic<UAdic>;
    fn split(&self, n: Self::DigitIndex) -> (Self::Truncation, Self::Quotient) {
        let p = self.p();
        match self.valuation() {
            Valuation::Finite(v) if n > v => {
                let (rem, quot) = self.internal_int.split((n-v).unsigned_abs());
                (QAdic::new(rem, v), quot)
            },
            Valuation::Finite(v) if n <= v => {
                let pp = PrimePower::try_from((p, (v-n).unsigned_abs())).expect("PrimePower from usize");
                (QAdic::zero(p), A::from_prime_power(pp) * self.internal_int.clone())
            },
            _ => (QAdic::zero(p), A::zero(p))
        }
    }
    fn into_split(self, n: Self::DigitIndex) -> (Self::Truncation, Self::Quotient) {
        let p = self.p();
        match self.valuation() {
            Valuation::Finite(v) if n > v => {
                let (rem, quot) = self.internal_int.into_split((n-v).unsigned_abs());
                (QAdic::new(rem, v), quot)
            },
            Valuation::Finite(v) if n <= v => {
                let pp = PrimePower::try_from((p, (v-n).unsigned_abs())).expect("PrimePower from usize");
                (QAdic::zero(p), A::from_prime_power(pp) * self.internal_int)
            },
            _ => (QAdic::zero(p), A::zero(p))
        }
    }
}
