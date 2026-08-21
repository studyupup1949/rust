use std::collections::HashMap;
use itertools::Itertools;
use num::{BigUint, One};

use crate::{
    divisible::{Composite, Divisible, Prime},
    error::{AdicError, AdicResult},
    traits::{AdicPrimitive, CanApproximate, HasApproximateDigits, PrimedFrom},
    ZAdic,
};
use super::{m_adic_digits::MAdicDigits, PowAdic};



#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// An adic with a composite as base
///
/// Used to represent base-`n` left-infinite numbers.
/// Splits canonically into `p`-adic components for each prime factor `p` of `n`.
/// Internally, these are stored as [`PowAdic`] components for each [`Prime`]
///  in the base [`Composite`]'s factorization.
///
/// This composite adic is necessarily approximate.
/// Particularly when operations are performed, the zero-divisors or "idempotents" of the ring must be used.
/// These idempotents are calculated approximately, so `ZAdic` is the preferred pure p-adic integer for this struct.
/// Note that the certainty of the `PowAdic` components will be far larger than the certainty of the corresponding `MAdic`.
/// This is just because e.g. it takes more 2-adic digits and 5-adic digits to get any amount of 10-adic digits.
///
/// This and the [`PowAdic`](crate::num_adic::PowAdic) struct are composite adic structures,
///  as opposed to prime-based structs like [`UAdic`](crate::UAdic) or [`ZAdic`](crate::ZAdic).
/// `MAdic` in fact is not a field but a ring and has
///  [zero divisors](<https://en.wikipedia.org/wiki/Zero_divisor>), so operations like division make less sense.
/// Expect fewer features generally in these composite structures.
///
/// <div class="warning">
/// This struct is not particularly stable.
/// Even operations as simple as construction and Display need to be done carefully.
/// Use with approximate ZAdic for best results
/// </div>
///
/// ```
/// # use adic::{divisible::Prime, num_adic::MAdic, traits::HasDigits};
/// let three = MAdic::approx_from_i32(10, 3, 6)?;
/// assert_eq!("...000003._10", three.to_string());
/// assert_eq!(vec![Prime::from(2), Prime::from(5)], three.base().factors().collect::<Vec<_>>());
/// let two_adic_part = three.p_adic(2);
/// let five_adic_part = three.p_adic(5);
/// assert_eq!("...000000000000000000000011._2", two_adic_part.to_string());
/// assert_eq!("...000000000003._5", five_adic_part.to_string());
/// let pure_two_ten_adic = MAdic::from_pure_p_adic(10, two_adic_part.adic_ref().clone())?;
/// let pure_five_ten_adic = MAdic::from_pure_p_adic(10, five_adic_part.adic_ref().clone())?;
/// let added = pure_two_ten_adic.clone() + pure_five_ten_adic.clone();
/// let multiplied = pure_two_ten_adic.clone() * pure_five_ten_adic.clone();
/// assert_eq!("...000003._10", added.to_string());
/// assert_eq!("0._10", multiplied.to_string());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct MAdic<A>
where A: HasApproximateDigits + AdicPrimitive {
    // Perhaps store the "prime idempotent"s in the p_adics/p_adic_data, or lazy static
    pub (super) pow_adics: HashMap<Prime, PowAdic<A>>,
}


