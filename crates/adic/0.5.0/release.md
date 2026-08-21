# Release notes

## 0.5.0

- Add a `function` module that contains:
  - `Polynomial`, normal and adic polynomials
  - `PowerSeries`, normal and adic power series
  - `Variety`, roots or solutions to polynomials
  - a `factory` module with basic functions
  - a `special` module with special functions
- Add a `sequence` module that contains:
  - `Sequence`, a trait for sequences (like iterators but as collections)
  - a `factory` module with many different sequences
  - a `derived` module for sequence manipulations, e.g. multiplying two sequences term-by-term
- Use modules to split up the crate, reserving top-level exports for the most important objects
- Add `EAdic` to replace `IAdic` and `RAdic` as exactly holding rational adic integers
- Add `PrimedFrom`/`PrimedInto` to transform from non-p-adic numbers to p-adic numbers by supplying a prime
- Implement `nth_root` and `variety` for fractional adic numbers
- Implement GCD calculations for adic numbers and polynomials
- Implement square-free factorization of polynomials
- Handle degenerate roots during rootfinding
- Add a "real number projection" method to `ZAdic`
- Implement a `local_num` module that has `num`-like traits, e.g. `LocalZero` for a number that can generate a `zero` local to itself: `local_zero(&self)`
- Implement a `normed` module that handles numbers whose "size" or `norm` can be measured, either with archimedian or ultrametric norms
- Implement a `mapping` module with simple "function-like" traits
- Move functionality from adic structs to traits where possible, especially `UAdic` and `ZAdic`
- Gather the `Divisible` traits and structs into a `divisible` module and added `Modulus`
- Genericize:
  - `AdicPolynomial` to `Polynomial`
  - `AdicVariety` to `Variety`
  - `AdicValuation` and `AdicValuationRing` to `Valuation` and `ValuationRing`
- Rename:
  - `AdicPower` to `PowAdic`
  - `AdicComposite` to `MAdic`
  - `AdicNumber` to `AdicPrimitive`
  - `AdicSized` to `UltraNormed` and reworked
  - `AdicApproximate` to `HasApproximateDigits`
- Move `special_function` to the `function` module
- Move `roots_of_unity` and `teichmuller` to be `ZAdic` static member functions
- Remove `IAdic` and `RAdic` from export
- Remove `LazyDiv` from export
- Remove macros from export
- Delete `AdicFraction`, `SignedAdicNumber`, and `RationalAdicNumber`
- Remove `overload` crate
- Re-exported our `num` dependency

## 0.4.1

- Retooled ZAdic to handle UAdic/IAdic/RAdic in an enum variant and a separate certainty
- Add `base` method to HasDigits so that AdicComposite has a digit base

## 0.4.0

- Solve AdicPolynomials to get varieties/roots
- Add num_nth_roots and variety size to hensel lifting methods
- Add Divisibles: Prime, PrimePower, Composite
- Add Natural, representing Composite + Zero
- Add AdicApproximate, AdicFraction, AdicNumber, AdicSized, and HasDigits traits
- Fix p > 9 digital expansions
- Simplify LazyDiv structures
- Refactor and improve rootfinding; more readable, maintainable, and robust
- Add AdicPower for adic numbers with base-p^n
- Add AdicComposite for adic numbers with composite base and "zero divisors"

## 0.3.1

- Fix ZAdic bugs
- ZAdicVariety add access to roots
- Link to adic-shape crate and to site

## 0.3.0

- Improve documentation
- Add QAdic for general adic numbers, generic on the AdicInteger you choose for a unit
- Add AdicPolynomial for polynomials over adic integers, including derivatives
- Implement Div on all adic integers and numbers with a lazy div structure
- Use more explicit error handling everywhere instead of "as"
- Implement ops for references in addition to values
- Reorganize num_adic and poly_adic modules
- Switch up how RAdic/ZAdic display their repeating/unknown digits
- Fixed sign bug in variety_to_digits (now polynomial_variety)

## 0.2.1

- Update display of RAdic and ZAdic
- Add IAdic
- Fix 2-adic roots_of_unity
- Simplify general nth_root

## 0.2.0

- ZAdic representing approximate adic numbers
- ZAdicVariety to hold a collection of related ZAdics
- AdicInteger trait for adic integers
- Implemented Pow
- nth_root takes AdicInteger input and outputs ZAdicVariety
- nth_root can handle more input: p-th roots and roots of p powers
- Added roots of unity for each p-adic
- Added certainty to AdicInteger
- More macros for adic struct creation
- Improved error handling
- Deprecate variety_to_fractions

## 0.1.0

- Hensel lifting with nth_root
- UAdic representing unsigned integers with adic numbers
- RAdic representing (most) rational numbers with adic numbers
- Adic arithmetic: Add, Neg, Sub, Mul
- Adic valuation and norm: |a/b p^v| = p^-v
- Macros for easy adic number creation
- Custom error struct AdicError
