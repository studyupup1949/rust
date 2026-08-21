//! Adic number math
//!
//! # Adic numbers
//!
//! p-adic numbers are an alternate number system to the reals.
//! This system is p-periodic and hierarchical.
//! It is used throughout number theory, but it not well-known outside of pure math.
//! This crate is partially an attempt to change that.
//!
//! ## Links
//! - [crates.io/adic](<https://crates.io/crates/adic>)
//! - [docs/adic](<https://docs.rs/adic/latest/adic/>)
//! - [gitlab/adic](https://gitlab.com/pmodular/adic)
//! - [adicmath.com](https://adicmath.com) - our site, to learn about and play with adic numbers
//! - [crates.io/adic-shape](<https://crates.io/crates/adic-shape>) - crate for visualizations and charts of adic numbers
//! - [wiki/P-adic_number](<https://en.wikipedia.org/wiki/P-adic_number>) - wikipedia for p-adic numbers
//!
//!
//! ## Motivation
//!
//! Adic numbers can represent any rational number as well as many numbers between them, just like the real numbers.
//! They can be represented similarly to the reals as infinite digital expansions.
//! Except where the reals have a finite number of digits to the left of the decimal and possibly infinite to the right
//!  (`1.414...`), the adics have finite digits to the right and infinite to the left (`...4132.13`).
//!
//! ```
//! # use adic::{apow, iadic_neg, qadic, radic, uadic, zadic_approx, AdicInteger};
//! assert_eq!("2314._5", uadic!(5, [4, 1, 3, 2]).to_string());
//! assert_eq!("(4)11._5", iadic_neg!(5, [1, 1]).to_string());
//! assert_eq!("(233)4._5", radic!(5, [4], [3, 3, 2]).to_string());
//! assert_eq!("...004123._5", zadic_approx!(5, 6, [3, 2, 1, 4]).to_string());
//! assert_eq!("...0041.23_5", qadic!(zadic_approx!(5, 6, [3, 2, 1, 4]), -2).to_string());
//! assert_eq!("...0ld._25", apow!(zadic_approx!(5, 6, [3, 2, 1, 4]), 2).to_string());
//! ```
//!
//! You might think this means they are "infinite" numbers, but they are not!
//! The key difference is how a number's size is measured.
//!
//! For a number, its "size" is its norm, its absolute value.
//! In the reals, the size of 4 is 4, the size of -2.31 is 2.31, etc.
//!
//! Each p-adic space is linked to a prime, `p`.
//! In the p-adics, the size of a number is the inverse of how many powers of `p` are in it: `|x| = |a/b * p^v| = p^(-v)`.
//! So in the 5-adics, 1, 2, 3, and 4 are all size 1, while 5, 10, 15, and 20 are size 1/5.
//! When you represent these numbers as digital expansions in base-p, the numbers further to the left are SMALLER, not bigger.
//!
//! ```
//! # use num::rational::Ratio;
//! # use adic::{uadic, AdicSized};
//! let one = uadic!(5, [1]);
//! let two = uadic!(5, [2]);
//! let three = uadic!(5, [3]);
//! let five = uadic!(5, [0, 1]);
//! let ten = uadic!(5, [0, 2]);
//! let twenty_five = uadic!(5, [0, 0, 1]);
//! let six_hundred_twenty_five = uadic!(5, [0, 0, 0, 0, 1]);
//! assert_eq!(Ratio::new(1, 1), one.norm());
//! assert_eq!(Ratio::new(1, 1), two.norm());
//! assert_eq!(Ratio::new(1, 1), three.norm());
//! assert_eq!(Ratio::new(1, 5), five.norm());
//! assert_eq!(Ratio::new(1, 5), ten.norm());
//! assert_eq!(Ratio::new(1, 25), twenty_five.norm());
//! assert_eq!(Ratio::new(1, 625), six_hundred_twenty_five.norm());
//! ```
//!
//! Adic numbers are used:
//! - to solve [diophantine equations](https://en.wikipedia.org/wiki/Diophantine_equation)
//! - as fundamental examples of [ultrametric spaces](https://en.wikipedia.org/wiki/Ultrametric_space)
//! - to form combined local/global structures, e.g. [adeles and ideles](https://en.wikipedia.org/wiki/Adelic_algebraic_group)
//! - in glassy physical systems, like in [replica/cavity theory](https://en.wikipedia.org/wiki/Cavity_method)
//! - in [tropical geometry](https://en.wikipedia.org/wiki/Tropical_geometry)
//!
//!
//! ## Crate
//!
//! This crate handles adic numbers, arithmetic, and calculations.
//!
//! Calculations:
//! - Adic arithmetic: Add, Sub, Mul, Div, Pow, Neg
//! - Rootfinding, through use of [hensel lifting](https://en.wikipedia.org/wiki/Hensel%27s_lemma)
//! - Idempotent computation for `AdicComposite` zero divisors
//!
//! Adic number structs:
//! | Struct | Type | Represents | macros | Example |
//! | ------ | ---- | ---------- | ------ | ------- |
//! | [`UAdic`] | Integer | Unsigned ordinary integers | [`uadic`] | 86 = `321._5` |
//! | [`IAdic`] | Integer | Signed ordinary integers | [`iadic_pos`], [`iadic_neg`] | -14 = `(4)21._5` |
//! | [`RAdic`] | Integer | Rationals, non-p-fractional | [`radic`] | 1/6 = `(04)1._5` |
//! | [`ZAdic`] | Integer | Exact & approximate | [`zadic_approx`]/[`zadic_exact`] | 86 ~= `...321._5` |
//! | [`QAdic<A>`] | Fraction | p-fractional numbers | [`qadic`] | 86/5 = `32.1_5` |
//! | [`AdicPower<A>`] | Composite | p^n-adic numbers | [`apow`] | 86 = `3b._25` |
//! | [`AdicComposite`] | Composite | Non-prime adic numbers | - | 86 = `...0086._10 |
//!
//! Related adic structs:
//! - [`AdicValuation`] - Valuation for adic numbers
//! - [`LazyDiv`] - Lazily calculate adic number division
//!
//! Adic number traits:
//! - [`AdicNumber`] - A number with a prime and Add/Mul arithmetic
//! - [`SignedAdicNumber`] - An `AdicNumber` that includes Neg and Sub
//! - [`RationalAdicNumber`] - An `AdicNumber` that can represent rational numbers
//! - [`AdicInteger`] - An adic number without p-fractional digits
//! - [`AdicFraction`] - An adic number with p-fractional digits
//! - [`AdicSized`] - An adic number with valuation and norm
//! - [`AdicApproximate`] - An adic number with certainty and significance
//! - [`HasDigits`] - A structure with indexed digits
//!
//! Polynomials:
//! - [`AdicPolynomial`] - Polynomial with adic integer coefficients
//! - [`AdicVariety`] - A collection of approximate [`AdicNumber`]s representing the roots of a polynomial
//!
//! Divisible:
//! - [`Divisible`] - A trait for structures made of prime decompositions
//! - [`Prime`] - A prime number
//! - [`PrimePower`] - The power of a `Prime`
//! - [`Composite`] - A combination of `PrimePower`s
//! - [`Natural`] - `Composite` or zero (note: NOT a `Divisible` struct)
//!
//! #### Example: calculate the two varieties for 7-adic sqrt(2) to 6 digits:
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use adic::{uadic, zadic_variety, AdicInteger};
//! // Create the 7-adic number 2
//! let seven_adic_two = uadic!(7, [2]);
//! // Take the square root of seven_adic_two, to 6 "decimal places"
//! let sqrt_two_variety = seven_adic_two.nth_root(2, 6);
//! assert_eq!(Ok(zadic_variety!(7, 6, [
//!     [3, 1, 2, 6, 1, 2],
//!     [4, 5, 4, 0, 5, 4],
//! ])), sqrt_two_variety);
//! let roots = sqrt_two_variety?.into_roots().collect::<Vec<_>>();
//! assert_eq!("...216213._7", roots[0].to_string());
//! assert_eq!("...450454._7", roots[1].to_string());
//! # Ok(()) }
//! ```
//!
//! #### Example: 5-adic arithmetic
//! ```
//! use adic::{uadic, radic, AdicInteger};
//! // 3 is a single digit (3) and no repeating digits
//! let three = radic![5, [3], []];
//! // -1/6 consists only of repeating ...040404.
//! let neg_one_sixth = radic![5, [], [4, 0]];
//! // 3 - 1/6 = 17/6 is two digits 12. and then repeating 04
//! let seventeen_sixth = three + neg_one_sixth;
//! assert_eq!(radic![5, [2, 1], [4, 0]], seventeen_sixth);
//! assert_eq!(uadic![5, [2, 1, 4, 0, 4, 0]], seventeen_sixth.truncation(6));
//! ```
//!
//! #### Example: 5-adic fourth roots of unity
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use num::traits::Pow;
//! use adic::{roots_of_unity, zadic_approx, zadic_variety, AdicInteger};
//! // Every (odd) p-adic number space has p-1 roots of unity
//! let roots = roots_of_unity(5, 6)?;
//! assert_eq!(
//!     zadic_variety!(5, 6, [[1], [2, 1, 2, 1, 3, 4], [3, 3, 2, 3, 1, 0], [4, 4, 4, 4, 4, 4]]),
//!     roots
//! );
//! let approx_one = zadic_approx!(5, 6, [1]);
//! for root in roots.into_roots() {
//!     assert_eq!(approx_one, root.pow(4));
//! }
//! # Ok(()) }
//! ```
//!
//! ## TODO
//!
//! - `QXAdic` for a number from a finite extension of [`QAdic`]
//! - `QCAdic` for a number in the algebraic closure of [`QAdic`]
//! - `CAdic`, a "complex adic number", in the norm completion of `QCAdic`
//! - `SAdic`, a "spherically complete adic number", in the spherical completion of `QCAdic`/`CAdic`

mod adic_error;
mod adic_valid;
mod macros;
mod math;
mod num_adic;
mod poly_adic;

use math::Sign;
use num_adic::ExactIntegerVariant;
use poly_adic::{nth_root, num_nth_roots, polynomial_variety, variety_size};

pub use adic_error::{
    AdicError, AdicResult,
};
pub use math::{
    special_function,
    Composite, Divisible, Natural, Prime, PrimePower,
};
pub use num_adic::{
    AdicApproximate, AdicComposite, AdicFraction, AdicInteger, AdicNumber,
    AdicPower, AdicSized, AdicValuation, AdicValuationRing,
    HasDigits, IAdic, LazyDiv,
    RAdic, SignedAdicNumber, RationalAdicNumber, UAdic,
    QAdic, QAdicValuation, ZAdic, ZAdicValuation,
};
pub use poly_adic::{
    roots_of_unity, teichmuller,
    AdicPolynomial, AdicVariety, ZAdicVariety,
};
