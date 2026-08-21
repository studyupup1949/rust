use std::{
    iter::{once, repeat_n},
    fmt::Display,
    ops::{Div, Range, Rem},
};
use itertools::{Itertools, MultiProduct};
use num::{traits::Pow, BigUint, Integer, One};

use super::{Composite, Divisible, Natural, Prime};


/// Prime power p^n
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PrimePower(Prime, u32);

impl PrimePower {

    /// Prime `p`
    pub fn p(&self) -> Prime {
        self.0
    }

    /// Power of the prime `n`
    pub fn power(&self) -> u32 {
        self.1
    }

    /// Modular negation `p^n - d`
    ///
    /// ```
    /// # use adic::PrimePower;
    /// let pp = PrimePower::from((5, 2));
    /// assert_eq!(23, pp.mod_neg(2));
    /// ```
    pub fn mod_neg(&self, d: u32) -> u32 {
        u32::from(self.0).pow(self.1) - d
    }

    /// Modular exponentiation
    ///
    /// ```
    /// # use adic::PrimePower;
    /// let pp = PrimePower::from((5, 2));
    /// assert_eq!(2, pp.mod_exp(3, 3));
    /// ```
    pub fn mod_exp(&self, d: u32, x: u32) -> u32 {

        if d == 0 {
            return 0;
        } else if x == 0 {
            return 1;
        }

        let (mut mult, mut ans, mut pow) = (d, 1, x);
        while pow > 1 {
            if pow.is_odd() {
                // ans mult^pow = (ans * mult) * mult^(pow-1)
                ans = ans * mult % self;
                pow = pow - 1;
            }
            // mult^pow = (mult^2)^(pow/2)
            mult = mult * mult % self;
            pow = pow / 2;
        }

        (ans * mult) % self

    }

}



impl From<Prime> for PrimePower {
    fn from(value: Prime) -> Self {
        Self(value, 1)
    }
}

impl<P> From<(P, u32)> for PrimePower
where P: Into<Prime> {
    fn from(pp: (P, u32)) -> Self {
        Self(pp.0.into(), pp.1)
    }
}

impl From<PrimePower> for u32 {
    fn from(pp: PrimePower) -> Self {
        u32::from(pp.0).pow(pp.1)
    }
}

impl From<&PrimePower> for u32 {
    fn from(pp: &PrimePower) -> Self {
        u32::from(pp.0).pow(pp.1)
    }
}

impl From<PrimePower> for BigUint {
    fn from(pp: PrimePower) -> Self {
        BigUint::from(pp.0).pow(pp.1)
    }
}

impl From<&PrimePower> for BigUint {
    fn from(pp: &PrimePower) -> Self {
        BigUint::from(pp.0).pow(pp.1)
    }
}

impl Display for PrimePower {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}^{}", self.0, self.1)
    }
}


impl Rem<&PrimePower> for u32 {
    type Output = Self;
    fn rem(self, rhs: &PrimePower) -> Self::Output {
        self % u32::from(rhs)
    }
}

impl Rem<PrimePower> for u32 {
    type Output = Self;
    fn rem(self, rhs: PrimePower) -> Self::Output {
        self.rem(&rhs)
    }
}

impl Div<&PrimePower> for u32 {
    type Output = Self;
    fn div(self, rhs: &PrimePower) -> Self::Output {
        self / u32::from(rhs)
    }
}

impl Div<PrimePower> for u32 {
    type Output = Self;
    fn div(self, rhs: PrimePower) -> Self::Output {
        self.div(&rhs)
    }
}

impl Pow<u32> for PrimePower {
    type Output = PrimePower;
    fn pow(self, power: u32) -> Self::Output {
        PrimePower(self.0, self.1 * power)
    }
}


#[cfg(test)]
mod tests {

    use super::PrimePower;

    #[test]
    fn modular_methods() {
        let pp = PrimePower(5.into(), 2);

        assert_eq!(23, pp.mod_neg(2));
        assert_eq!(3, pp.mod_neg(22));

        assert_eq!(1, pp.mod_exp(5, 0));
        assert_eq!(5, pp.mod_exp(5, 1));
        assert_eq!(0, pp.mod_exp(5, 2));
        assert_eq!(0, pp.mod_exp(5, 3));
        assert_eq!(2, pp.mod_exp(3, 3));
        assert_eq!(6, pp.mod_exp(3, 4));
        assert_eq!(0, pp.mod_exp(0, 0));
        assert_eq!(0, pp.mod_exp(0, 3));
        assert_eq!(1, pp.mod_exp(1, 7));

    }

}
