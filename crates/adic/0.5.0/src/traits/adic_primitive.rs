use std::iter::{once, repeat_n};
use crate::{
    divisible::{Prime, PrimePower},
    local_num::{LocalOne, LocalZero},
    UAdic,
};


/// An adic number primitive, associated to a prime `p` and with semiring arithmetic
///
/// Structs implementing this trait tend to represent adic numbers.
/// Each adic number is associated with a prime, p.
///
/// There are several interesting spaces in the adics, so this trait is pretty light.
/// See e.g. [`AdicInteger`](crate::traits::AdicInteger) for more functionality.
///
/// <https://en.wikipedia.org/wiki/P-adic_number>
pub trait AdicPrimitive
where Self: Clone
    + std::ops::Add<Self, Output=Self>
    + std::ops::Mul<Self, Output=Self>
    + LocalZero + LocalOne
    + From<UAdic> {

    /// Prime for this adic
    fn p(&self) -> Prime;

    // Constructors

    /// Create the `zero` adic number
    ///
    /// ```
    /// # use adic::{traits::AdicPrimitive, UAdic};
    /// assert_eq!("0._5", UAdic::zero(5).to_string());
    /// ```
    fn zero<P>(p: P) -> Self
    where P: Into<Prime>;

    /// Create the `one` adic number
    ///
    /// ```
    /// # use adic::{traits::AdicPrimitive, UAdic};
    /// assert_eq!("1._5", UAdic::one(5).to_string());
    /// ```
    fn one<P>(p: P) -> Self
    where P: Into<Prime>;

    /// Create the adic number associated with its prime
    ///
    /// ```
    /// # use adic::{traits::AdicPrimitive, UAdic};
    /// assert_eq!("10._5", UAdic::from_prime(5).to_string());
    fn from_prime<P>(p: P) -> Self
    where P: Into<Prime> {
        let p = p.into();
        Self::from(UAdic::new(p, vec![0, 1]))
    }

    /// Create the adic number associated with a power of its prime
    ///
    /// ```
    /// # use adic::{divisible::PrimePower, traits::AdicPrimitive, UAdic};
    /// assert_eq!("1000._5", UAdic::from_prime_power(PrimePower::from((5, 3))).to_string());
    fn from_prime_power<PP>(pp: PP) -> Self
    where PP: Into<PrimePower> {
        let pp = pp.into();
        let p = pp.p();
        let power = usize::try_from(pp.power()).expect("u32 -> usize conversion");
        let d = repeat_n(0, power).chain(once(1)).collect();
        Self::from(UAdic::new(p, d))
    }

    // `MaybePrime` has become less important.
    // Consider removing these along with MaybePrimed if it remains less useful.

    // /// Change into a `MaybePrime`
    // fn into_maybe<N>(self) -> MaybePrimed<Self, N>
    // where N: RingBacking<Self> {
    //     MaybePrimed::Primed(self)
    // }

    // /// Change into a `MaybePrime`
    // fn maybe<N>(&self) -> MaybePrimed<Self, N>
    // where N: RingBacking<Self> {
    //     MaybePrimed::Primed(self.clone())
    // }

}
