# Release notes

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
