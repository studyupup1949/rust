use std::{
    fmt,
    iter::{repeat, repeat_n},
};
use itertools::Either;
use num::{rational::Ratio, traits::Pow, Zero};
use crate::{
    divisible::{Composite, Divisible, PrimePower},
    error::AdicResult,
    local_num::{LocalOne, LocalZero},
    normed::{Normed, UltraNormed, Valuation},
    traits::{AdicPrimitive, CanApproximate, CanTruncate, HasApproximateDigits, HasDigitDisplay, HasDigits, PrimedFrom},
    ZAdic,
};
use super::{IAdic, RAdic, Sign, UAdic};


macro_rules! impl_display {
    ( $AdicInt:ty ) => {
        impl fmt::Display for $AdicInt {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let p = self.p();
                let digits = self.digit_display();
                if digits.is_empty() {
                    let zero = p.display_zero();
                    write!(f, "{zero}._{p}")
                } else {
                    write!(f, "{digits}._{p}")
                }
            }
        }
    }
}

impl_display!(IAdic);
impl_display!(RAdic);
impl_display!(UAdic);



impl LocalZero for UAdic {
    fn local_zero(&self) -> Self {
        Self::zero(self.p())
    }
    fn is_local_zero(&self) -> bool {
        self.d.iter().all(LocalZero::is_local_zero)
    }
}
impl LocalOne for UAdic {
    fn local_one(&self) -> Self {
        Self::one(self.p())
    }
    fn is_local_one(&self) -> bool {
        self.d.first().is_some_and(LocalOne::is_local_one)
            && self.d.iter().skip(1).all(LocalZero::is_local_zero)
    }
}

impl LocalZero for IAdic {
    fn local_zero(&self) -> Self {
        Self::zero(self.p())
    }
    fn is_local_zero(&self) -> bool {
        matches!(self.sign, Sign::Pos) && self.d.iter().all(LocalZero::is_local_zero)
    }
}
impl LocalOne for IAdic {
    fn local_one(&self) -> Self {
        Self::one(self.p())
    }
    fn is_local_one(&self) -> bool {
        matches!(self.sign, Sign::Pos)
            && self.d.first().is_some_and(LocalOne::is_local_one)
            && self.d.iter().skip(1).all(LocalZero::is_local_zero)
    }
}

impl LocalZero for RAdic {
    fn local_zero(&self) -> Self {
        Self::zero(self.p())
    }
    fn is_local_zero(&self) -> bool {
        self.fix_d.iter().all(LocalZero::is_local_zero) && self.rep_d.iter().all(LocalZero::is_local_zero)
    }
}
impl LocalOne for RAdic {
    fn local_one(&self) -> Self {
        Self::one(self.p())
    }
    fn is_local_one(&self) -> bool {
        self.fix_d.first().is_some_and(LocalOne::is_local_one)
            && self.fix_d.iter().skip(1).all(LocalZero::is_local_zero)
            && self.rep_d.iter().all(LocalZero::is_local_zero)
    }
}



macro_rules! impl_normed_and_ultranormed {
    ( $AdicInt:ty ) => {

        impl Normed for $AdicInt {
            type Norm = Ratio<u32>;
            type Unit = Self;
            fn norm(&self) -> Ratio<u32> {
                match self.valuation() {
                    Valuation::PosInf => Ratio::zero(),
                    Valuation::Finite(valuation) => {
                        let v = u32::try_from(valuation).expect("norm usize -> u32 conversion");
                        let inv_norm = self.p().pow(v);
                        Ratio::new(1, u32::from(inv_norm))
                    },
                }
            }
            fn unit(&self) -> Option<Self::Unit> {
                match self.valuation() {
                    Valuation::PosInf => None,
                    Valuation::Finite(valuation) => Some(self.quotient(valuation)),
                }
            }
            fn into_unit(self) -> Option<Self::Unit> {
                match self.valuation() {
                    Valuation::PosInf => None,
                    Valuation::Finite(valuation) => Some(self.into_quotient(valuation)),
                }
            }
            fn from_norm_and_unit(norm: Self::Norm, u: Self::Unit) -> Self {
                let (numer, denom) = norm.into_raw();
                if numer == 0 {
                    u.local_zero()
                } else if numer == 1 {
                    let adic_inv_norm = Self::primed_from(u.p(), denom);
                    adic_inv_norm * u
                } else {
                    panic!("Cannot invert norm to adic integer");
                }
                }
            fn from_unit(u: Self::Unit) -> Self {
                u
            }
            fn is_unit(&self) -> bool {
                self.valuation().is_zero()
            }
        }

        impl UltraNormed for $AdicInt {
            type ValuationRing = usize;
            fn from_unit_and_valuation(u: Self::Unit, v: Valuation<Self::ValuationRing>) -> Self {
                let p = u.p();
                match v {
                    Valuation::Finite(v) => {
                        let pp = PrimePower::try_from((p, v)).expect("PrimePower from usize");
                        u * Self::from_prime_power(pp)
                    },
                    Valuation::PosInf => Self::zero(p),
                }
            }
            fn valuation(&self) -> Valuation<usize> {
                if self.is_local_zero() {
                    Valuation::PosInf
                } else {
                    Valuation::Finite(
                        self.digits().take_while(Zero::is_zero).count()
                    )
                }
            }
        }

    }
}

