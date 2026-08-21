# Release notes

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
