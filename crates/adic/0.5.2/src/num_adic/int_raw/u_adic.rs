use itertools::Itertools;
use num::{
    traits::Pow,
    BigUint,
};
use crate::{
    divisible::{Divisible, Prime},
    error::{validate_digits_mod_p, AdicError, AdicResult},
    traits::{AdicInteger, AdicPrimitive, CanTruncate, HasDigitDisplay, HasDigits},
};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Arbitrary precision unsigned integer
///
/// An [`AdicInteger`](crate::traits::AdicInteger).
/// The struct holds a finite list of digits, from the "zero" digit place to the max.
/// This is the most basic Adic struct, just consisting of a finite number of digits.
/// With this, you can represent exactly the non-negative (true) integers, Z_+.
///
/// ```
/// # use adic::{traits::AdicPrimitive, UAdic};
/// assert_eq!("4321._5", UAdic::new(5, vec![1, 2, 3, 4]).to_string());
/// let two = UAdic::new(5, vec![2]);
/// assert_eq!(Ok(2), two.u32_value());
/// let five = UAdic::new(5, vec![0, 1]);
/// assert_eq!(Ok(7), (two.clone() + five.clone()).u32_value());
/// assert_eq!(Ok(10), (two.clone() * five.clone()).u32_value());
/// ```
///
/// This representation EXACTLY matches the (base-p) digits for a non-negative real integer.
/// `2 = 2._5, 123 (base 5) = 123._5`
/// You can perform the same arithmetic on these numbers.
/// However, just like e.g. `u32`, these numbers do not subtract or divide well.
/// Instead, look to exact adic integers for subtraction and some division: [`EAdic`](crate::EAdic).
/// Or full adic **numbers** for a field: [`QAdic`](crate::QAdic).
///
/// # Panics
/// Many methods will panic if a provided prime `p` is not prime or digits are outside of `[0, p)`.
pub struct UAdic {
    /// Adic prime
    pub (super) p: Prime,
    /// Adic digits, each 0 to p-1
    pub (super) d: Vec<u32>,
}


impl UAdic {

    /// Create an adic number with the given digits; truncates if zero
    pub fn new<P>(p: P, init_digits: Vec<u32>) -> Self
    where P: Into<Prime> {

        let p = p.into();
        validate_digits_mod_p(p, &init_digits);

        let mut s = Self {
            p,
            d: init_digits,
        };
        s.truncate_zeros();
        s

    }

    /// The (finite) number of digits in `UAdic` (avoiding infinity from [`num_digits`](Self::num_digits) method)
    pub fn finite_num_digits(&self) -> usize {
        self.d.len()
    }

    /// Consume `UAdic` and get the digits vector
    ///
    /// ```
    /// # use adic::UAdic;
    /// let u = UAdic::new(5, vec![1, 2, 3, 4, 0]);
    /// assert_eq!(vec![1, 2, 3, 4], u.digits_vec());
    /// ```
    pub fn digits_vec(self) -> Vec<u32> {
        self.digits().collect()
    }

    /// Push another digit onto the end of the number
    pub fn push_digit(&mut self, digit: u32) {

        validate_digits_mod_p(self.p, &[digit]);

        if digit != 0 { self.d.push(digit) }

    }

    /// Pop (nonzero) digit off the end of the number
    pub fn pop_digit(&mut self) -> Option<u32> {
        let d = self.d.pop();
        self.truncate_zeros();
        d
    }

    /// Push digits onto the end of the number
    pub fn extend_digits(&mut self, additional: &[u32]) {

        validate_digits_mod_p(self.p, additional);

        self.d.extend(additional);
        self.truncate_zeros();

    }

    /// The (pseudo-)remainder and (pseudo-)quotient with respect to `(p^n-1)`.
    /// The caveat is that `(p^n-1)` itself will return `(p^n-1)` remainder and `0` quotient.
    /// This method is used in [`EAdic`](crate::EAdic) multiplication.
    pub (crate) fn pseudo_pn_minus_1_rem_quot(self, n: usize) -> (Self, Self) {

        // As a note, there's a trick for calculating quotient and mod of a UAdic with respect to p^n-1
        // You split at n, the (p^n) quotient is a partial quotient
        //  and you ADD the quotient to the remainder for a trial remainder
        // If the trial remainder grows bigger than p^n, you repeat, accumulating quotients
        // Once the trial remainder is below p^n, your accumulated quotient is (almost!) the quotient
        // The big caveat is that p^n-1 is left untouched at the end, instead of giving mod -> 0

        let (mut acc_quotient, mut trial_remainder) = (UAdic::zero(self.p), self);
        while trial_remainder.d.len() > n {
            let (new_remainder, new_quotient) = trial_remainder.split(n);
            acc_quotient = acc_quotient + new_quotient.clone();
            trial_remainder = new_remainder + new_quotient;
        }
        (trial_remainder, acc_quotient)

    }

