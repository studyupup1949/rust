use std::{
    collections::HashMap,
    fmt::Display,
    iter::repeat_n,
    num::TryFromIntError,
    ops::{Div, Mul, MulAssign, Rem},
};
use itertools::Itertools;
use num::{traits::Pow, BigUint, One, ToPrimitive, Zero};
use crate::combinatorics::prime_power_factors;
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

        let mut prime_map = HashMap::new();
        for pp in value {
            let pp = pp.into();
            let p = pp.p();
            let current_pp = prime_map.entry(p).or_insert(PrimePower::from((p, 0)));
            *current_pp *= pp;
        }

        Self {
            prime_map
        }

    }

    /// The distinct prime factors of this `Composite`
    pub fn primes(&self) -> impl Iterator<Item = Prime> + use<'_> {
        self.prime_map.iter().filter_map(|(p, pp)| (pp.power() > 0).then_some(p)).copied().sorted()
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
        self.prime_map.get(&p.into()).is_some_and(|pp| pp.power() > 0)
    }

    /// Removes all powers of a [`Prime`] from the `Composite`, returning the [`PrimePower`]
    pub fn remove_prime<P>(&mut self, p: P) -> PrimePower
    where P: Into<Prime> {
        let p = p.into();
        self.prime_map.remove(&p).unwrap_or(PrimePower::from((p, 0)))
    }

    /// Remove prime p from the Composite factors
    #[must_use]
    pub fn without_p<P>(&self, p: P) -> Composite
    where P: Into<Prime> {
        let p = p.into();
        Self::new(self.prime_powers().filter(|pp| pp.p() != p))
    }


    /// Greatest common divisor
    /// ```
    /// # use adic::divisible::Composite;
    /// let n1 = Composite::try_from(6)?;
    /// let n2 = Composite::try_from(15)?;
    /// assert_eq!(Composite::try_from(3)?, n1.gcd(&n2));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
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
    /// # use adic::divisible::Composite;
    /// let n1 = Composite::try_from(6)?;
    /// let n2 = Composite::try_from(15)?;
    /// assert_eq!(Composite::try_from(30)?, n1.lcm(&n2));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
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

    fn div_rem(self, c: &Self) -> (u32, Self) {
        let mut quotient = 0;
        let mut remainder = self;
        while c.prime_powers().all(|pp| remainder.prime_power(pp.p()).power() >= pp.power()) {
            remainder = Composite::new(
                remainder.prime_powers().map(|pp| PrimePower::from((pp.p(), pp.power() - c.prime_power(pp.p()).power())))
            );
            quotient += 1;
        }
        (quotient, remainder)
    }

    /// Performs the base-p logarithm of the `Composite`; useful for base conversion
    /// ```
    /// # use adic::divisible::Composite;
    /// let n = Composite::try_from(20)?;
    /// assert_eq!(n.log_ceil(&Composite::try_from(2)?), 5);
    /// assert_eq!(n.log_ceil(&Composite::try_from(5)?), 2);
    /// assert_eq!(n.log_ceil(&Composite::try_from(20)?), 1);
    /// let n = Composite::try_from(144).unwrap();
    /// assert_eq!(n.log_ceil(&Composite::try_from(2).unwrap()), 8);
    /// assert_eq!(n.log_ceil(&Composite::try_from(5).unwrap()), 4);
    /// assert_eq!(n.log_ceil(&Composite::try_from(12).unwrap()), 2);
    /// assert_eq!(n.log_ceil(&Composite::try_from(143).unwrap()), 2);
    /// assert_eq!(n.log_ceil(&Composite::try_from(144).unwrap()), 1);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn log_ceil(&self, c: &Composite) -> u32 {
        let (exact_log, remainder) = self.clone().div_rem(c);
        let cf64 = f64::from(u32::from(c));
        let approx_log = remainder.prime_powers().map(
            |pp| f64::from(u32::from(pp.p())).log(cf64) * f64::from(pp.power())
        ).sum::<f64>();
        exact_log + approx_log.ceil().to_u32().expect("f64 -> u32 conversion")
    }

    pub (crate) fn factor_from_u64(value: u64) -> Self {
        let factors = prime_power_factors(value).map(|(p, p_pow)| {
            let p32 = u32::try_from(p).expect("prime should be convertible to u32");
            let p_pow32 = u32::from(p_pow);
            PrimePower::from((p32, p_pow32))
        });
        Self::new(factors)
    }

    /// Carmichael function of the separate powers of `c^cpower`, in an iterator
    pub (crate) fn carmichael_iter(&self, cpower: u32) -> impl Iterator<Item = Self> + use<'_> {
        // car(n) = tot(n) if n=1,2,4, or (p>2)^k
        //  = 1/2 tot(n) if n=2^k
        //  = lcm(car(n1), car(n2), ...), where n = n1*n2*... and ni rel prime to nj
        self.prime_powers().map(move |pp| {
            let p = pp.p();
            let k = cpower * pp.power();
            let is_two = (p == 2.into());
            if is_two && [0, 1].contains(&k) {
                Composite::one()
            } else if is_two && k == 2 {
                Prime::from(2).into()
            } else if is_two {
                PrimePower::from((p, k-2)).into()
            } else {
                let c = Self::factor_from_u64(u64::from(p) - 1);
                c * Composite::from(PrimePower::from((p, k-1)))
            }
        })

    }

}



