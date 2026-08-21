use std::ops::{Add, Mul};
use num::traits::NumOps;



/// Returns an additive identity element for `self`
///```
/// # use adic::{local_num::LocalZero, UAdic};
/// let a = UAdic::new(5, vec![0, 1, 2, 3, 4]);
/// let b = a.local_zero();
/// assert_eq!("0._5", b.to_string());
/// assert_eq!(a.clone() + b, a);
///```
pub trait LocalZero: Sized + Add<Output = Self> {

    #[must_use]
    /// Returns a zero local to `self`
    fn local_zero(&self) -> Self;

    /// Checks whether or not `self` is equivalent to its local zero
    fn is_local_zero(&self) -> bool;

    /// Sets the object equal to its local zero
    fn set_local_zero(&mut self) {
        *self = self.local_zero();
    }

}


/// Returns a multiplicative identity element for `self`
///```
/// # use adic::{local_num::LocalOne, UAdic};
/// let a = UAdic::new(5, vec![0, 1, 2, 3, 4]);
/// let b = a.local_one();
/// assert_eq!("1._5", b.to_string());
/// assert_eq!(a.clone() * b, a);
///```
pub trait LocalOne: Sized + Mul<Output = Self> {

    #[must_use]
    /// Returns a one local to `self`
    fn local_one(&self) -> Self;

    /// Checks whether or not `self` is equivalent to its local one
    fn is_local_one(&self) -> bool;

    /// Sets the object equal to its local one
    fn set_local_one(&mut self) {
        *self = self.local_one();
    }

}


/// The base trait for numeric types, covering **local** 0 and 1 values,
///  comparisons, basic numeric operations, and string conversion
pub trait LocalNum: PartialEq + LocalZero + LocalOne + NumOps {

    /// String conversion error
    type FromStrRadixErr;

    /// Convert from a string and radix (typically 2..=36)
    fn from_str_radix(
        s: &str,
        radix: u32,
    ) -> Result<Self, Self::FromStrRadixErr>;

}


/// A [`LocalNum`] that allows division, remainder, GCD, and LCM
///
/// This is something like a [Euclidean domain](https://en.wikipedia.org/wiki/Euclidean_domain),
///  but do not expect it to be mathematically equivalent.
pub trait LocalInteger: Sized + Eq + LocalNum {

    #[must_use]
    /// Greatest Common Divisor (GCD)
    fn gcd(&self, other: &Self) -> Self;

    #[must_use]
    /// Lowest Common Multiple (LCM)
    fn lcm(&self, other: &Self) -> Self;

    #[must_use]
    /// Greatest Common Divisor (GCD) and Lowest Common Multiple (LCM) together
    ///
    /// Potentially more efficient than calling `gcd` and `lcm`
    /// individually for identical inputs.
    fn gcd_lcm(&self, other: &Self) -> (Self, Self) {
        (self.gcd(other), self.lcm(other))
    }

    /// Returns `true` if `self` is a multiple of `other`
    fn is_multiple_of(&self, other: &Self) -> bool;


    #[must_use]
    /// Simultaneous truncated integer division and modulus,
    /// returns `(quotient, remainder)`
    fn div_rem(self, other: Self) -> (Self, Self)
    where Self: Clone {
        (self.clone() / other.clone(), self % other)
    }

    /// Decrements `self` by one
    fn dec(&mut self)
    where Self: Clone {
        *self = self.clone() - self.local_one();
    }

    /// Increments `self` by one
    fn inc(&mut self)
    where Self: Clone {
        *self = self.clone() + self.local_one();
    }

}
