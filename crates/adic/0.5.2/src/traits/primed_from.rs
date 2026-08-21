use std::{
    convert::Infallible,
    iter::repeat,
};
use num::{
    bigint::Sign,
    traits::{Euclid, Pow},
    BigInt, BigRational, BigUint, One, Rational32, Zero,
};
use crate::{
    combinatorics::carmichael,
    divisible::{Composite, Prime},
    error::{AdicError, AdicResult},
    normed::{Normed, UltraNormed, Valuation},
    EAdic, QAdic, RAdic, UAdic, ZAdic,
};
use super::{AdicInteger, AdicPrimitive, CanTruncate, HasDigits};


/// Can be created from `N` with the given `Prime`
pub trait PrimedFrom<N> {

    /// Convert from `N` to `Self` with `Prime` `p`
    ///
    /// ```
    /// # use num::{BigUint, BigInt, BigRational, Rational32};
    /// # use adic::{traits::PrimedFrom, EAdic, QAdic, UAdic, ZAdic};
    /// assert_eq!("123._5", UAdic::primed_from(5, 38).to_string());
    /// assert_eq!("123._5", EAdic::primed_from(5, BigUint::from(38u32)).to_string());
    /// assert_eq!("123._5", ZAdic::primed_from(5, 38).to_string());
    /// assert_eq!("(4)23._5", EAdic::primed_from(5, -12).to_string());
    /// assert_eq!("(4)23._5", ZAdic::primed_from(5, -BigInt::from(12)).to_string());
    /// let n = Rational32::new(1, 24);
    /// assert_eq!("(34)4._5", QAdic::<EAdic>::primed_from(5, n).to_string());
    /// let n = BigRational::new(1.into(), 24.into());
    /// assert_eq!("(34)4._5", QAdic::<ZAdic>::primed_from(5, n).to_string());
    /// // Does not compile because i32 could be negative and `UAdic` cannot hold a negative number
    /// // assert_eq!("(4)23._5", UAdic::primed_from(5, -12).to_string());
    /// ```
    fn primed_from<P>(p: P, n: N) -> Self
    where P: Into<Prime>;

}

/// Can be converted into `A` with the given `Prime`
pub trait PrimedInto<A> {

    /// Convert from `Self` to `A` with `Prime` `p`
    ///
    /// ```
    /// # use num::{BigUint, BigInt, BigRational, Rational32};
    /// # use adic::{traits::PrimedInto, EAdic, QAdic, UAdic, ZAdic};
    /// let u: UAdic = 38u32.primed_into(5);
    /// assert_eq!("123._5", u.to_string());
    /// let e: EAdic = (-12).primed_into(5);
    /// assert_eq!("(4)23._5", e.to_string());
    /// let n = Rational32::new(1, 24);
    /// let q: QAdic<ZAdic> = n.primed_into(5);
    /// assert_eq!("(34)4._5", q.to_string());
    /// // Does not compile because i32 could be negative and `UAdic` cannot hold a negative number
    /// // let u: UAdic = (-12).primed_into(5);
    /// ```
    fn primed_into<P>(self, p: P) -> A
    where P: Into<Prime>;

}


// Impl PrimedInto in reverse to PrimedFrom
impl<A, N> PrimedInto<A> for N
where A: PrimedFrom<N> {
    fn primed_into<P>(self, p: P) -> A
    where P: Into<Prime> {
        A::primed_from(p, self)
    }
}


/// Can attempt to create from `N` with the given `Prime`
pub trait TryPrimedFrom<N>
where Self: Sized {

    /// The type returned in the event of a conversion error
    type Error;


    /// Convert from `N` to `Self` with `Prime` `p`
    ///
    /// ```
    /// # use num::{BigUint, BigInt, BigRational, Rational32};
    /// # use adic::{error::AdicError, traits::TryPrimedFrom, EAdic, QAdic, UAdic, ZAdic};
    /// assert_eq!(Ok("123._5"), UAdic::try_primed_from(5, 38).map(|a| a.to_string()).as_deref());
    /// assert_eq!(Ok("(4)23._5"), EAdic::try_primed_from(5, -12).map(|a| a.to_string()).as_deref());
    /// let n = Rational32::new(1, 24);
    /// assert_eq!(Ok("(34)4._5"), EAdic::try_primed_from(5, n).map(|a| a.to_string()).as_deref());
    /// let n = Rational32::new(1, 5);
    /// assert_eq!(Err(AdicError::DivideByPrime), EAdic::try_primed_from(5, n));
    /// ```
    fn try_primed_from<P>(p: P, n: N) -> Result<Self, Self::Error>
    where P: Into<Prime>;

}

