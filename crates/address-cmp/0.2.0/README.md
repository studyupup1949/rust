# Address Comparison

This macro will allow you to annotate your types to indicate that two instances of that type are only equal if they exist at the exact same address (if they are literally the same instance).

With this crate, you can derive `AddressEq`, `AddressOrd`, or `AddressHash` depending on your needs.

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
