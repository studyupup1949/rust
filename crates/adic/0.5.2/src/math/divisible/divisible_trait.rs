use std::{
    iter::repeat_n,
    ops::Range,
};
use itertools::{Itertools, MultiProduct};
use num::{BigUint, One};

use super::{Composite, Prime, PrimePower};


/// Trait for numbers that are composted of primes.
/// Implementors cannot be zero.
///
/// Implementors:
/// - [`Prime`]
/// - [`PrimePower`]
/// - [`Composite`]
pub trait Divisible where Self: Clone + Into<Composite> {

    /// Is `other` exactly divisible by `self`
    fn divides<D>(&self, other: &D) -> bool
    where D: Divisible;

    /// Is `self` exactly divisible by `other`
    fn is_divisible_by<D>(&self, other: &D) -> bool
    where D: Divisible {
        other.divides(self)
    }


    /// Integer value of the `Divisible`
    fn value32(&self) -> u32 {
        Into::<Composite>::into(self.clone()).prime_powers().map(u32::from).product()
    }

    /// Integer value of the `Divisible`
    fn value_big(&self) -> BigUint {
        Into::<Composite>::into(self.clone()).prime_powers().map(BigUint::from).product()
    }

    /// Integer value minus 1
    fn m1(&self) -> u32 { self.value32() - 1 }

    /// Digit range `[0, val) = 0..val`
    fn digit_range(&self) -> Range<u32> {
        0..self.value32()
    }


    /// Display digit, mod this divisible.
    /// If divisible less than 37, use alphanumeric digits 0-9a-z.
    /// If greater, use integers surrounded by square brackets.
    fn display_digit(&self, d: u32) -> String {
        let radix = self.value32();
        let dstr = if radix < 37 {
            char::from_digit(d, radix).map(|s| s.to_string())
        } else if d < radix {
            Some(format!("[{d}]"))
        } else {
            None
        };
        dstr.unwrap_or("BAD_DIGIT({d})".to_string())
    }

    /// Display digit zero, mod this divisible.
    /// If divisible less than 37, just 0, otherwise surround by square brackets.
    fn display_zero(&self) -> String {
        let radix = self.value32();
        if radix < 37 { "0".to_string() } else { "[0]".to_string() }
    }


    /// Multiple digit range [000, 1000) = [0..val, 0..val, 0..val]
    ///
    /// <div class="warning">
    ///
    /// This could easily hang; use judiciously
    ///
    /// </div>
    fn multi_digit_range(&self, num_digits: usize) -> MultiProduct<std::ops::Range<u32>> {
        repeat_n(self.digit_range(), num_digits).multi_cartesian_product()
    }


    /// Carmichael function of this `Divisible`
    fn carmichael(&self) -> Composite {
        self.carmichael_pow(1)
    }

    /// Carmichael function of a power of this `Divisible`
    fn carmichael_pow(&self, cpower: u32) -> Composite {
        let c = self.clone().into();
        c.carmichael_iter(cpower).fold(
            Composite::one(),
            |c1, c2| Composite::lcm(&c1, &c2)
        )
    }

}


impl Divisible for Composite {
    fn divides<D>(&self, other: &D) -> bool
    where D: Divisible {
        let cother: Composite = other.clone().into();
        let val = self.prime_powers().all(|pp| cother.prime_power(pp.p()).power() >= pp.power());
        val
    }
}
impl Divisible for Prime {
    fn divides<D>(&self, other: &D) -> bool
    where D: Divisible {
        let cself: Composite = (*self).into();
        cself.divides(other)
    }
}
impl Divisible for PrimePower {
    fn divides<D>(&self, other: &D) -> bool
    where D: Divisible {
        let cself: Composite = (*self).into();
        cself.divides(other)
    }
}