impl_normed_and_ultranormed!(UAdic);
impl_normed_and_ultranormed!(IAdic);
impl_normed_and_ultranormed!(RAdic);


macro_rules! impl_can_approximate {
    ( $AdicInt:ty ) => {
        impl HasApproximateDigits for $AdicInt {
            fn certainty(&self) -> Valuation<Self::DigitIndex> {
                Valuation::PosInf
            }
        }
        impl CanApproximate for $AdicInt {
            type Approximation = ZAdic;
            fn approximation(&self, n: usize) -> Self::Approximation {
                let c = match self.certainty() {
                    Valuation::PosInf => n,
                    Valuation::Finite(v) => std::cmp::min(v, n),
                };
                ZAdic::new_approx(self.p(), c, self.digits().take(c).collect())
            }
            fn into_approximation(self, n: usize) -> Self::Approximation {
                let c = match self.certainty() {
                    Valuation::PosInf => n,
                    Valuation::Finite(v) => std::cmp::min(v, n),
                };
                ZAdic::new_approx(self.p(), c, self.digits().take(c).collect())
            }
        }
    }
}
impl_can_approximate!(IAdic);
impl_can_approximate!(RAdic);
impl_can_approximate!(UAdic);


impl HasDigits for IAdic {
    type DigitIndex = usize;
    fn base(&self) -> Composite {
        self.p.into()
    }
    fn min_index(&self) -> Valuation<Self::DigitIndex> {
        0.into()
    }
    fn num_digits(&self) -> Valuation<usize> {
        match self.sign {
            Sign::Pos => Valuation::Finite(self.d.len()),
            Sign::Neg => Valuation::PosInf,
        }
    }
    fn digit(&self, n: usize) -> AdicResult<u32> {
        Ok(self.d.get(n).copied().unwrap_or(match self.sign {
            Sign::Pos => 0,
            Sign::Neg => self.p.m1(),
        }))
    }
    fn digits(&self) -> impl Iterator<Item=u32> {
        // Returns infinite iterator if num_digits PosInf and finite else
        match self.sign {
            Sign::Pos => Either::Left(self.d.iter().copied()),
            Sign::Neg => Either::Right(self.d.iter().copied().chain(repeat(self.p.m1())))
        }
    }
}

impl CanTruncate for IAdic {
    type Quotient = Self;
    type Truncation = UAdic;
    fn split(&self, n: Self::DigitIndex) -> (Self::Truncation, Self::Quotient) {
        self.clone().into_split(n)
    }
    fn into_split(self, n: Self::DigitIndex) -> (Self::Truncation, Self::Quotient) {
        let non_trailing = self.num_non_trailing();
        if non_trailing > n {
            let (r, q) = UAdic::new(self.p, self.d).into_split(n);
            let digit_vec = q.digits().chain(repeat(0)).take(non_trailing - n).collect();
            (r, Self::new(self.p, digit_vec, self.sign))
        } else {
            let trail = self.trailing_digit();
            let rem = self.d.into_iter().chain(repeat(trail)).take(n).collect();
            (UAdic::new(self.p, rem), Self::new(self.p, vec![], self.sign))
        }
    }
}

