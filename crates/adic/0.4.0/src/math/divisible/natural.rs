use std::{
    collections::HashMap,
    fmt::Display,
    num::NonZero,
    ops::{Add, Div, Mul, Rem},
};
use num::{pow::Pow, BigInt, BigUint, Integer, One, Zero};
use num_prime::nt_funcs::factorize;
use super::{Composite, Divisible, Prime, PrimePower};


/// Natural number, including zero: `c = prod p_k^n_k OR 0`.
/// Holds a [`Composite`] or `Zero` in an enum.
///
/// Note this struct does NOT implement [`Divisible`](crate::Divisible).
/// That trait is meant for numbers that can be uniquely decomposed into `Prime`s.
/// This is not the case, since `Natural` can be zero.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Natural {
    /// The natural number 0
    Zero,
    /// A nonzero natural number
    Nonzero(Composite),
}


impl Natural {

    /// Greatest common divisor
    /// ```
    /// use adic::Natural;
    /// let n1 = Natural::from(6);
    /// let n2 = Natural::from(15);
    /// assert_eq!(Natural::from(3), n1.gcd(&n2));
    /// ```
    ///
    /// # Panics
    /// Panics if either self or other are zero
    #[must_use]
    pub fn gcd(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::Zero, _) | (_, Self::Zero) => panic!("not sure what to do with gcd(0, x); infinity?"),
            (Self::Nonzero(cs), Self::Nonzero(co)) => Self::Nonzero(cs.gcd(co)),
        }
    }

    /// Least common multiple
    /// ```
    /// use adic::Natural;
    /// let n1 = Natural::from(6);
    /// let n2 = Natural::from(15);
    /// assert_eq!(Natural::from(30), n1.lcm(&n2));
    /// assert_eq!(Natural::from(0), n1.lcm(&Natural::from(0)));
    /// ```
    #[must_use]
    pub fn lcm(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::Zero, _) | (_, Self::Zero) => Self::Zero,
            (Self::Nonzero(cs), Self::Nonzero(co)) => Self::Nonzero(cs.lcm(co)),
        }
    }

}


impl From<Prime> for Natural {
    fn from(value: Prime) -> Self {
        Self::from(Composite::from(value))
    }
}

impl From<PrimePower> for Natural {
    fn from(value: PrimePower) -> Self {
        Self::from(Composite::from(value))
    }
}

impl From<Composite>  for Natural {
    fn from(value: Composite) -> Self {
        Self::Nonzero(value)
    }
}

impl From<Natural> for u32 {
    fn from(n: Natural) -> Self {
        match n {
            Natural::Zero => 0,
            Natural::Nonzero(c) => c.prime_powers().map(u32::from).product(),
        }
    }
}

impl From<u32> for Natural {
    fn from(u: u32) -> Self {
        if let Some(nz) = NonZero::new(u) {
            Natural::Nonzero(Composite::new(factorize(u32::from(nz)).into_iter().map(|(p, p_pow)| {
                let p_pow32 = u32::try_from(p_pow).expect("usize -> u32 conversion");
                PrimePower::from((p, p_pow32))
            })))
        } else {
            Natural::Zero
        }
    }
}

impl From<Natural> for BigUint {
    fn from(n: Natural) -> Self {
        match n {
            Natural::Zero => 0u32.into(),
            Natural::Nonzero(c) => c.prime_powers().map(BigUint::from).product(),
        }
    }
}

impl From<BigUint> for Natural {
    fn from(i: BigUint) -> Self {
        if i.is_zero() {
            Natural::Zero
        } else {
            Natural::Nonzero(Composite::new(factorize(i).into_iter().map(|(p, p_pow)| {
                let p_pow32 = u32::try_from(p_pow).expect("usize -> u32 conversion");
                PrimePower::from((p, p_pow32))
            })))
        }
    }
}


impl Display for Natural {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Zero => write!(f, "0"),
            Self::Nonzero(c) => write!(f, "{c}"),
        }
    }
}


