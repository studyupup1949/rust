use std::collections::HashMap;
use num::One;

use crate::{
    AdicApproximate, AdicError, AdicInteger, AdicNumber, AdicPower, AdicResult, AdicValuation, AdicValuationRing,
    Composite, HasDigits, Prime, SignedAdicNumber, ZAdic,
};



#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// An adic with a composite as base
///
/// Used to represent base-`n` left-infinite numbers.
/// Splits canonically into `p`-adic components for each prime factor `p` of `n`.
/// Internally, these are stored as [`AdicPower`] components for each [`Prime`]
///  in the base [`Composite`]'s factorization.
///
/// This and the [`AdicPower`](crate::AdicPower) struct are composite adic structures,
///  as opposed to prime-based structs like [`UAdic`](crate::UAdic) or [`ZAdic`](crate::ZAdic).
/// `AdicComposite` in fact is not a field but a ring, so operations like division make less sense.
/// Expect fewer features generally in these composite structures.
///
/// <div class="warning">
/// This struct is not fully stabilized.
/// Even operations as simple as construction and Display need to be done carefully.
/// Use with approximate ZAdic for best results
/// </div>
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use adic::{AdicComposite, HasDigits, Prime};
/// let three = AdicComposite::approx_from_i32(10, 3, 6)?;
/// assert_eq!("...000003._10", three.to_string());
/// assert_eq!(vec![Prime::from(2), Prime::from(5)], three.base().factors().collect::<Vec<_>>());
/// let two_adic_part = three.p_adic(2);
/// let five_adic_part = three.p_adic(5);
/// assert_eq!("...000000000000000000000000000011._2", two_adic_part.to_string());
/// assert_eq!("...000000000003._5", five_adic_part.to_string());
/// let pure_two_ten_adic = AdicComposite::from_pure_p_adic(10, two_adic_part.adic_ref().clone())?;
/// let pure_five_ten_adic = AdicComposite::from_pure_p_adic(10, five_adic_part.adic_ref().clone())?;
/// let added = pure_two_ten_adic.clone() + pure_five_ten_adic.clone();
/// let multiplied = pure_two_ten_adic.clone() * pure_five_ten_adic.clone();
/// assert_eq!("...000003._10", added.to_string());
/// assert_eq!("0._10", multiplied.to_string());
/// # Ok(()) }
/// ```
pub struct AdicComposite<A>
where A: AdicApproximate + AdicNumber {
    // Perhaps store the "prime idempotent"s in the p_adics/p_adic_data, or lazy static
    pub (super) p_adics: HashMap<Prime, AdicPower<A>>,
}


