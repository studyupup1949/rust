<!-- cargo-rdme start -->

Adic number math

# Adic numbers

p-adic numbers are an alternate number system to the reals.
This system is p-periodic and hierarchical.
It is used throughout number theory, but it not well-known outside of pure math.
This crate is partially an attempt to change that.

## Links
- [crates.io/adic](<https://crates.io/crates/adic>)
- [docs/adic](<https://docs.rs/adic/latest/adic/>)
- [gitlab/adic](https://gitlab.com/saplingcalculations/adic)
- [adicmath.com](https://adicmath.com) - learn about and play with adic numbers
- [crates.io/adic-shape](<https://crates.io/crates/adic-shape>) - dependent crate for visualizations and charts of adic numbers
- [wiki/P-adic_number](<https://en.wikipedia.org/wiki/P-adic_number>) - wikipedia for p-adic numbers

## References

- [Murty: Introduction to p-adic...](<https://bookstore.ams.org/amsip-27/>) -
  Introduces p-adics starting from Bernoulli numbers and interpolation; good first book
- [Koblitz: p-adic Numbers...](<https://link.springer.com/book/10.1007/978-1-4612-1112-9>) -
  Introduces p-adics slowly while analyzing zeta functions; good first book
- [Gouveau: p-adic Numbers](https://link.springer.com/book/10.1007/978-3-030-47295-5)
  Widely recommended; likely a good first book
- [Roberts: A Course in p-adic Analysis](<https://link.springer.com/book/10.1007/978-1-4757-3254-2>) -
  p-adics from Z_p through analytical functions; formal and thorough, fantastic


## Motivation

Adic numbers can represent any rational number as well as many numbers between them, just like the real numbers.
They can be represented similarly to the reals as infinite digital expansions.
Except where the reals have a finite number of digits to the **left** of the decimal and possibly infinite to the **right**
 (`1.414...`), the adics have finite digits to the **right** and infinite to the **left** (`...4132.13`).

```rust
assert_eq!("2314._5", UAdic::new(5, vec![4, 1, 3, 2]).to_string());
assert_eq!("2314._5", EAdic::new(5, vec![4, 1, 3, 2]).to_string());
assert_eq!("(4)11._5", EAdic::new_neg(5, vec![1, 1]).to_string());
assert_eq!("(233)4._5", EAdic::new_repeating(5, vec![4], vec![3, 3, 2]).to_string());
assert_eq!("...004123._5", ZAdic::new_approx(5, 6, vec![3, 2, 1, 4]).to_string());
assert_eq!("...0041.23_5", QAdic::new(ZAdic::new_approx(5, 6, vec![3, 2, 1, 4]), -2).to_string());
```

You might think this means they are "infinite" numbers, but they are not.
The key difference is how a number's **size** is measured.

For a number, its size is its norm, its absolute value.
In the reals, the size of `4` is `4`, the size of `-2.31` is `2.31`, etc.

Each p-adic space is linked to a prime, `p`.
In the p-adics, the size of a number is the inverse of how many powers of `p` are in it: `|x| = |a/b * p^v| = p^(-v)`.
So in the 5-adics, 1, 2, 3, and 4 are all size `1`, while 5, 10, 15, and 20 are size `1/5`.
When you represent these numbers as digital expansions in base-p, the numbers further to the left are **smaller**, not bigger.

```rust
let one = EAdic::new(5, vec![1]);
let two = EAdic::new(5, vec![2]);
let three = EAdic::new(5, vec![3]);
let five = EAdic::new(5, vec![0, 1]);
let ten = EAdic::new(5, vec![0, 2]);
let twenty_five = EAdic::new(5, vec![0, 0, 1]);
let six_hundred_twenty_five = EAdic::new(5, vec![0, 0, 0, 0, 1]);
assert_eq!(Ratio::new(1, 1), one.norm());
assert_eq!(Ratio::new(1, 1), two.norm());
assert_eq!(Ratio::new(1, 1), three.norm());
assert_eq!(Ratio::new(1, 5), five.norm());
assert_eq!(Ratio::new(1, 5), ten.norm());
assert_eq!(Ratio::new(1, 25), twenty_five.norm());
assert_eq!(Ratio::new(1, 625), six_hundred_twenty_five.norm());
```

Adic numbers are used:
- to solve [diophantine equations](https://en.wikipedia.org/wiki/Diophantine_equation)
- as fundamental examples of [ultrametric spaces](https://en.wikipedia.org/wiki/Ultrametric_space)
- to form combined local/global structures, e.g. [adeles and ideles](https://en.wikipedia.org/wiki/Adelic_algebraic_group)
- in glassy physical systems, like in [replica/cavity theory](https://en.wikipedia.org/wiki/Cavity_method)
- in [tropical geometry](https://en.wikipedia.org/wiki/Tropical_geometry)


## Crate

This crate handles adic numbers, arithmetic, and calculations.

Calculations:
- Adic arithmetic: Add, Sub, Mul, Div, Pow, Neg
- Rootfinding, through use of [hensel lifting](https://en.wikipedia.org/wiki/Hensel%27s_lemma)
- Polynomial, power series, and sequence manipulations
- Idempotent computation for `MAdic` zero divisors

Adic number structs:

| Struct | Type | Represents | Example |
| ------ | ---- | ---------- | ------- |
| [`UAdic`](num_adic::UAdic) | Integer | Unsigned ordinary integers | 86 = `321._5` |
| [`EAdic`](num_adic::EAdic) | Integer | Exact p-adic integers | -7/6 = `(31)2._5` |
| [`ZAdic`](num_adic::ZAdic) | Integer | Approximate p-adic integers | sqrt(6) ~= `...24031._5` |
| [`QAdic<A>`](num_adic::QAdic) | Fraction | p-fractional numbers | 86/5 = `32.1_5` |
| [`PowAdic<A>`](num_adic::PowAdic) | Composite | p^n-adic numbers | 86 = `3b._25` |
| [`MAdic`](num_adic::MAdic) | Composite | Non-prime adic numbers | 86 = `...0086._10` |

Functions:
- [`Polynomial`](function::Polynomial) - Polynomial, enhanced with adic integer or adic number coefficients e.g. with rootfinding
- [`PowerSeries`](function::PowerSeries) - A power series, useful for expressing special functions
  e.g. [the Iwasawa Log function](function::special::iwasawa_log)
- [`Variety`](function::Variety) - A collection of approximate items representing the roots of a polynomial

#### Example: calculate the two varieties for 7-adic sqrt(2) to 6 digits:
```rust
use adic::{traits::AdicInteger, EAdic, Variety, ZAdic};
// Create the 7-adic number 2
let seven_adic_two = EAdic::new(7, vec![2]);
// Take the square root of seven_adic_two, to 6 "decimal places"
let sqrt_two_variety = seven_adic_two.nth_root(2, 6);
assert_eq!(Ok(Variety::new(vec![
    ZAdic::new_approx(7, 6, vec![3, 1, 2, 6, 1, 2]),
    ZAdic::new_approx(7, 6, vec![4, 5, 4, 0, 5, 4]),
])), sqrt_two_variety);
let roots = sqrt_two_variety?.into_roots().collect::<Vec<_>>();
assert_eq!("...216213._7", roots[0].to_string());
assert_eq!("...450454._7", roots[1].to_string());
```

#### Example: 5-adic arithmetic
```rust
use adic::{traits::CanTruncate, EAdic, UAdic};
// 3 is a single digit (3) and no repeating digits
let three = EAdic::new_repeating(5, vec![3], vec![]);
// -1/6 consists only of repeating ...040404.
let neg_one_sixth = EAdic::new_repeating(5, vec![], vec![4, 0]);
// 3 - 1/6 = 17/6 is two digits 12. and then repeating 04
let seventeen_sixth = three + neg_one_sixth;
assert_eq!(EAdic::new_repeating(5, vec![2, 1], vec![4, 0]), seventeen_sixth);
assert_eq!(UAdic::new(5, vec![2, 1, 4, 0, 4, 0]), seventeen_sixth.truncation(6));
```

#### Example: 5-adic fourth roots of unity
```rust
use num::traits::Pow;
use adic::{traits::AdicInteger, Variety, ZAdic};
// Every (odd) p-adic number space has p-1 roots of unity
let roots = ZAdic::roots_of_unity(5, 6)?;
assert_eq!(
    Variety::new(vec![
        ZAdic::new_approx(5, 6, vec![1]),
        ZAdic::new_approx(5, 6, vec![2, 1, 2, 1, 3, 4]),
        ZAdic::new_approx(5, 6, vec![3, 3, 2, 3, 1, 0]),
        ZAdic::new_approx(5, 6, vec![4, 4, 4, 4, 4, 4]),
    ]),
    roots
);
// Each root taken to the (5-1)-th power is approximately one
for root in roots.into_roots() {
    assert!(root.pow(4).is_approx_one());
}
```


## Future

- `QXAdic` for a number from a finite extension of [`QAdic`](num_adic::QAdic)
- `QCAdic` for a number in the algebraic closure of [`QAdic`](num_adic::QAdic)
- `CAdic`, a "complex adic number", in the norm completion of `QCAdic`
- `SAdic`, a "spherically complete adic number", in the spherical completion of `QCAdic`/`CAdic`

<!-- cargo-rdme end -->

License: MIT OR Apache-2.0
