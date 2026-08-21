use std::{
    collections::HashMap,
    fmt::Display,
    iter::repeat_n,
    num::{NonZero, TryFromIntError},
    ops::{Div, Mul, Rem},
};
use itertools::Itertools;
use num::{traits::Pow, BigUint, One, Zero};
use num_prime::nt_funcs::factorize;
use crate::AdicError;
use super::{Prime, PrimePower};


/// Composite natural number: `c = prod p_k^n_k`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Composite {
    prime_map: HashMap<Prime, PrimePower>,
}

impl Composite {

    /// Construct from prime power iterator (its factors)
    pub fn new<I, P>(value: I) -> Self
    where I: IntoIterator<Item = P>, P: Into<PrimePower> {
        // TODO: Make sure each prime is distinct
        Self {
            prime_map: value.into_iter().map(|pp| {
                let pp = pp.into();
                (pp.p(), pp)
            }).collect()
        }
    }

    /// The distinct prime factors of this `Composite`
    pub fn primes(&self) -> impl Iterator<Item = Prime> + use<'_> {
        self.prime_map.keys().copied().sorted()
    }

    /// The prime power factors of this `Composite`
    pub fn prime_powers(&self) -> impl Iterator<Item = PrimePower> + use<'_> {
        self.prime_map.values().copied().sorted_by_key(PrimePower::p)
    }

    /// The prime power factor for prime `p`
    pub fn prime_power<P>(&self, p: P) -> PrimePower
    where P: Into<Prime> {
        let p = p.into();
        self.prime_map.get(&p).copied().unwrap_or(PrimePower::from((p, 0)))
    }

    /// All prime factors of this `Composite`
    pub fn factors(&self) -> impl Iterator<Item = Prime> + use<'_> {
        self.prime_powers().flat_map(
            |pp| repeat_n(pp.p(), pp.power().try_into().expect("u32 -> usize conversion"))
        )
    }

    /// If the composite contains the `Prime`
    pub fn has_prime<P>(&self, p: P) -> bool
    where P: Into<Prime> {
        self.prime_map.contains_key(&p.into())
    }

    /// Removes all powers of a [`Prime`] from the `Composite`, returning the [`PrimePower`]
    pub fn remove_prime<P>(&mut self, p: P) -> Option<PrimePower>
    where P: Into<Prime> {
        self.prime_map.remove(&p.into())
    }

    /// Remove prime p from the Composite factors
    #[must_use]
    pub fn without_p<P>(&self, p: P) -> Composite
    where P: Into<Prime> {
        let p = p.into();
        Self::new(self.prime_powers().filter(|pp| pp.p() != p))
    }


    /// Modular negation n - d
    ///
    /// ```
    /// # use adic::Composite;
    /// let c = Composite::new([(2, 1), (3, 2), (5, 1)]);
    /// assert_eq!(88, c.mod_neg(2));
    /// ```
    pub fn mod_neg(&self, d: u32) -> u32 {
        u32::from(self) - d
    }

    /// Greatest common divisor
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use adic::Composite;
    /// let n1 = Composite::try_from(6)?;
    /// let n2 = Composite::try_from(15)?;
    /// assert_eq!(Composite::try_from(3)?, n1.gcd(&n2));
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn gcd(&self, other: &Self) -> Self {
        Self::new(self.prime_map.clone().into_values().filter_map(|pp| {
            other.prime_map.get(&pp.p()).map(|opp| {
                if pp.power() > opp.power() {
                    *opp
                } else {
                    pp
                }
            })
        }))
    }

    /// Least common multiple
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use adic::Composite;
    /// let n1 = Composite::try_from(6)?;
    /// let n2 = Composite::try_from(15)?;
    /// assert_eq!(Composite::try_from(30)?, n1.lcm(&n2));
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn lcm(&self, other: &Self) -> Self {
        let mut common_pp = self.prime_map.clone();
        for opp in other.prime_map.values() {
            match common_pp.get_mut(&opp.p()) {
                None => { common_pp.insert(opp.p(), *opp); }
                Some(pp) if pp.power() < opp.power() => { *pp = *opp; }
                _ => { },
            }
        }
        Self::new(common_pp.into_values())
    }

}



impl From<NonZero<u32>> for Composite {
    fn from(value: NonZero<u32>) -> Self {
        Self::new(factorize(u32::from(value)).into_iter().map(|(p, p_pow)| {
            let p_pow32 = u32::try_from(p_pow).expect("usize -> u32 conversion");
            PrimePower::from((p, p_pow32))
        }))
    }
}

impl TryFrom<u32> for Composite {
    type Error = AdicError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(Self::from(NonZero::try_from(value)?))
    }
}

impl TryFrom<BigUint> for Composite {
    type Error = AdicError;
    fn try_from(value: BigUint) -> Result<Self, Self::Error> {
        if value.is_zero() {
            Err(AdicError::TryFromIntError)
        } else {
            Ok(Self::new(factorize(value).into_iter().map(|(p, p_pow)| {
                let p_pow32 = u32::try_from(p_pow).expect("usize -> u32 conversion");
                PrimePower::from((p, p_pow32))
            })))
        }
    }
}