/// Can attempt to convert into `A` with the given `Prime`
pub trait TryPrimedInto<A>
where Self: Sized {

    /// The type returned in the event of a conversion error
    type Error;

    /// Convert from `Self` to `A` with `Prime` `p`
    ///
    /// ```
    /// # use num::{BigUint, BigInt, BigRational, Rational32};
    /// # use adic::{error::AdicError, traits::TryPrimedInto, EAdic, QAdic, UAdic, ZAdic};
    /// let u: Result<UAdic, _> = 38.try_primed_into(5);
    /// assert_eq!(Ok("123._5"), u.map(|a| a.to_string()).as_deref());
    /// let e: Result<EAdic, _> = (-12).try_primed_into(5);
    /// assert_eq!(Ok("(4)23._5"), e.map(|a| a.to_string()).as_deref());
    /// let e: Result<EAdic, _> = Rational32::new(1, 24).try_primed_into(5);
    /// assert_eq!(Ok("(34)4._5"), e.map(|a| a.to_string()).as_deref());
    /// let e: Result<EAdic, _> = Rational32::new(1, 5).try_primed_into(5);
    /// assert_eq!(Err(AdicError::DivideByPrime), e);
    /// ```
    fn try_primed_into<P>(self, p: P) -> Result<A, Self::Error>
    where P: Into<Prime>;

}


// Impl TryPrimedFrom for all PrimedFrom
impl<A, N> TryPrimedFrom<N> for A
where A: PrimedFrom<N> {
    type Error = Infallible;
    fn try_primed_from<P>(p: P, n: N) -> Result<Self, Self::Error>
    where P: Into<Prime> {
        Ok(Self::primed_from(p, n))
    }
}

// Impl TryPrimedInto in reverse to TryPrimedFrom
impl<A, N> TryPrimedInto<A> for N
where A: TryPrimedFrom<N> {
    type Error = A::Error;
    fn try_primed_into<P>(self, p: P) -> Result<A, Self::Error>
    where P: Into<Prime> {
        A::try_primed_from(p, self)
    }
}


// Unsigned

impl<T> PrimedFrom<u32> for T
where T: AdicPrimitive {
    fn primed_from<P>(p: P, n: u32) -> Self
    where P: Into<Prime> {
        let p = p.into();
        let mut m = n;
        let mut digits = vec![];
        while m != 0 {
            digits.push(m % p);
            m = m / p;
        }
        Self::from(UAdic::new(p, digits))
    }
}
impl<T> PrimedFrom<BigUint> for T
where T: AdicPrimitive {
    fn primed_from<P>(p: P, n: BigUint) -> Self
    where P: Into<Prime> {
        let p = BigUint::from(p.into());
        let mut m = n;
        let mut digits = vec![];
        while !m.is_zero() {
            let (quot, rem) = m.div_rem_euclid(&p);
            let rem = rem.try_into().expect("BigUint -> u32 conversion");
            digits.push(rem);
            m = quot;
        }
        Self::from(UAdic::new(p, digits))
    }
}


// Signed

impl<T> PrimedFrom<i32> for T
where T: PrimedFrom<u32> + std::ops::Neg<Output=Self> + std::ops::Sub<Output=Self> {
    fn primed_from<P>(p: P, n: i32) -> Self
    where P: Into<Prime> {
        let p = p.into();
        if n.is_negative() {
            -Self::primed_from(p, n.unsigned_abs())
        } else {
            Self::primed_from(p, n.unsigned_abs())
        }
    }
}
impl<T> PrimedFrom<BigInt> for T
where T: PrimedFrom<BigUint> + std::ops::Neg<Output=Self> + std::ops::Sub<Output=Self> {
    fn primed_from<P>(p: P, n: BigInt) -> Self
    where P: Into<Prime> {
        let p = p.into();
        match n.into_parts() {
            (Sign::Minus, n) => -Self::primed_from(p, n),
            (_, n) => Self::primed_from(p, n),
        }
    }
}