impl<A> MAdic<A>
where A: HasApproximateDigits + AdicPrimitive {

    /// Create a base-`n` adic number
    ///
    /// ```
    /// use adic::{num_adic::{MAdic, PowAdic}, ZAdic};
    /// let two_adic_four = PowAdic::new(ZAdic::new_approx(2, 30, vec![0, 0, 1]), 1);
    /// let five_adic_four = PowAdic::new(ZAdic::new_approx(5, 12, vec![4]), 1);
    /// let ten_adic_four = MAdic::new([two_adic_four, five_adic_four]);
    /// assert_eq!("...000004._10", ten_adic_four.to_string());
    /// assert_eq!("...000016._10", (ten_adic_four.clone() * ten_adic_four.clone()).to_string());
    /// ```
    pub fn new(adics: impl IntoIterator<Item=PowAdic<A>>) -> Self {
        let pow_adics = adics.into_iter().map(
            |a| (a.p(), a)
        ).collect::<HashMap<_, _>>();
        MAdic {
            pow_adics,
        }
    }

    /// Create a `MAdic` equivalent to `adic` when projected to the `p`-adics and to `0` otherwise
    ///
    /// ```
    /// use adic::{num_adic::{MAdic, PowAdic}, ZAdic};
    /// let two_adic_four = ZAdic::new_approx(2, 24, vec![0, 0, 1]);
    /// let ten_adic_pure_two_four = MAdic::from_pure_p_adic(10, two_adic_four)?;
    /// assert_eq!("...562500._10", ten_adic_pure_two_four.to_string());
    /// let five_adic_four = ZAdic::new_approx(5, 12, vec![4]);
    /// let ten_adic_pure_five_four = MAdic::from_pure_p_adic(10, five_adic_four)?;
    /// assert_eq!("...562500._10", ten_adic_pure_two_four.to_string());
    /// let ten_adic_four = ten_adic_pure_two_four + ten_adic_pure_five_four;
    /// assert_eq!("...000004._10", ten_adic_four.to_string());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    /// Error if c fails to convert to a [`Composite`] or does not contain `adic`'s prime as a factor
    pub fn from_pure_p_adic<C>(c: C, adic: A) -> AdicResult<Self>
    where C: Into<Composite> {
        let c = c.into();
        if c.has_prime(adic.p()) {
            Ok(Self::new(c.prime_powers().map(move |pp| {
                let (p, power) = (pp.p(), pp.power());
                if p == adic.p() {
                    PowAdic::new(adic.clone(), power)
                } else {
                    PowAdic::new(A::zero(p), power)
                }
            })))
        } else {
            Err(AdicError::IllDefined("`MAdic` must contain the prime of `adic`.".to_string()))
        }
    }

    /// Create mixed adic from a single signed integer
    ///
    /// ```
    /// use adic::{num_adic::{MAdic, PowAdic}, EAdic};
    /// let six_adic_11 = MAdic::from_i32(6, 11)?;
    /// assert_eq!(PowAdic::new(EAdic::new(2, vec![1, 1, 0, 1]), 1), six_adic_11.p_adic(2));
    /// assert_eq!(PowAdic::new(EAdic::new(3, vec![2, 0, 1]), 1), six_adic_11.p_adic(3));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    /// Error if c fails to convert to a [`Composite`] or has no prime factors (i.e. it's one)
    pub fn from_i32<C>(c: C, a: i32) -> AdicResult<Self>
    where A: PrimedFrom<i32>, C: Into<Composite> {
        let c = c.into();
        if c.is_one() {
            Err(AdicError::IllDefined("Nontrivial `MAdic` contain at least one prime.".to_string()))
        } else {
            Ok(Self::new(c.prime_powers().map(move |pp| {
                let (p, power) = (pp.p(), pp.power());
                PowAdic::new(A::primed_from(p, a), power)
            })))
        }
    }

    /// Return the p-adic number associated with the prime `p` in this `MAdic`
    ///
    /// ```
    /// use adic::{num_adic::{MAdic, PowAdic}, EAdic};
    /// let sixteen = MAdic::from_i32(10, 16)?;
    /// assert_eq!(PowAdic::new(EAdic::new(2, vec![0, 0, 0, 0, 1]), 1), sixteen.p_adic(2));
    /// assert_eq!(PowAdic::new(EAdic::new(5, vec![1, 3]), 1), sixteen.p_adic(5));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn p_adic<P>(&self, p: P) -> PowAdic<A>
    where P: Into<Prime> {
        let p = p.into();
        self.pow_adics.get(&p).cloned().unwrap_or(PowAdic::new(A::one(p), 0))
    }

    /// All child `PowAdics`
    pub fn into_pow_adics(self) -> impl Iterator<Item = PowAdic<A>> {
        self.pow_adics.into_values().sorted_by_key(PowAdic::p)
    }
    /// All child `PowAdics`
    pub fn pow_adics(&self) -> impl Iterator<Item = &PowAdic<A>> {
        self.pow_adics.values().sorted_by_key(|ap| ap.p())
    }

    /// The product of all idempotents except `T_p ~ p^carmichael((n / p^k)^infinity)`, as `BigUint`
    pub fn idempotent_excluding<P, C>(p: P, c: C, precision: usize) -> AdicResult<BigUint>
    where P: Into<Prime>, C: Into<Composite> {
        let c = c.into();
        let mdigits = MAdicDigits::idempotent_excluding(p, &c, precision)?;
        let digit_vec = mdigits.digits().map(u8::try_from).collect::<Result<Vec<_>, _>>()?;
        BigUint::from_radix_le(digit_vec.as_slice(), c.value32()).ok_or(
            AdicError::Severe("Error during conversion to BigUint".to_string())
        )
    }

    /// The prime idempotent `T_p ~ p^carmichael((n / p^k)^infinity)`, as `BigUint`
    pub fn prime_idempotent<P, C>(p: P, c: C, precision: usize) -> AdicResult<BigUint>
    where P: Into<Prime>, C: Into<Composite> {
        let c = c.into();
        let mdigits = MAdicDigits::prime_idempotent(p, &c, precision)?;
        let digit_vec = mdigits.digits().map(u8::try_from).collect::<Result<Vec<_>, _>>()?;
        BigUint::from_radix_le(digit_vec.as_slice(), c.value32()).ok_or(
            AdicError::Severe("Error during conversion to BigUint".to_string())
        )
    }

    /// All prime idempotents, as `BigUint`
    pub fn prime_idempotents<C>(c: C, precision: usize) -> AdicResult<Vec<BigUint>>
    where C: Into<Composite> {
        let c = c.into();
        c.clone().primes().map(|p| Self::prime_idempotent(p, c.clone(), precision)).collect()
    }

    /// All prime idempotents, as `BigUint`
    pub fn all_idempotents<C>(c: C, precision: usize) -> AdicResult<Vec<BigUint>>
    where C: Into<Composite> {
        let c = c.into();
        let prime_idemps = c.clone().primes()
            .map(|p| Self::prime_idempotent(p, c.clone(), precision))
            .collect::<Result<Vec<_>, _>>()?;
        let modulus = BigUint::from(c).pow(u32::try_from(precision)?);
        let idempotents = prime_idemps
            .into_iter()
            .powerset()
            .map(|ps| ps.into_iter().product::<BigUint>() % modulus.clone())
            .collect::<Vec<_>>();
        Ok(idempotents)
    }

}