impl HasDigits for RAdic {
    type DigitIndex = usize;
    fn base(&self) -> Composite {
        self.p.into()
    }
    fn min_index(&self) -> Valuation<Self::DigitIndex> {
        0.into()
    }
    fn num_digits(&self) -> Valuation<usize> {
        if self.rep_d.is_empty() {
            Valuation::Finite(self.fix_d.len())
        } else {
            Valuation::PosInf
        }
    }
    fn digit(&self, n: usize) -> AdicResult<u32> {
        if n < self.fix_d.len() {
            Ok(self.fix_d.get(n).copied().unwrap_or(0))
        } else if self.rep_d.is_empty() {
            Ok(0)
        } else {
            let diff = n - self.fix_d.len();
            let n_phase = diff % self.rep_d.len();
            Ok(self.rep_d.get(n_phase).copied().unwrap_or(0))
        }

    }
    fn digits(&self) -> impl Iterator<Item=u32> {
        self.fix_d.iter().chain(self.rep_d.iter().cycle()).copied()
    }
}

impl CanTruncate for RAdic {
    type Quotient = Self;
    type Truncation = UAdic;
    fn split(&self, n: Self::DigitIndex) -> (Self::Truncation, Self::Quotient) {
        self.clone().into_split(n)
    }
    fn into_split(self, n: Self::DigitIndex) -> (Self::Truncation, Self::Quotient) {
        let p = self.p;
        let (fix_len, rep_len) = (self.fix_d.len(), self.rep_d.len());
        if fix_len >= n {
            let (r, q) = UAdic::new(p, self.fix_d).into_split(n);
            let qn = q.finite_num_digits();
            let fix_digit_vec = q.digits().chain(repeat_n(0, fix_len - n - qn)).collect::<Vec<_>>();
            (r, Self::new(p, fix_digit_vec, self.rep_d))
        } else if rep_len == 0 {
            (UAdic::new(p, self.fix_d), Self::zero(p))
        } else {
            let rep_clone = self.rep_d.clone();
            let before = UAdic::new(self.p, self.digits().take(n).collect());
            let rem_disp = (n - fix_len) % rep_len;
            let after = Self::new(
                p, vec![],
                rep_clone.into_iter().cycle().skip(rem_disp).take(rep_len).collect()
            );
            (before, after)
        }
    }
}

impl HasDigits for UAdic {
    type DigitIndex = usize;
    fn base(&self) -> Composite {
        self.p.into()
    }
    fn min_index(&self) -> Valuation<Self::DigitIndex> {
        0.into()
    }
    fn num_digits(&self) -> Valuation<usize> {
        Valuation::Finite(self.d.len())
    }
    fn digit(&self, n: usize) -> AdicResult<u32> {
        Ok(self.d.get(n).copied().unwrap_or(0))
    }
    fn digits(&self) -> impl Iterator<Item=u32> {
        self.d.iter().copied()
    }
}

impl CanTruncate for UAdic {
    type Quotient = Self;
    type Truncation = UAdic;
    fn split(&self, n: Self::DigitIndex) -> (Self::Truncation, Self::Quotient) {
        self.clone().into_split(n)
    }
    fn into_split(self, n: Self::DigitIndex) -> (Self::Truncation, Self::Quotient) {
        let mut remainder = Vec::with_capacity(n);
        let mut quotient = Vec::with_capacity(self.finite_num_digits());
        for (i, d) in self.d.into_iter().enumerate() {
            if i < n {
                remainder.push(d);
            } else {
                quotient.push(d);
            }
        }
        (UAdic::new(self.p, remainder), Self::new(self.p, quotient))
    }
}


#[cfg(test)]
mod test {

    use crate::{traits::{AdicPrimitive, PrimedFrom}, EAdic};
    use super::UAdic;


