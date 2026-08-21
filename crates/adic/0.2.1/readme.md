# adic

Adic number math

#### Adic numbers

p-adic numbers are an alternate number system to the reals.
This system is p-periodic and hierarchical.
It is used throughout number theory, but it not well-known outside of pure math.
This crate is partially an attempt to change that.

<https://en.wikipedia.org/wiki/P-adic_number>

Adic numbers can represent any rational number as well as many numbers between them, just like the real numbers.
They can be represented similarly to the reals as infinite digital expansions.
Except where the reals have a finite number of digits to the left of the decimal and possibly infinite to the right
 (`1.414...`), the adics have finite digits to the right and infinite to the left (`...4132.13`).

```rust
assert_eq!("2314._5", uadic!(5, [4, 1, 3, 2]).to_string());
assert_eq!("(231)4._5", radic!(5, [4], [1, 3, 2]).to_string());
```

You might think this means they are "infinite" numbers, but they are not!
The key difference is how a number's size is measured.

For a number, it's "size" is its norm, its absolute value.
In the reals, the size of 4 is 4, the size of -2.31 is 2.31, etc.
In the p-adics, the size is the inverse of how many powers of p are in the number: `|x| = |a/b * p^v| = p^(-v)`.
When you represent these numbers as (base-p) digital expansions, the numbers further to the left are SMALLER, not bigger.

Adic numbers are used:
- as fundamental examples of ultrametric spaces
- to solve diophantine equations
- to form combined local/global structures, e.g. adeles and ideles
- in glassy physical systems, like in replica/cavity theory
- in tropical geometry

#### Crate

This crate handles adic numbers, arithmetic, and calculations, including:
- Adic integers of various types, e.g.
- - [`UAdic`] for natural numbers
- - [`IAdic`] for real integers
- - [`RAdic`] for (most) rationals
- - [`ZAdic`] for approximate numbers
- Rootfinding, through use of hensel lifting

Important objects:
- [`UAdic`]
- [`IAdic`]
- [`RAdic`]
- [`ZAdic`]
- [`AdicInteger`]
- [`AdicInteger::nth_root`]
- [`ZAdicVariety`]

##### Example: calculate the two varieties for 7-adic sqrt(2) to 6 digits:
```rust
use adic::{uadic, zadic_variety, AdicInteger};
// Create the 7-adic number 2
let seven_adic_two = uadic!(7, [2]);
// Take the square root of seven_adic_two, to 6 "decimal places"
let sqrt_two_variety = seven_adic_two.nth_root(2, 6);
assert_eq!(Ok(zadic_variety!(7, 6, [
    [3, 1, 2, 6, 1, 2],
    [4, 5, 4, 0, 5, 4],
])), sqrt_two_variety);
```

##### Example: 5-adic arithmetic
```rust
use adic::{uadic, radic, AdicInteger};
// 3 is a single digit (3) and no repeating digits
let three = radic![5, [3], []];
// -1/6 consists only of repeating ...040404.
let neg_one_sixth = radic![5, [], [4, 0]];
// 3 - 1/6 = 17/6 is two digits 12. and then repeating 04
let seventeen_sixth = three + neg_one_sixth;
assert_eq!(radic![5, [2, 1], [4, 0]], seventeen_sixth);
assert_eq!(uadic![5, [2, 1, 4, 0, 4, 0]], seventeen_sixth.truncation(6));
```

#### TODO

- `QAdic`, an "adic number", to include powers of p in the denominator
- `QXAdic` for a number from a finite extension of `QAdic`
- `QCAdic` for a number in the algebraic closure of `QAdic`
- `CAdic`, a "complex adic number", in the norm completion of `QCAdic`
- `SAdic`, a "spherically complete adic number", in the spherical completion of `QCAdic`/`CAdic`

License: MIT OR Apache-2.0