// Rational

impl<T> PrimedFrom<Rational32> for QAdic<T>
where T: AdicInteger, Self: From<QAdic<EAdic>> {
    fn primed_from<P>(p: P, n: Rational32) -> Self
    where P: Into<Prime> {
        Self::primed_from(p, BigRational::new((*n.numer()).into(), (*n.denom()).into()))
    }
}
impl<T> PrimedFrom<BigRational> for QAdic<T>
where T: AdicInteger, Self: From<QAdic<EAdic>> {
    fn primed_from<P>(p: P, n: BigRational) -> Self
    where P: Into<Prime> {

        // Calculate this by
        // - Retrieve the numerator and denominator
        // - Factor out the prime, p, giving the QAdic valuation
        // - Use carmichael number to find a power that lets denom divide (p^power - 1)
        // - Calculate -1/denom = ((p^power-1)/denom) * -1/(p^power-1)
        // - Calculate -numer
        // - QAdic::new(, valuation)

        // - Retrieve the numerator and denominator
        let p = p.into();
        let (numer, denom) = n.clone().into_raw();
        let (nsgn, numer) = numer.into_parts();
        let (dsgn, denom) = denom.into_parts();
        let sgn = nsgn * dsgn;
        if numer.is_zero() {
            return QAdic::zero(p);
        } else if denom.is_one() {
            let u = match sgn {
                Sign::Plus => EAdic::primed_from(p, numer),
                Sign::Minus => -EAdic::primed_from(p, numer),
                Sign::NoSign => {
                    panic!("Rational does not have expected sign during rational conversion");
                },
            };
            return QAdic::new(u, 0).into()
        }

        // Neither should be zero now, so convert to Composite
        let mut cnumer = Composite::from(numer);
        let mut cdenom = Composite::from(denom);

        // - Factor out the prime, p, giving the QAdic valuation
        let numer_pow = isize::try_from(cnumer.remove_prime(p).power()).unwrap();
        let denom_pow = isize::try_from(cdenom.remove_prime(p).power()).unwrap();
        let valuation = Valuation::from(numer_pow) - Valuation::from(denom_pow);
        let Some(valuation) = valuation else {
            panic!("Something went wrong; infinite negative valuation during rational conversion");
        };

        let reduced_numer = BigUint::from(cnumer);
        let reduced_denom = BigUint::from(cdenom);

        // - Use carmichael number to find a power that lets denom divide (p^power - 1)
        let carm = carmichael(u64::try_from(reduced_denom.clone()).expect("BigUint -> u64 conversion"));
        let power = u32::try_from(carm).expect("u64 -> u32 conversion");
        let power_usize = usize::try_from(carm).expect("u32 -> usize conversion");

        // - Calculate -1/denom = ((p^power-1)/denom) * -1/(p^power-1)
        let ppm1_over_denom = (BigUint::from(p.pow(power)) - BigUint::one()) / reduced_denom;
        let basic_repeat = UAdic::primed_from(p, ppm1_over_denom);
        let padded_repeat = basic_repeat.digits().chain(repeat(0)).take(power_usize).collect();
        let r = EAdic::from(RAdic::new(p, vec![], padded_repeat));

        // - Calculate -numer
        let u = EAdic::primed_from(p, reduced_numer);

        // - QAdic::new(-numer * -1/denom, valuation)
        let q = QAdic::new(u * r, valuation);
        match sgn {
            Sign::Plus => (-q).into(),
            Sign::Minus => q.into(),
            Sign::NoSign => {
                panic!("Rational does not have expected sign during rational conversion");
            },
        }

    }
}


// Try Rational