    /// The natural number value of the number, e.g. 5-adic 123 is 25+10+3=38
    ///
    /// Warning: This can overflow; use [`bigint_value`](Self::bigint_value) if unsure
    ///
    /// ```
    /// # use adic::UAdic;
    /// assert_eq!(Ok(38), UAdic::new(5, vec![3, 2, 1]).u32_value());
    /// ```
    pub fn u32_value(&self) -> AdicResult<u32> {
        // sum (d * p^k)
        let mut ans = 0u32;
        for (d, k) in self.digits().zip(0..) {
            let trial = u32::from(self.p).checked_pow(k).and_then(|pn| d.checked_mul(pn)).and_then(|n| ans.checked_add(n));
            if let Some(t) = trial {
                ans = t;
            } else {
                Err(AdicError::TryFromIntError)?;
            }
        }
        Ok(ans)
    }

    /// The bigint representation for the natural number value of the number ([`u32_value`](`Self::u32_value`))
    ///
    /// ```
    /// # use num::BigUint;
    /// # use adic::UAdic;
    /// assert_eq!(BigUint::from(38u32), UAdic::new(5, vec![3, 2, 1]).bigint_value());
    /// ```
    pub fn bigint_value(&self) -> BigUint {
        self.digits()
            .zip(0u32..)
            .map(|(d, k)| BigUint::from(d) * BigUint::from(self.p).pow(k))
            .sum()
    }


    // Truncate zeros so there should never be leading zeros for a UAdic
    fn truncate_zeros(&mut self) {
        while let Some(0) = self.d.last() {
            self.d.pop();
        }
    }

}


impl AdicPrimitive for UAdic {

    fn zero<P>(p: P) -> Self
    where P: Into<Prime> {
        Self::new(p, vec![])
    }
    fn one<P>(p: P) -> Self
    where P: Into<Prime> {
        Self::new(p, vec![1])
    }
    fn p(&self) -> Prime {
        self.p
    }

}


impl AdicInteger for UAdic { }
impl HasDigitDisplay for UAdic {
    type DigitDisplay = String;
    fn digit_display(&self) -> String {
        let p = self.p();
        let ds = self.digits().map(|d| p.display_digit(d)).collect::<Vec<_>>();
        ds.into_iter().rev().join("")
    }
}

#[cfg(test)]
mod tests {
    use num::{rational::Ratio, traits::Pow};
    use crate::{
        error::AdicError,
        local_num::LocalZero,
        normed::{Normed, UltraNormed, Valuation},
        traits::{CanApproximate, CanTruncate, HasApproximateDigits},
        Variety, ZAdic,
    };
    use super::{AdicInteger, UAdic};

    use crate::num_adic::test_util::u::*;


    #[test]
    fn strips_zeros() {
        let strips_zeros = uadic!(5, [2, 0, 0, 0, 0]);
        assert_eq!(uadic!(5, [2]), strips_zeros);
        assert_eq!(one_twenty_five().certainty(), Valuation::PosInf);
    }

    #[test]
    fn u32_value() {
        assert_eq!(Ok(1), uadic!(5, [1]).u32_value());
        assert_eq!(Ok(2), uadic!(5, [2]).u32_value());
        assert_eq!(Ok(6), uadic!(5, [1, 1]).u32_value());
        assert_eq!(Ok(126), uadic!(5, [1, 0, 0, 1]).u32_value());
        assert_eq!(Ok(124), uadic!(5, [4, 4, 4]).u32_value());
        assert_eq!(Ok(4294967295), uadic!(5, [0, 4, 1, 3, 2, 4, 2, 0, 0, 4, 4, 2, 2, 3]).u32_value());
        assert_eq!(Err(AdicError::TryFromIntError), uadic!(5, [1, 4, 1, 3, 2, 4, 2, 0, 0, 4, 4, 2, 2, 3]).u32_value());
    }