impl<A> AdicComposite<A>
where A: AdicApproximate + AdicNumber {

    /// Create a base-`n` adic number
    ///
    /// ```
    /// use adic::{AdicComposite, apow, zadic_approx};
    /// let two_adic_four = apow!(zadic_approx!(2, 30, [0, 0, 1]), 1);
    /// let five_adic_four = apow!(zadic_approx!(5, 12, [4]), 1);
    /// let ten_adic_four = AdicComposite::new([two_adic_four, five_adic_four]);
    /// assert_eq!("...000004._10", ten_adic_four.to_string());
    /// assert_eq!("...000016._10", (ten_adic_four.clone() * ten_adic_four.clone()).to_string());
    /// ```
    pub fn new(adics: impl IntoIterator<Item=AdicPower<A>>) -> Self {
        let p_adics = adics.into_iter().map(
            |a| (a.p(), a)
        ).collect::<HashMap<_, _>>();
        AdicComposite {
            p_adics,
        }
    }

    /// Create a `AdicComposite` equivalent to `adic` when projected to the `p`-adics and to `0` otherwise
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use adic::{AdicComposite, apow, zadic_approx};
    /// let two_adic_four = zadic_approx!(2, 30, [0, 0, 1]);
    /// let ten_adic_pure_two_four = AdicComposite::from_pure_p_adic(10, two_adic_four)?;
    /// assert_eq!("...562500._10", ten_adic_pure_two_four.to_string());
    /// let five_adic_four = zadic_approx!(5, 12, [4]);
    /// let ten_adic_pure_five_four = AdicComposite::from_pure_p_adic(10, five_adic_four)?;
    /// assert_eq!("...562500._10", ten_adic_pure_two_four.to_string());
    /// let ten_adic_four = ten_adic_pure_two_four + ten_adic_pure_five_four;
    /// assert_eq!("...000004._10", ten_adic_four.to_string());
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    /// Error if c fails to convert to a [`Composite`] or does not contain `adic`'s prime as a factor
    pub fn from_pure_p_adic<C>(c: C, adic: A) -> AdicResult<Self>
    where C: TryInto<Composite, Error = AdicError> {
        let c = c.try_into()?;
        if c.has_prime(adic.p()) {
            Ok(Self::new(c.prime_powers().map(move |pp| {
                let (p, power) = (pp.p(), pp.power());
                if p == adic.p() {
                    AdicPower::new(adic.clone(), power)
                } else {
                    AdicPower::new(A::zero(p), power)
                }
            })))
        } else {
            Err(AdicError::IllDefined("`AdicComposite` must contain the prime of `adic`.".to_string()))
        }
    }

    /// Create mixed adic from a single signed integer
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use adic::{AdicComposite, apow, iadic_pos};
    /// let six_adic_11 = AdicComposite::from_i32(6, 11)?;
    /// assert_eq!(apow!(iadic_pos!(2, [1, 1, 0, 1]), 1), six_adic_11.p_adic(2));
    /// assert_eq!(apow!(iadic_pos!(3, [2, 0, 1]), 1), six_adic_11.p_adic(3));
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    /// Error if c fails to convert to a [`Composite`] or has no prime factors (i.e. it's one)
    pub fn from_i32<C>(c: C, a: i32) -> AdicResult<Self>
    where A: SignedAdicNumber, C: TryInto<Composite, Error = AdicError> {
        let c = c.try_into()?;
        if c.is_one() {
            Err(AdicError::IllDefined("Nontrivial `AdicComposite` contain at least one prime.".to_string()))
        } else {
            Ok(Self::new(c.prime_powers().map(move |pp| {
                let (p, power) = (pp.p(), pp.power());
                AdicPower::new(A::from_i32(p, a), power)
            })))
        }
    }

    /// Return the p-adic number associated with the prime `p` in this `AdicComposite`
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use adic::{AdicComposite, apow, iadic_pos};
    /// let sixteen = AdicComposite::from_i32(10, 16)?;
    /// assert_eq!(apow!(iadic_pos!(2, [0, 0, 0, 0, 1]), 1), sixteen.p_adic(2));
    /// assert_eq!(apow!(iadic_pos!(5, [1, 3]), 1), sixteen.p_adic(5));
    /// # Ok(()) }
    /// ```
    pub fn p_adic<P>(&self, p: P) -> AdicPower<A>
    where P: Into<Prime> {
        let p = p.into();
        self.p_adics.get(&p).cloned().unwrap_or(AdicPower::new(A::one(p), 0))
    }

}


impl AdicComposite<ZAdic> {

    /// Create approximate mixed adic with no certainty
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use adic::{AdicComposite, apow, iadic_pos};
    /// let ten_adic_empty = AdicComposite::empty(10)?;
    /// assert_eq!("...._10", ten_adic_empty.to_string());
    /// assert_eq!("...._2", ten_adic_empty.p_adic(2).to_string());
    /// assert_eq!("...._5", ten_adic_empty.p_adic(5).to_string());
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    /// Error if c fails to convert to a [`Composite`]
    pub fn empty<C>(c: C) -> AdicResult<Self>
    where C: TryInto<Composite, Error = AdicError> {
        let c = c.try_into()?;
        Ok(Self::new(c.prime_powers().map(|pp| AdicPower::new(ZAdic::empty(pp.p()), pp.power()))))
    }

    /// Create mixed adic from a single signed integer with given precision
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use adic::{AdicComposite, apow, zadic_approx};
    /// let three = AdicComposite::approx_from_i32(10, 3, 6)?;
    /// assert_eq!(apow!(zadic_approx!(2, 30, [1, 1]), 1), three.p_adic(2));
    /// assert_eq!(apow!(zadic_approx!(5, 12, [3]), 1), three.p_adic(5));
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    /// Error if c fails to convert to a [`Composite`]
    pub fn approx_from_i32<C>(c: C, a: i32, precision: usize) -> AdicResult<Self>
    where C: TryInto<Composite, Error = AdicError> {
        let c = c.try_into()?;
        Ok(Self::new(c.clone().prime_powers().map(
            move |pp| {
                let (p, power) = (pp.p(), pp.power());
                let adjusted_prec = precision * usize::try_from(c.without_p(p))?;
                Ok(AdicPower::new(ZAdic::from_i32(p, a).into_approximation(adjusted_prec), power))
            }
        ).collect::<Result<Vec<_>, AdicError>>()?))
    }

}