impl From<u32> for Composite {
    fn from(value: u32) -> Self {
        assert!(!value.is_zero(), "Zero cannot be a Composite");
        Self::factor_from_u64(value.into())
    }
}

impl From<BigUint> for Composite {
    fn from(value: BigUint) -> Self {
        assert!(!value.is_zero(), "Zero cannot be a Composite");
        let value64 = u64::try_from(value).expect("Cannot factor Composite larger than u64");
        Self::factor_from_u64(value64)
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


impl Mul for &Composite {
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
        Composite::new(new_pp.into_values())
    }

}
impl Mul<Composite> for &Composite {
    type Output = Composite;
    fn mul(self, rhs: Composite) -> Self::Output {
        self * &rhs
    }
}
impl Mul<&Composite> for Composite {
    type Output = Composite;
    fn mul(self, rhs: &Composite) -> Self::Output {
        &self * rhs
    }
}
impl Mul<Composite> for Composite {
    type Output = Composite;
    fn mul(self, rhs: Composite) -> Self::Output {
        &self * &rhs
    }
}

impl MulAssign for Composite {
    fn mul_assign(&mut self, rhs: Self) {
        *self = &*self * rhs;
    }
}

impl std::iter::Product for Composite {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::one(), |acc, x| acc * x)
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

    use crate::divisible::{Modulus, Prime, PrimePower};
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

        assert_eq!(Composite::new([(2, 2), (5, 3), (2, 3)]), Composite::new([(5, 2), (2, 5), (5, 1)]));

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

    #[test]
    fn log() {

        let n = Composite::try_from(20).unwrap();
        assert_eq!(n.log_ceil(&Composite::try_from(2).unwrap()), 5);
        assert_eq!(n.log_ceil(&Composite::try_from(3).unwrap()), 3);
        assert_eq!(n.log_ceil(&Composite::try_from(5).unwrap()), 2);
        assert_eq!(n.log_ceil(&Composite::try_from(10).unwrap()), 2);
        assert_eq!(n.log_ceil(&Composite::try_from(19).unwrap()), 2);
        assert_eq!(n.log_ceil(&Composite::try_from(20).unwrap()), 1);

        let n = Composite::try_from(125).unwrap();
        assert_eq!(n.log_ceil(&Composite::try_from(2).unwrap()), 7);
        assert_eq!(n.log_ceil(&Composite::try_from(3).unwrap()), 5);
        assert_eq!(n.log_ceil(&Composite::try_from(5).unwrap()), 3);
        assert_eq!(n.log_ceil(&Composite::try_from(25).unwrap()), 2);
        assert_eq!(n.log_ceil(&Composite::try_from(125).unwrap()), 1);

        let n = Composite::try_from(144).unwrap();
        assert_eq!(n.log_ceil(&Composite::try_from(2).unwrap()), 8);
        assert_eq!(n.log_ceil(&Composite::try_from(3).unwrap()), 5);
        assert_eq!(n.log_ceil(&Composite::try_from(5).unwrap()), 4);
        assert_eq!(n.log_ceil(&Composite::try_from(6).unwrap()), 3);
        assert_eq!(n.log_ceil(&Composite::try_from(11).unwrap()), 3);
        assert_eq!(n.log_ceil(&Composite::try_from(12).unwrap()), 2);
        assert_eq!(n.log_ceil(&Composite::try_from(143).unwrap()), 2);
        assert_eq!(n.log_ceil(&Composite::try_from(144).unwrap()), 1);

    }

}
