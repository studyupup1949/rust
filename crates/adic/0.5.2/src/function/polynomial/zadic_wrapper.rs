use num::{rational::Ratio, Zero};
use crate::{
    error::AdicError,
    local_num::{LocalInteger, LocalNum, LocalOne, LocalZero},
    normed::{Normed, UltraNormed, Valuation},
    traits::HasDigits,
    ZAdic,
};
use super::Polynomial;


pub (super) fn wrap_poly(poly: Polynomial<ZAdic>) -> Polynomial::<ZAdicZeroWrapper> {
    Polynomial::new(poly.into_coefficients().map(ZAdicZeroWrapper).collect())
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub (super) struct ZAdicZeroWrapper(pub (super) ZAdic);



impl std::fmt::Display for ZAdicZeroWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


impl LocalZero for ZAdicZeroWrapper {
    fn local_zero(&self) -> Self {
        Self(self.0.local_zero())
    }
    fn is_local_zero(&self) -> bool {
        self.0.is_approx_zero()
    }
}

impl LocalOne for ZAdicZeroWrapper {
    fn local_one(&self) -> Self {
        Self(self.0.local_one())
    }
    fn is_local_one(&self) -> bool {
        self.0.is_approx_one()
    }
}

impl LocalNum for ZAdicZeroWrapper {
    type FromStrRadixErr = AdicError;
    fn from_str_radix(
        s: &str,
        radix: u32,
    ) -> Result<Self, Self::FromStrRadixErr> {
        Ok(Self(ZAdic::from_str_radix(s, radix)?))
    }
}

impl LocalInteger for ZAdicZeroWrapper {
    fn gcd(&self, other: &Self) -> Self {
        Self(self.0.gcd(&other.0))
    }
    fn lcm(&self, other: &Self) -> Self {
        Self(self.0.lcm(&other.0))
    }
    fn is_multiple_of(&self, other: &Self) -> bool {
        self.0.is_multiple_of(&other.0)
    }
}

impl Normed for ZAdicZeroWrapper {
    type Norm = Ratio<u32>;
    type Unit = ZAdicZeroWrapper;
    fn norm(&self) -> Self::Norm {
        match self.valuation() {
            Valuation::PosInf => Ratio::zero(),
            Valuation::Finite(_) => self.0.norm(),
        }
    }
    fn unit(&self) -> Option<Self::Unit> {
        match self.valuation() {
            Valuation::PosInf => None,
            Valuation::Finite(_) => self.0.unit().map(Self)
        }
    }
    fn into_unit(self) -> Option<Self::Unit> {
        match self.valuation() {
            Valuation::PosInf => None,
            Valuation::Finite(_) => self.0.into_unit().map(Self)
        }
    }
    fn from_norm_and_unit(norm: Self::Norm, u: Self::Unit) -> Self {
        Self(ZAdic::from_norm_and_unit(norm, u.0))
    }
    fn from_unit(u: Self::Unit) -> Self {
        u
    }
    fn is_unit(&self) -> bool {
        self.valuation().is_zero()
    }
}

impl UltraNormed for ZAdicZeroWrapper {
    type ValuationRing = usize;
    fn from_unit_and_valuation(u: Self::Unit, v: Valuation<Self::ValuationRing>) -> Self {
        Self(ZAdic::from_unit_and_valuation(u.0, v))
    }
    fn valuation(&self) -> Valuation<usize> {
        // Note this will give a DIFFERENT valuation than for ZAdic!
        // ...000 will have infinite valuation instead of 3.
        if self.is_local_zero() {
            Valuation::PosInf
        } else {
            Valuation::Finite(
                self.0.digits().take_while(Zero::is_zero).count()
            )
        }
    }
}


impl std::ops::Add for ZAdicZeroWrapper {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub for ZAdicZeroWrapper {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl std::ops::Mul for ZAdicZeroWrapper {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl std::ops::Div for ZAdicZeroWrapper {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl std::ops::Rem for ZAdicZeroWrapper {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}