impl TryPrimedFrom<Rational32> for EAdic {
    type Error = AdicError;
    fn try_primed_from<P>(p: P, n: Rational32) -> AdicResult<Self>
    where P: Into<Prime> {
        let p = p.into();
        let qa = QAdic::primed_from(p, n);
        if qa.valuation() >= 0.into() {
            Ok(qa.quotient(0))
        } else {
            Err(AdicError::DivideByPrime)
        }
    }
}
impl TryPrimedFrom<Rational32> for RAdic {
    type Error = AdicError;
    fn try_primed_from<P>(p: P, n: Rational32) -> AdicResult<Self>
    where P: Into<Prime> {
        Ok(EAdic::try_primed_from(p, n)?.into())
    }
}
impl TryPrimedFrom<Rational32> for ZAdic {
    type Error = AdicError;
    fn try_primed_from<P>(p: P, n: Rational32) -> AdicResult<Self>
    where P: Into<Prime> {
        Ok(EAdic::try_primed_from(p, n)?.into())
    }
}

impl TryPrimedFrom<BigRational> for EAdic {
    type Error = AdicError;
    fn try_primed_from<P>(p: P, n: BigRational) -> AdicResult<Self>
    where P: Into<Prime> {
        let p = p.into();
        let qa = QAdic::primed_from(p, n);
        if qa.valuation() > 0.into() {
            Ok(qa.into_unit().unwrap_or(EAdic::zero(p)))
        } else {
            Err(AdicError::DivideByPrime)
        }
    }
}
impl TryPrimedFrom<BigRational> for RAdic {
    type Error = AdicError;
    fn try_primed_from<P>(p: P, n: BigRational) -> AdicResult<Self>
    where P: Into<Prime> {
        Ok(EAdic::try_primed_from(p, n)?.into())
    }
}
impl TryPrimedFrom<BigRational> for ZAdic {
    type Error = AdicError;
    fn try_primed_from<P>(p: P, n: BigRational) -> AdicResult<Self>
    where P: Into<Prime> {
        Ok(EAdic::try_primed_from(p, n)?.into())
    }
}



#[cfg(test)]
mod test {

    use num::Rational32;
    use crate::{error::AdicError, normed::Normed, EAdic, QAdic};
    use super::{PrimedFrom, PrimedInto, TryPrimedFrom, TryPrimedInto};

    #[test]
    fn primed_from_into() {

        assert_eq!(eadic!(3, [2]), EAdic::primed_from(3, 2u32));
        assert_eq!(eadic!(3, [2]), 2u32.primed_into(3));
        assert_eq!(eadic_neg!(3, [1]), EAdic::primed_from(3, -2));
        assert_eq!(eadic_neg!(3, [1]), (-2).primed_into(3));
        assert_eq!(qadic!(eadic!(3, [1]), -1), QAdic::primed_from(3, Rational32::new(1, 3)));
        assert_eq!(qadic!(eadic!(3, [1]), -1), Rational32::new(1, 3).primed_into(3));

        // Large denominator still reasonably fast
        assert!(QAdic::<EAdic>::primed_from(5, Rational32::new(17, 10001)).is_unit());

    }

    #[test]
    fn try_primed_from_into() {

        assert_eq!(Ok(eadic!(3, [2])), EAdic::try_primed_from(3, Rational32::new(2, 1)));
        assert_eq!(Ok(eadic!(3, [2])), Rational32::new(2, 1).try_primed_into(3));
        assert_eq!(Ok(eadic_neg!(3, [1])), EAdic::try_primed_from(3, Rational32::new(-2, 1)));
        assert_eq!(Ok(eadic_neg!(3, [1])), Rational32::new(-2, 1).try_primed_into(3));
        assert_eq!(Ok(eadic_rep!(3, [2], [1])), EAdic::try_primed_from(3, Rational32::new(1, 2)));
        assert_eq!(Ok(eadic_rep!(3, [2], [1])), Rational32::new(1, 2).try_primed_into(3));
        assert_eq!(Err(AdicError::DivideByPrime), EAdic::try_primed_from(3, Rational32::new(1, 3)));
        let e: Result<EAdic, _> = Rational32::new(1, 3).try_primed_into(3);
        assert_eq!(Err(AdicError::DivideByPrime), e);

        // Large denominator still reasonably fast
        assert!(EAdic::try_primed_from(5, Rational32::new(-200000, 10001)).is_ok());

    }

}