impl<A> AdicApproximate for AdicComposite<A>
where A: AdicNumber + AdicApproximate {

    fn certainty(&self) -> AdicValuation<A::DigitIndex> {
        let base = self.base();
        self.p_adics.values().map(|ap| {
            if let AdicValuation::Finite(nd) = ap.adic_ref().certainty() {
                let base_wo_p = base.without_p(ap.p());
                let base_wo_p = usize::try_from(base_wo_p).expect("composite conversion -> usize");
                let base_wo_p = A::DigitIndex::try_from_usize(base_wo_p).expect("base conversion to valuation ring");
                let cert = nd / base_wo_p;
                cert.into()
            } else {
                AdicValuation::PosInf
            }
        }).min().unwrap_or(AdicValuation::PosInf)
    }

}



#[cfg(test)]
mod tests {

    use crate::{qadic, zadic_approx, AdicPower, HasDigits};

    use super::AdicComposite;

    #[test]
    fn constructors() {

        let ac = AdicComposite::new([
            AdicPower::new(zadic_approx!(2, 30, [1]), 1),
            AdicPower::new(zadic_approx!(5, 12, [2]), 1),
        ]);
        assert_eq!(vec![7, 7, 3, 9, 0, 1], ac.digits().collect::<Vec<_>>());
        assert_eq!(Ok(9), ac.digit(3));
        assert!(matches!(ac.digit(6), Err(_)));
        let adic2 = ac.p_adic(2);
        assert_eq!(vec![1, 0, 0, 0, 0, 0], adic2.digits().take(6).collect::<Vec<_>>());
        let adic5 = ac.p_adic(5);
        assert_eq!(vec![2, 0, 0, 0, 0, 0], adic5.digits().take(6).collect::<Vec<_>>());

        let ac = AdicComposite::from_pure_p_adic(30, zadic_approx!(3, 60, [1])).unwrap();
        assert_eq!(vec![10, 3, 1, 17, 24, 7], ac.digits().collect::<Vec<_>>());
        assert_eq!(Ok(17), ac.digit(3));
        assert!(matches!(ac.digit(6), Err(_)));
        let adic2 = ac.p_adic(2);
        assert_eq!(None, adic2.digits().next());
        let adic3 = ac.p_adic(3);
        assert_eq!(vec![1, 0, 0, 0, 0, 0], adic3.digits().take(6).collect::<Vec<_>>());
        let adic5 = ac.p_adic(5);
        assert_eq!(None, adic5.digits().next());

        let ac = AdicComposite::approx_from_i32(10, 2, 6).unwrap();
        assert_eq!(vec![2, 0, 0, 0, 0, 0], ac.digits().collect::<Vec<_>>());
        assert_eq!(Ok(0), ac.digit(3));
        assert!(matches!(ac.digit(6), Err(_)));
        let ac = AdicComposite::approx_from_i32(10, 10, 6).unwrap();
        assert_eq!(vec![0, 1, 0, 0, 0, 0], ac.digits().collect::<Vec<_>>());
        let ac = AdicComposite::approx_from_i32(10, -1, 6).unwrap();
        assert_eq!(vec![9, 9, 9, 9, 9, 9], ac.digits().collect::<Vec<_>>());

        let ac = AdicComposite::new([
            AdicPower::new(qadic!(zadic_approx!(2, 30, [1]), -5), 1),
            AdicPower::new(qadic!(zadic_approx!(5, 12, [2]), -2), 1),
        ]);
        assert_eq!(vec![7, 7, 3, 9, 0, 1], ac.digits().collect::<Vec<_>>());
        assert_eq!(Ok(9), ac.digit(2));
        assert!(matches!(ac.digit(5), Err(_)));
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
        let ac = AdicComposite::new([
            AdicPower::new(zadic_approx!(2, 5 * precision, [1]), 1),
            AdicPower::new(zadic_approx!(5, 2 * precision, [2]), 1),
        ]);
        println!("{}", ac);
    }

}
