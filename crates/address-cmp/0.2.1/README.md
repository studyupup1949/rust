# Easy Address Comparison
[![GitHub license](https://img.shields.io/github/license/Xavientois/address-cmp)](https://github.com/Xavientois/address-cmp)
![GitHub Workflow Status](https://img.shields.io/github/workflow/status/Xavientois/address-cmp/tests)
[![Crates.io](https://img.shields.io/crates/v/address-cmp)](https://crates.io/crates/address-cmp)

A set of macros to allow your types to be compared based on where they are stored in memory. This is useful when two instances of a type should not be considered equal unless they are literally the same instance.

With this crate, you can derive `AddressEq`, `AddressOrd`, or `AddressHash` depending on your needs.

## Usage

```rust
use address_cmp::AddressEq;

#[derive(AddressEq, Debug)]
struct A {
  pub a: u8,
}

let a1 = A { a: 0 };
let a2 = A { a: 0 };

assert_ne!(a1, a2);
assert_eq!(a1, a1);
```
