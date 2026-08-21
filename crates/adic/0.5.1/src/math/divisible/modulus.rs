use num::Integer;
use crate::divisible::{Composite, Prime, PrimePower};


/// Trait for numbers that can serve as a modulus
/// Implementors cannot be zero.
///
/// Implementors:
/// - [`Prime`]
/// - [`PrimePower`]
/// - [`Composite`]
pub trait Modulus where Self: Clone + Into<u32> {

    /// Performing modulus/remainder
    ///
    /// ```
    /// # use adic::divisible::{Composite, Modulus, Prime};
    /// let p = Prime::from(13);
    /// assert_eq!(11, p.modding(24));
    /// let c = Composite::new([(2, 1), (3, 1)]);
    /// assert_eq!(5, c.modding(17));
    /// ```
    fn modding(&self, d: u32) -> u32 {
        let uself: u32 = self.clone().into();
        d % uself
    }

    /// Modular negation
    ///
    /// ```
    /// # use adic::divisible::{Composite, Modulus, Prime};
    /// let p = Prime::from(13);
    /// assert_eq!(11, p.mod_neg(2));
    /// let c = Composite::new([(2, 1), (3, 2), (5, 1)]);
    /// assert_eq!(88, c.mod_neg(2));
    /// assert_eq!(88, c.mod_neg(92));
    /// ```
    fn mod_neg(&self, d: u32) -> u32 {
        let uself: u32 = self.clone().into();
        let dmod = d % uself;
        if dmod == 0 {
            0
        } else {
            uself - dmod
        }
    }

    /// Modular subtraction
    ///
    /// ```
    /// # use adic::divisible::{Composite, Modulus, Prime};
    /// let p = Prime::from(13);
    /// assert_eq!(11, p.mod_sub(2, 4));
    /// let c = Composite::new([(2, 1), (3, 2), (5, 1)]);
    /// assert_eq!(88, c.mod_sub(2, 4));
    /// assert_eq!(88, c.mod_sub(3, 95));
    /// ```
    fn mod_sub(&self, d0: u32, d1: u32) -> u32 {
        let uself: u32 = self.clone().into();
        let d0mod = d0 % uself;
        let d1mod = d1 % uself;
        if d0mod >= d1mod {
            d0mod - d1mod
        } else {
            uself + d0mod - d1mod
        }
    }

    /// Modular exponentiation
    ///
    /// ```
    /// # use adic::divisible::{Modulus, Prime, PrimePower};
    /// let p = Prime::from(13);
    /// assert_eq!(8, p.mod_exp(5, 3));
    /// let pp = PrimePower::from((5, 2));
    /// assert_eq!(2, pp.mod_exp(3, 3));
    /// assert_eq!(2, pp.modding(27));
    /// ```
    fn mod_exp(&self, d: u32, x: u32) -> u32 {

        if d == 0 {
            return 0;
        } else if x == 0 {
            return 1;
        }

        let uself: u32 = self.clone().into();
        let (mut mult, mut ans, mut pow) = (d, 1, x);
        while pow > 1 {
            if pow.is_odd() {
                // ans mult^pow = (ans * mult) * mult^(pow-1)
                ans = ans * mult % uself;
                pow = pow - 1;
            }
            // mult^pow = (mult^2)^(pow/2)
            mult = mult * mult % uself;
            pow = pow / 2;
        }

        (ans * mult) % uself

    }

}


impl Modulus for Prime { }
impl Modulus for PrimePower { }
impl Modulus for Composite { }
