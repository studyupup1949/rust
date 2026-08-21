use std::{
    fmt::Debug,
    hash::Hash,
};
use num::Rational32;
use crate::{IAdic, RAdic, Prime, UAdic};


/// An adic number primitive, associated to a prime `p` and with semiring arithmetic
///
/// Structs implementing this trait represent adic numbers.
/// Each adic number is associated with a prime, p.
///
/// There are several interesting spaces in the adics, so this trait is pretty light.
/// See more specific traits for more functionality:
/// - [`AdicInteger`](crate::AdicInteger)
/// - [`AdicFraction`](crate::AdicFraction)
///
/// <https://en.wikipedia.org/wiki/P-adic_number>
pub trait AdicNumber
where Self: Debug + Clone + PartialEq + Eq + Hash + From<UAdic>
    + std::ops::Add<Self, Output=Self>
    + std::ops::Mul<Self, Output=Self> + std::ops::Mul<u32, Output=Self> {

    // Constructors

    /// Create the `zero` adic number
    ///
    /// ```
    /// # use adic::{AdicNumber, UAdic};
    /// assert_eq!("0._5", UAdic::zero(5).to_string());
    /// ```
    fn zero<P>(p: P) -> Self
    where P: Into<Prime>;

    /// Create the `one` adic number
    ///
    /// ```
    /// # use adic::{AdicNumber, UAdic};
    /// assert_eq!("1._5", UAdic::one(5).to_string());
    /// ```
    fn one<P>(p: P) -> Self
    where P: Into<Prime>;

    /// Create `AdicNumber` from u32
    ///
    /// ```
    /// # use adic::{AdicNumber, IAdic, RAdic, UAdic, ZAdic};
    /// assert_eq!("123._5", UAdic::from_u32(5, 38).to_string());
    /// assert_eq!("123._5", IAdic::from_u32(5, 38).to_string());
    /// assert_eq!("123._5", RAdic::from_u32(5, 38).to_string());
    /// assert_eq!("123._5", ZAdic::from_u32(5, 38).to_string());
    /// ```
    fn from_u32<P>(p: P, mut n: u32) -> Self
    where P: Into<Prime> {
        let p = p.into();
        let mut digits = vec![];
        while n != 0 {
            digits.push(n % p);
            n = n / p;
        }
        Self::from(UAdic::new(p, digits))
    }

    /// Prime for this adic
    fn p(&self) -> Prime;

    /// Test if it is the zero adic number
    ///
    /// ```
    /// # use adic::{uadic, AdicNumber};
    /// assert!(uadic!(5, []).is_zero());
    /// assert!(!uadic!(5, [2, 3, 1, 2, 3, 1]).is_zero());
    /// ```
    fn is_zero(&self) -> bool {
        *self == Self::zero(self.p())
    }

}


/// An adic number primitive, associated to a prime `p` and with ring arithmetic
///
/// Structs implementing this trait are adic number that can represent signed adics, e.g. [`IAdic`](crate::IAdic).
/// For the most part, these are the same as [`AdicNumber`](crate::AdicNumber).
/// The main difference is that `SignedAdicNumber` can be transformed between e.g. `IAdic` and `i32`.
pub trait SignedAdicNumber: AdicNumber
    + std::ops::Neg<Output=Self>
    + std::ops::Sub<Self, Output=Self> {

    /// Create `SignedAdicNumber` from i32
    ///
    /// ```
    /// # use adic::{IAdic, RAdic, SignedAdicNumber, ZAdic};
    /// assert_eq!("(4)23._5", IAdic::from_i32(5, -12).to_string());
    /// assert_eq!("(4)23._5", RAdic::from_i32(5, -12).to_string());
    /// assert_eq!("(4)23._5", ZAdic::from_i32(5, -12).to_string());
    /// ```
    fn from_i32<P>(p: P, n: i32) -> Self
    where P: Into<Prime>;

}


impl<T> SignedAdicNumber for T
where T: AdicNumber + From<IAdic>
    + std::ops::Neg<Output=Self>
    + std::ops::Sub<Self, Output=Self> {

    fn from_i32<P>(p: P, n: i32) -> Self
    where P: Into<Prime> {
        let p = p.into();
        let ia = if n.is_negative() {
            -IAdic::from_u32(p, n.unsigned_abs())
        } else {
            IAdic::from_u32(p, n.unsigned_abs())
        };
        Self::from(ia)
    }

}

/// An adic number primitive, associated to a prime `p` and that can represent rational numbers
///
/// Structs implementing this trait are adic number that can represent adics corresponding to rational numbers, e.g. [`RAdic`](crate::RAdic).
/// For the most part, these are the same as [`AdicNumber`](crate::AdicNumber).
/// The main difference is that `RationalAdicNumber` can be transformed between e.g. `RAdic` and `Rational32`.
pub trait RationalAdicNumber: AdicNumber {

    /// Create `RationalAdicNumber` from `Rational32`
    ///
    /// ```
    /// # use adic::{RAdic, RationalAdicNumber};
    /// # use num::Rational32;
    /// let n = Rational32::new(1, 4);
    /// assert_eq!("(3)4._5", RAdic::from_rational(5, n).to_string());
    /// ```
    fn from_rational<P>(p: P, n: Rational32) -> Self
    where P: Into<Prime>;

}


impl<T> RationalAdicNumber for T
where T: AdicNumber + SignedAdicNumber + From<RAdic> {
    fn from_rational<P>(p: P, n: Rational32) -> Self
    where P: Into<Prime> {
        let p = p.into();
        let ra = RAdic::from_i32(p, *n.numer()) / RAdic::from_i32(p, *n.denom());
        Self::from(ra.rexact().expect("Trying to exactly invert an approximate number"))
    }

}