impl MAdic<ZAdic> {

    /// Create approximate mixed adic with no certainty
    ///
    /// ```
    /// # use adic::num_adic::MAdic;
    /// let ten_adic_empty = MAdic::empty(10);
    /// assert_eq!("...._10", ten_adic_empty.to_string());
    /// assert_eq!("...._2", ten_adic_empty.p_adic(2).to_string());
    /// assert_eq!("...._5", ten_adic_empty.p_adic(5).to_string());
    /// ```
    pub fn empty<C>(c: C) -> Self
    where C: Into<Composite> {
        let c = c.into();
        Self::new(c.prime_powers().map(|pp| PowAdic::new(ZAdic::empty(pp.p()), pp.power())))
    }

    /// Create mixed adic from a single signed integer with given precision
    ///
    /// ```
    /// use adic::{num_adic::{MAdic, PowAdic}, ZAdic};
    /// let three = MAdic::approx_from_i32(10, 3, 6)?;
    /// assert_eq!(PowAdic::new(ZAdic::new_approx(2, 24, vec![1, 1]), 1), three.p_adic(2));
    /// assert_eq!(PowAdic::new(ZAdic::new_approx(5, 12, vec![3]), 1), three.p_adic(5));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    /// Error from integer conversion
    pub fn approx_from_i32<C>(c: C, a: i32, precision: usize) -> AdicResult<Self>
    where C: Into<Composite> {
        let c = c.into();
        Ok(Self::new(c.clone().prime_powers().map(
            move |pp| {
                let (p, power) = (pp.p(), pp.power());
                let adjusted_prec = precision * usize::try_from(c.log_ceil(&pp.into()))?;
                Ok(PowAdic::new(ZAdic::primed_from(p, a).into_approximation(adjusted_prec), power))
            }
        ).collect::<AdicResult<Vec<_>>>()?))
    }

}



