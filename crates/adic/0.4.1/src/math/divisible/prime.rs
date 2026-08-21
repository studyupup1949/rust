use std::{
    fmt::Display,
    ops::{Div, Rem},
};
use num::{traits::Pow, BigUint, Integer};
use num_prime::nt_funcs::is_prime;
use super::PrimePower;


/// Prime number p
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Prime(u32);

impl Prime {

    /// Is this prime two (a very special case)
    pub fn is_two(&self) -> bool {
        self.0 == 2
    }

    /// p-2
    pub fn m2(&self) -> u32 {
        self.0 - 2
    }

    /// Modular negation p - d
    ///
    /// ```
    /// # use adic::Prime;
    /// let p = Prime::from(13);
    /// assert_eq!(11, p.mod_neg(2));
    /// ```
    pub fn mod_neg(&self, d: u32) -> u32 {
        self.0 - d
    }

    /// Modular exponentiation
    ///
    /// ```
    /// # use adic::Prime;
    /// let p = Prime::from(13);
    /// assert_eq!(8, p.mod_exp(5, 3));
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

    /// Modular inverse
    ///
    /// ```
    /// # use adic::Prime;
    /// let p = Prime::from(13);
    /// assert_eq!(10, p.mod_inv(4));
    /// ```
    pub fn mod_inv(&self, d: u32) -> u32 {
        // d^(p-1) % p = 1  Fermat's little theorem
        // d^-1 % p = d^(p-2) % p
        self.mod_exp(d, self.0-2)
    }

}



// TODO: Change to TryFrom with AdicError
impl From<u32> for Prime {
    fn from(p: u32) -> Self {
        assert!(is_prime(&p, None).probably(), "{p} is not prime");
        Self(p)
    }
}

impl From<BigUint> for Prime {
    fn from(p: BigUint) -> Self {
        assert!(is_prime(&p, None).probably(), "{p} is not prime");
        let p = u32::try_from(p).expect("prime should be convertible to u32");
        Self(p)
    }
}


impl From<Prime> for u32 {
    fn from(p: Prime) -> Self {
        p.0
    }
}

impl From<&Prime> for u32 {
    fn from(p: &Prime) -> Self {
        p.0
    }
}

impl From<Prime> for BigUint {
    fn from(p: Prime) -> Self {
        p.0.into()
    }
}

impl From<&Prime> for BigUint {
    fn from(p: &Prime) -> Self {
        p.0.into()
    }
}

impl AsRef<u32> for Prime {
    fn as_ref(&self) -> &u32 {
        &self.0
    }
}

impl Display for Prime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}


impl Rem<&Prime> for u32 {
    type Output = Self;
    fn rem(self, rhs: &Prime) -> Self::Output {
        self % u32::from(rhs)
    }
}

impl Rem<Prime> for u32 {
    type Output = Self;
    fn rem(self, rhs: Prime) -> Self::Output {
        self.rem(&rhs)
    }
}

impl Div<&Prime> for u32 {
    type Output = Self;
    fn div(self, rhs: &Prime) -> Self::Output {
        self / u32::from(rhs)
    }
}

impl Div<Prime> for u32 {
    type Output = Self;
    fn div(self, rhs: Prime) -> Self::Output {
        self.div(&rhs)
    }
}

impl Pow<u32> for Prime {
    type Output = PrimePower;
    fn pow(self, power: u32) -> Self::Output {
        PrimePower::from((self, power))
    }
}


#[cfg(test)]
mod tests {

    use super::Prime;

    #[test]
    fn modular_methods() {
        let p = Prime::from(13);

        assert_eq!(11, p.mod_neg(2));
        assert_eq!(6, p.mod_neg(7));

        assert_eq!(1, p.mod_exp(5, 0));
        assert_eq!(5, p.mod_exp(5, 1));
        assert_eq!(12, p.mod_exp(5, 2));
        assert_eq!(8, p.mod_exp(5, 3));
        assert_eq!(0, p.mod_exp(0, 0));
        assert_eq!(0, p.mod_exp(0, 3));
        assert_eq!(1, p.mod_exp(1, 7));

        assert_eq!(1, p.mod_inv(1));
        assert_eq!(7, p.mod_inv(2));
        assert_eq!(9, p.mod_inv(3));
        assert_eq!(10, p.mod_inv(4));
        assert_eq!(8, p.mod_inv(5));
        assert_eq!(11, p.mod_inv(6));

        for d in 1..p.into() {
            assert_eq!(1, d * p.mod_inv(d) % p)
        }

    }

}