impl From<Prime> for Composite {
    fn from(value: Prime) -> Self {
        Composite::new([PrimePower::from((value, 1))])
    }
}

impl From<PrimePower> for Composite {
    fn from(value: PrimePower) -> Self {
        Composite::new([value])
    }
}

impl From<Composite> for u32 {
    fn from(c: Composite) -> Self {
        c.prime_powers().map(u32::from).product()
    }
}

impl From<&Composite> for u32 {
    fn from(c: &Composite) -> Self {
        c.prime_powers().map(u32::from).product()
    }
}

impl From<Composite> for BigUint {
    fn from(c: Composite) -> Self {
        c.prime_powers().map(BigUint::from).product()
    }
}

impl From<&Composite> for BigUint {
    fn from(c: &Composite) -> Self {
        c.prime_powers().map(BigUint::from).product()
    }
}

impl TryFrom<Composite> for usize {
    type Error = TryFromIntError;
    fn try_from(c: Composite) -> Result<Self, Self::Error> {
        u32::from(c).try_into()
    }
}

impl TryFrom<Composite> for isize {
    type Error = TryFromIntError;
    fn try_from(c: Composite) -> Result<Self, Self::Error> {
        u32::from(c).try_into()
    }
}

impl Display for Composite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_one() {
            write!(f, "1")
        } else {
            write!(f, "{}", self.prime_map.values().map(PrimePower::to_string).join(" * "))
        }
    }
}


impl One for Composite {
    /// Composite(1), no prime factors, multiplicative identity
    fn one() -> Self {
        Self { prime_map: HashMap::new() }
    }
    fn is_one(&self) -> bool
    where Self: PartialEq {
        self.prime_map.is_empty()
    }
}


impl Mul for Composite {
    type Output = Composite;

    // Ignore suspicious + in the Mul impl
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, rhs: Self) -> Self::Output {
        let mut new_pp = HashMap::new();
        for (&p, pp) in &self.prime_map {
            new_pp.insert(p, *pp);
        }
        for (&p, pp) in &rhs.prime_map {
            if let Some(self_pp) = new_pp.get_mut(&p) {
                *self_pp = PrimePower::from((p, self_pp.power() + pp.power()));
            } else {
                new_pp.insert(p, *pp);
            }
        }
        Self::new(new_pp.into_values())
    }

}

impl Pow<u32> for Composite {
    type Output = Composite;
    fn pow(self, power: u32) -> Self::Output {
        let new_prime_map = self.prime_map.into_values().map(|pp| pp.pow(power));
        Composite::new(new_prime_map)
    }
}


impl Rem<&Composite> for u32 {
    type Output = Self;
    fn rem(self, rhs: &Composite) -> Self::Output {
        self % u32::from(rhs)
    }
}

impl Rem<Composite> for u32 {
    type Output = Self;
    fn rem(self, rhs: Composite) -> Self::Output {
        self.rem(&rhs)
    }
}

impl Div<&Composite> for u32 {
    type Output = Self;
    fn div(self, rhs: &Composite) -> Self::Output {
        self / u32::from(rhs)
    }
}

impl Div<Composite> for u32 {
    type Output = Self;
    fn div(self, rhs: Composite) -> Self::Output {
        self.div(&rhs)
    }
}



#[cfg(test)]
mod tests {

    use crate::{Prime, PrimePower};
    use super::Composite;

    #[test]
    fn factors() {

        let c = Composite::try_from(90).unwrap();

        assert_eq!(Composite::new([(2, 1), (3, 2), (5, 1)]), c);

        let expected_power_factors = [(2, 1), (3, 2), (5, 1)].into_iter().map(PrimePower::from).collect::<Vec<_>>();
        assert_eq!(expected_power_factors, c.prime_powers().collect::<Vec<_>>());
        let expected_all_factors = [2, 3, 3, 5].into_iter().map(Prime::from).collect::<Vec<_>>();
        assert_eq!(expected_all_factors, c.factors().collect::<Vec<_>>());
        let expected_distinct_factors = [2, 3, 5].into_iter().map(Prime::from).collect::<Vec<_>>();
        assert_eq!(expected_distinct_factors, c.primes().collect::<Vec<_>>());

        assert_eq!(Composite::try_from(45).unwrap(), c.without_p(2));
        assert_eq!(Composite::try_from(10).unwrap(), c.without_p(3));
        assert_eq!(Composite::try_from(18).unwrap(), c.without_p(5));
        assert_eq!(Composite::try_from(90).unwrap(), c.without_p(7));

    }

    #[test]
    fn modular_methods() {

        let c = Composite::try_from(90).unwrap();

        assert_eq!(88, c.mod_neg(2));
        assert_eq!(3, c.mod_neg(87));

    }

    #[test]
    fn common_factors() {

        let c1 = Composite::try_from(90).unwrap();
        let c2 = Composite::try_from(70).unwrap();

        assert_eq!(Composite::try_from(10).unwrap(), c1.gcd(&c2));
        assert_eq!(Composite::try_from(10).unwrap(), c2.gcd(&c1));
        assert_eq!(Composite::try_from(630).unwrap(), c1.lcm(&c2));
        assert_eq!(Composite::try_from(630).unwrap(), c2.lcm(&c1));

    }

}