    #[test]
    fn truncate() {
        let u = uadic!(5, [1, 1, 1, 1, 1]);
        assert_eq!(5, u.d.len());
        assert_eq!(Ok(781), u.u32_value());
        let t = u.truncation(4);
        assert_eq!(4, t.d.len());
        assert_eq!(Ok(156), t.u32_value());
        let t = u.truncation(3);
        assert_eq!(3, t.d.len());
        assert_eq!(Ok(31), t.u32_value());
        let t = u.truncation(2);
        assert_eq!(2, t.d.len());
        assert_eq!(Ok(6), t.u32_value());
        let t = u.truncation(1);
        assert_eq!(1, t.d.len());
        assert_eq!(Ok(1), t.u32_value());
        let t = u.truncation(6);
        assert_eq!(5, t.d.len());
        assert_eq!(Ok(781), t.u32_value());
        let t = u.truncation(0);
        assert_eq!(0, t.d.len());
        assert_eq!(Ok(0), t.u32_value());
        assert!(t.is_local_zero());
    }

    #[test]
    fn pseudo_pn_minus_1_rem_quot() {
        let check = |r: &UAdic, q: &UAdic, a: &UAdic, n: usize| {
            assert_eq!((r.clone(), q.clone()), a.clone().pseudo_pn_minus_1_rem_quot(n));
        };
        check(&zero(), &zero(), &zero(), 0);
        check(&zero(), &zero(), &zero(), 1);
        check(&zero(), &zero(), &zero(), 2);
        for a in [&one(), &two(), &three(), &four()] {
            check(a, &zero(), a, 1);
            check(a, &zero(), a, 2);
        }
        check(&one(), &one(), &five(), 1);
        check(&five(), &zero(), &five(), 2);
        check(&twelve(), &six(), &one_fifty_six(), 2);
        check(&twenty_four(), &twenty_five(), &six_twenty_four(), 2);

        check(&UAdic::new(5, vec![0, 2]), &UAdic::new(5, vec![4, 4]), &UAdic::new(5, vec![1, 2, 3, 4]), 2);
        check(&twenty_four(), &zero(), &twenty_four(), 2);

    }

    #[test]
    fn u_adic_norm() {
        assert_eq!(Valuation::PosInf, zero().valuation());
        assert_eq!(Ratio::ZERO, zero().norm());
        assert_eq!(Valuation::Finite(0), one().valuation());
        assert_eq!(Ratio::new(1, 1), one().norm());
        assert_eq!(Valuation::Finite(0), two().valuation());
        assert_eq!(Ratio::new(1, 1), two().norm());
        assert_eq!(Valuation::Finite(1), five().valuation());
        assert_eq!(Ratio::new(1, 5), five().norm());
        assert_eq!(Valuation::Finite(0), six().valuation());
        assert_eq!(Ratio::new(1, 1), six().norm());
        assert_eq!(Valuation::Finite(2), twenty_five().valuation());
        assert_eq!(Ratio::new(1, 25), twenty_five().norm());
        assert_eq!(Valuation::Finite(3), one_twenty_five().valuation());
        assert_eq!(Ratio::new(1, 125), one_twenty_five().norm());
        assert_eq!(Valuation::Finite(0), one_twenty_six().valuation());
        assert_eq!(Ratio::new(1, 1), one_twenty_six().norm());
    }

    #[test]
    fn nth_root() {

        let check = |a: &UAdic, n: u32, precision: usize, roots: Vec<ZAdic>| {
            // Check each root powers to match a to at least precision digits
            for root in &roots {
                assert_eq!(a.approximation(precision), root.clone().pow(n));
            }
            // Check roots match the output of nth_root
            assert_eq!(Ok(Variety::new(roots)), a.nth_root(n, precision));
        };

        check(&uadic!(2, [1, 0, 0, 0, 1]), 2, 6, vec![
            zadic_approx!(2, 6, [1, 0, 0, 1, 0, 1]),
            zadic_approx!(2, 6, [1, 1, 1, 0, 1, 0]),
        ]);

        check(&uadic!(5, [1]), 2, 6, vec![
            zadic_approx!(5, 6, [1]),
            zadic_approx!(5, 6, [4, 4, 4, 4, 4, 4]),
        ]);

        check(&uadic!(5, [2]), 2, 6, vec![]);

        check(&uadic!(7, [2]), 2, 6, vec![
            zadic_approx!(7, 6, [3, 1, 2, 6, 1, 2]),
            zadic_approx!(7, 6, [4, 5, 4, 0, 5, 4]),
        ]);

    }

}