impl Zero for Natural {
    fn zero() -> Self {
        Self::Zero
    }
    fn is_zero(&self) -> bool {
        matches!(self, Self::Zero)
    }
}

impl One for Natural {
    fn one() -> Self {
        Self::Nonzero(Composite::one())
    }
    fn is_one(&self) -> bool
    where Self: PartialEq, {
        match self {
            Self::Zero => false,
            Self::Nonzero(c) => c.is_one(),
        }
    }
}


impl Add for Natural {
    type Output = Natural;
    fn add(self, rhs: Self) -> Self::Output {
        // For now: change to BigUint, do sum, change back
        let self_bigint = BigUint::from(self);
        let rhs_bigint = BigUint::from(rhs);
        Natural::from(self_bigint + rhs_bigint)
    }
}

impl Mul for Natural {
    type Output = Natural;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Nonzero(s), Self::Nonzero(r)) => Self::Nonzero(s * r),
            _ => Self::Zero,
        }
    }
}

impl Pow<u32> for Natural {
    type Output = Natural;
    fn pow(self, power: u32) -> Self::Output {
        match (self, power) {
            (Self::Nonzero(s), _) => Self::Nonzero(s.pow(power)),
            (Self::Zero, 0) => Self::one(), // <- To match the behavior of num::traits::pow
            (Self::Zero, _) => Self::Zero,
        }
    }
}

impl Rem<Composite> for Natural {
    type Output = BigUint;
    fn rem(self, rhs: Composite) -> Self::Output {
        match self {
            Self::Zero => 0u32.into(),
            Self::Nonzero(c) => {
                // Chinese remainder theorem
                // Split into prime powers (relatively prime to eachother) q_i
                // Calculate composite without q_i: Q_i = c / q_i
                // Calculate remainders for each q_i, giving a_i
                // Solve the Bezout equation M_i Q_i + m_i q_i = 1
                // Total remainder is x = sum_i ( a_i M_i Q_i )
                c.prime_powers().map(|pp| {
                    let p = pp.p();
                    let cwop = c.without_p(p);
                    let q = BigUint::from(pp);
                    let cwop = BigUint::from(cwop);
                    let e = BigInt::extended_gcd(&BigInt::from(q), &BigInt::from(cwop.clone()));
                    let bez_y = (e.y % BigInt::from(BigUint::from(rhs.clone()))).magnitude().clone();
                    let rem = BigUint::from(c.clone()) % BigUint::from(rhs.clone());
                    rem * cwop * bez_y
                }).sum()
            }
        }
    }
}



#[cfg(test)]
mod tests {

    use num::{traits::Pow, One, Zero};
    use super::Natural;

    #[test]
    fn gcd_lcm() {

        let n1 = Natural::from(12);
        let n2 = Natural::from(210);
        assert_eq!(Natural::from(6), n1.gcd(&n2));
        assert_eq!(Natural::from(6), n2.gcd(&n1));
        assert_eq!(Natural::from(420), n1.lcm(&n2));
        assert_eq!(Natural::from(420), n2.lcm(&n1));

    }

    #[test]
    fn ops() {

        let n1 = Natural::from(157);
        let n2 = Natural::from(7654);
        let z = Natural::zero();
        let o = Natural::one();
        assert_eq!(Natural::from(7811), n1.clone() + n2.clone());
        assert_eq!(Natural::from(157 * 7654), n1.clone() * n2.clone());
        assert_eq!(Natural::from(157 * 157 * 157 * 157), n1.clone().pow(4));
        assert_eq!(n1, n1.clone() + z.clone());
        assert_eq!(z, n1.clone() * z.clone());
        assert_eq!(o, z.clone().pow(0u32));
        assert_eq!(Natural::from(158), n1.clone() + o.clone());
        assert_eq!(n1, n1.clone() * o.clone());
        assert_eq!(o, o.clone().pow(0u32));

    }

}