#[cfg(test)]
mod tests {
    use assertables::assert_matches;

    use crate::{num_adic::PowAdic, traits::HasDigits};

    use super::MAdic;

    #[test]
    fn constructors() {

        let ac = MAdic::new([
            PowAdic::new(zadic_approx!(2, 24, [1]), 1),
            PowAdic::new(zadic_approx!(5, 12, [2]), 1),
        ]);
        assert_eq!(vec![7, 7, 3, 9, 0, 1], ac.digits().collect::<Vec<_>>());
        assert_eq!("...109377._10", ac.to_string());
        assert_eq!(Ok(9), ac.digit(3));
        assert_matches!(ac.digit(6), Err(_));
        let adic2 = ac.p_adic(2);
        assert_eq!(vec![1, 0, 0, 0, 0, 0], adic2.digits().take(6).collect::<Vec<_>>());
        let adic5 = ac.p_adic(5);
        assert_eq!(vec![2, 0, 0, 0, 0, 0], adic5.digits().take(6).collect::<Vec<_>>());

        let ac = MAdic::from_pure_p_adic(30, zadic_approx!(3, 24, [1])).unwrap();
        assert_eq!(vec![10, 3, 1, 17, 24, 7], ac.digits().collect::<Vec<_>>());
        assert_eq!(Ok(17), ac.digit(3));
        assert_matches!(ac.digit(6), Err(_));
        let adic2 = ac.p_adic(2);
        assert_eq!(None, adic2.digits().next());
        let adic3 = ac.p_adic(3);
        assert_eq!(vec![1, 0, 0, 0, 0, 0], adic3.digits().take(6).collect::<Vec<_>>());
        let adic5 = ac.p_adic(5);
        assert_eq!(None, adic5.digits().next());

        let ac = MAdic::approx_from_i32(10, 2, 6).unwrap();
        assert_eq!(vec![2, 0, 0, 0, 0, 0], ac.digits().collect::<Vec<_>>());
        assert_eq!(Ok(0), ac.digit(3));
        assert_matches!(ac.digit(6), Err(_));
        let ac = MAdic::approx_from_i32(10, 10, 6).unwrap();
        assert_eq!(vec![0, 1, 0, 0, 0, 0], ac.digits().collect::<Vec<_>>());
        let ac = MAdic::approx_from_i32(10, -1, 6).unwrap();
        assert_eq!(vec![9, 9, 9, 9, 9, 9], ac.digits().collect::<Vec<_>>());

        let ac = MAdic::new([
            PowAdic::new(qadic!(zadic_approx!(2, 25, [1]), -5), 1),
            PowAdic::new(qadic!(zadic_approx!(5, 12, [2]), -2), 1),
        ]);
        assert_eq!(vec![7, 7, 3, 9, 0, 1], ac.digits().collect::<Vec<_>>());
        assert_eq!("...10937.7_10", ac.to_string());
        assert_eq!(Ok(9), ac.digit(2));
        assert_matches!(ac.digit(5), Err(_));
        let adic2 = ac.p_adic(2);
        assert_eq!(vec![1, 0, 0, 0, 0, 0], adic2.digits().take(6).collect::<Vec<_>>());
        let adic5 = ac.p_adic(5);
        assert_eq!(vec![2, 0, 0, 0, 0, 0], adic5.digits().take(6).collect::<Vec<_>>());

    }

    #[test]
    fn test_timing() {
        // let precision = 10;
        let precision = 100;
        // let precision = 1000;
        // let precision = 10000;
        let _ac = MAdic::new([
            PowAdic::new(zadic_approx!(2, 5 * precision, [1]), 1),
            PowAdic::new(zadic_approx!(5, 2 * precision, [2]), 1),
        ]);
        // println!("{}", _ac);
    }

}