    #[test]
    fn display() {

        // UAdic
        assert_eq!("0._5", uadic!(5, []).to_string());
        assert_eq!("1._5", UAdic::one(5).to_string());
        assert_eq!("2._5", uadic!(5, [2]).to_string());
        assert_eq!("3._5", uadic!(5, [3]).to_string());
        assert_eq!("4._5", uadic!(5, [4]).to_string());
        assert_eq!("10._5", uadic!(5, [0, 1]).to_string());
        assert_eq!("11._5", UAdic::primed_from(5, 6).to_string());
        assert_eq!("20._5", uadic!(5, [0, 2]).to_string());
        assert_eq!("44._5", uadic!(5, [4, 4]).to_string());
        assert_eq!("100._5", uadic!(5, [0, 0, 1]).to_string());
        assert_eq!("22._5", uadic!(5, [2, 2, 0, 0]).to_string());
        assert_eq!("1111._5", uadic!(5, [1, 1, 1, 1]).to_string());
        assert_eq!("4444._5", uadic!(5, [4, 4, 4, 4]).to_string());
        assert_eq!("1000._5", uadic!(5, [0, 0, 0, 1, 0, 0]).to_string());
        assert_eq!("1001._5", uadic!(5, [1, 0, 0, 1]).to_string());
        assert_eq!("uka1._31", uadic!(31, [1, 10, 20, 30]).to_string());
        assert_eq!("[30][20][10][1]._37", uadic!(37, [1, 10, 20, 30]).to_string());

        // IAdic
        assert_eq!("0._5", eadic!(5, []).to_string());
        assert_eq!("1._5", EAdic::one(5).to_string());
        assert_eq!("2._5", eadic!(5, [2]).to_string());
        assert_eq!("3._5", eadic!(5, [3]).to_string());
        assert_eq!("4._5", eadic!(5, [4]).to_string());
        assert_eq!("10._5", eadic!(5, [0, 1]).to_string());
        assert_eq!("11._5", EAdic::primed_from(5, 6).to_string());
        assert_eq!("20._5", eadic!(5, [0, 2]).to_string());
        assert_eq!("44._5", eadic!(5, [4, 4]).to_string());
        assert_eq!("100._5", eadic!(5, [0, 0, 1]).to_string());
        assert_eq!("22._5", eadic!(5, [2, 2, 0, 0]).to_string());
        assert_eq!("(4)._5", eadic_neg!(5, []).to_string());
        assert_eq!("(4)3._5", eadic_neg!(5, [3]).to_string());
        assert_eq!("(4)2._5", (-eadic!(5, [3])).to_string());
        assert_eq!("(4)0._5", eadic_neg!(5, [0]).to_string());
        assert_eq!("(4)34._5", EAdic::primed_from(5, -6).to_string());
        assert_eq!("(4)30._5", eadic_neg!(5, [0, 3]).to_string());
        assert_eq!("(4)00._5", eadic_neg!(5, [0, 0]).to_string());
        assert_eq!("(u)ka1._31", eadic_neg!(31, [1, 10, 20]).to_string());
        assert_eq!("([36])[20][10][1]._37", eadic_neg!(37, [1, 10, 20]).to_string());

        // RAdic
        assert_eq!("0._5", eadic_rep!(5, [], []).to_string());
        assert_eq!("1._5", EAdic::one(5).to_string());
        assert_eq!("2._5", eadic_rep!(5, [2], []).to_string());
        assert_eq!("3._5", eadic_rep!(5, [3], []).to_string());
        assert_eq!("4._5", eadic_rep!(5, [4], []).to_string());
        assert_eq!("10._5", eadic_rep!(5, [0, 1], []).to_string());
        assert_eq!("11._5", EAdic::primed_from(5, 6).to_string());
        assert_eq!("20._5", eadic_rep!(5, [0, 2], []).to_string());
        assert_eq!("22._5", eadic_rep!(5, [2, 2], []).to_string());
        assert_eq!("100._5", eadic_rep!(5, [0, 0, 1], []).to_string());
        assert_eq!("(4)._5", eadic_rep!(5, [], [4]).to_string());
        assert_eq!("(4)3._5", eadic_rep!(5, [3], [4]).to_string());
        assert_eq!("(4)2._5", (-eadic_rep!(5, [3], [])).to_string());
        assert_eq!("(4)1._5", eadic_rep!(5, [1], [4]).to_string());
        assert_eq!("(4)0._5", eadic_rep!(5, [0], [4]).to_string());
        assert_eq!("(4)30._5", eadic_rep!(5, [0, 3], [4]).to_string());
        assert_eq!("(1)._5", eadic_rep!(5, [], [1]).to_string());
        assert_eq!("(3)4._5", (-eadic_rep!(5, [], [1])).to_string());
        assert_eq!("(1)0._5", eadic_rep!(5, [0], [1]).to_string());
        assert_eq!("(1)32._5", eadic_rep!(5, [2, 3, 1, 1], [1]).to_string());
        assert_eq!("(01)._5", eadic_rep!(5, [], [1, 0]).to_string());
        assert_eq!("(10)._5", (eadic_rep!(5, [0, 1], []) * eadic_rep!(5, [], [1, 0])).to_string());
        assert_eq!("(004)._5", eadic_rep!(5, [], [4, 0, 0, 4, 0, 0]).to_string());
        assert_eq!("(04)._5", eadic_rep!(5, [4, 0, 4], [0, 4]).to_string());
        assert_eq!("(uk)a1._31", eadic_rep!(31, [1, 10], [20, 30]).to_string());
        assert_eq!("([30][20])[10][1]._37", eadic_rep!(37, [1, 10], [20, 30]).to_string());

    }

}
